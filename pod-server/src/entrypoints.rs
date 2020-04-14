use super::AppData;
use crate::error::AppError;
use crate::models::{NewUser, User};

use diesel::prelude::*;

use actix_web::{web, HttpResponse, Responder};
use rust_sgx_util::{IasHandle, Nonce, Quote};
use serde::Deserialize;
use std::env;
use tokio::task;
use tokio_diesel::AsyncRunQueryDsl;

fn pub_key_from_quote(_quote: &Quote) -> String {
    "0123456789abcdef".to_string()
}

fn verify_quote(quote: &Quote, nonce: Option<&Nonce>) -> Result<(), AppError> {
    // Verify the provided data with IAS.
    let api_key = env::var("POD_SERVER_API_KEY")?;
    let handle = IasHandle::new(&api_key, None, None)?;
    handle.verify_quote(quote, nonce, None, None, None, None)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    login: String,
    quote: Quote,
    nonce: Option<Nonce>,
}

pub async fn register(
    info: web::Json<RegisterInfo>,
    app_data: web::Data<AppData>,
) -> impl Responder {
    use crate::schema::users::dsl::*;

    log::info!(
        "Received register request for user with login '{}'.",
        info.login
    );
    log::debug!("Received data: {:?}", info);

    // Check if user is already registered.
    let result = users
        .filter(login.eq(info.login.clone()))
        .limit(1)
        .load_async::<User>(&app_data.pool)
        .await?;
    log::debug!("Matching user records found: {:?}", result);

    if result.len() > 0 {
        log::info!("User already registered.");
        return Err(AppError::AlreadyRegistered);
    }

    let quote = info.quote.clone();
    let nonce = info.nonce.clone();
    task::spawn_blocking(move || verify_quote(&quote, nonce.as_ref())).await??;

    // Extract pub_key from Quote
    let pub_key_ = pub_key_from_quote(&info.quote);

    // Insert user to the database.
    let new_user = NewUser {
        login: info.login.clone(),
        pub_key: pub_key_,
    };
    diesel::insert_into(users)
        .values(new_user)
        .execute_async(&app_data.pool)
        .await?;

    log::info!("User '{}' successfully inserted into db.", info.login);

    Ok(HttpResponse::Ok())
}
