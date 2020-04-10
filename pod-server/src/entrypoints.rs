use crate::models::{NewUser, User};
use actix_web::{web, Responder};
use diesel::prelude::*;
use rust_sgx_util::{IasHandle, Nonce, Quote};
use serde::{Deserialize, Serialize};
use tokio_diesel::AsyncRunQueryDsl;

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
    handle: web::Data<IasHandle>,
) -> impl Responder {
    use crate::schema::users::dsl::*;

    log::info!(
        "Received register request for user with login '{}'.",
        info.login
    );
    log::debug!("Received data = {:?}", info);

    // Check if user is already registered.
    let result = users
        .filter(login.eq(&info.login))
        .limit(1)
        .load_async::<User>(&pool)
        .await
        .unwrap();
    if result.len() > 0 {
        return web::Json(RegisterResponse::AlreadyRegistered);
    }

    // Verify the provided data with IAS.
    if let Err(err) = handle.verify_quote(&info.quote, info.nonce.as_ref(), None, None, None, None)
    {
        return web::Json(RegisterResponse::Error(err.to_string()));
    }

    // Add user to the database
    let new_user = NewUser {
        login: &info.login,
        pub_key: pub_key_from_quote(&info.quote),
    };
    diesel::insert_into(users)
        .values(&new_user)
        .execute_async(&pool)
        .await
        .unwrap();
    web::Json(RegisterResponse::Registered)
}
