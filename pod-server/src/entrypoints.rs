use super::AppData;
use crate::models::{NewUser, User};
use actix_web::{web, Responder};
use diesel::prelude::*;
use rust_sgx_util::{Nonce, Quote};
use serde::{Deserialize, Serialize};
use tokio_diesel::AsyncRunQueryDsl;

fn pub_key_from_quote(_quote: &Quote) -> String {
    "0123456789abcdef".to_string()
}

#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    login: String,
    quote: Quote,
    nonce: Option<Nonce>,
}

#[derive(Serialize)]
pub enum RegisterResponse {
    AlreadyRegistered,
    Registered,
    Error(String),
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
    log::debug!("Received data = {:?}", info);

    // Check if user is already registered.
    let result = users
        .filter(login.eq(info.login.clone()))
        .limit(1)
        .load_async::<User>(&app_data.pool)
        .await
        .unwrap();
    if result.len() > 0 {
        return web::Json(RegisterResponse::AlreadyRegistered);
    }

    // Verify the provided data with IAS.
    if let Err(err) =
        app_data
            .handle
            .verify_quote(&info.quote, info.nonce.as_ref(), None, None, None, None)
    {
        return web::Json(RegisterResponse::Error(err.to_string()));
    }

    // Add user to the database
    let new_user = NewUser {
        login: info.login.clone(),
        pub_key: pub_key_from_quote(&info.quote),
    };
    diesel::insert_into(users)
        .values(new_user)
        .execute_async(&app_data.pool)
        .await
        .unwrap();
    web::Json(RegisterResponse::Registered)
}
