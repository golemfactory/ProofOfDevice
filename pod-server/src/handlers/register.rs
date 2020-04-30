use super::{AppError, Message};
use crate::models::{NewUser, User};
use crate::AppData;

use diesel::prelude::*;

use actix_web::error::BlockingError;
use actix_web::{web, HttpResponse, Responder};
use rust_sgx_util::{IasHandle, Quote};
use serde::Deserialize;
use std::env;
use tokio_diesel::{AsyncRunQueryDsl, OptionalExtension};

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    login: String,
    quote: Quote,
}

pub async fn post(info: web::Json<RegisterInfo>, app_data: web::Data<AppData>) -> impl Responder {
    use crate::schema::users::dsl::*;

    log::info!(
        "Received register request for user with login '{}'.",
        info.login
    );
    log::debug!("Received data: {:?}", info);

    // Check if the user is already registered.
    let result = users
        .filter(login.eq(info.login.clone()))
        .get_result_async::<User>(&app_data.pool)
        .await
        .optional()?;
    log::debug!("Matching user records found: {:?}", result);

    if let Some(_) = result {
        log::info!("User '{}' already registered.", info.login);
        return Err(AppError::AlreadyRegistered);
    }

    let quote = info.quote.clone();
    if let Err(err) = web::block(move || {
        // Verify the provided data with IAS.
        let api_key = env::var("POD_SERVER_API_KEY")?;
        let handle = IasHandle::new(&api_key, None, None)?;
        handle.verify_quote(&quote, None, None, None, None, None)?;
        Ok(())
    })
    .await
    {
        match err {
            BlockingError::Error(err) => return Err(err),
            BlockingError::Canceled => return Err(AppError::ActixBlockingCanceled),
        }
    };

    // Extract pub_key from Quote
    // ED25519 public key is 32 bytes long
    let report_data = info.quote.report_data()?;
    let pub_key_ = base64::encode(&report_data[..32]);
    log::debug!("Extracted public key (base64 encoded): {}", pub_key_);

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
    Ok(HttpResponse::Ok().json(Message::ok().add_param("description", "registration successful")))
}
