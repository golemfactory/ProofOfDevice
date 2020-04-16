use super::AppData;
use crate::error::AppError;
use crate::models::{NewUser, User};

use diesel::prelude::*;

use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use rust_sgx_util::{IasHandle, Nonce, Quote};
use serde::Deserialize;
use serde_json::json;
use std::env;
use tokio::task;
use tokio_diesel::{AsyncRunQueryDsl, OptionalExtension};

fn pub_key_from_quote(quote: &Quote) -> String {
    let (start, stop) = (48 + 320, 48 + 320 + 64);
    let as_bytes = &quote[start..stop];
    // ED25519 public key is 32 bytes long
    base64::encode(&as_bytes[..32])
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

    // Check if the user is already registered.
    let result = users
        .filter(login.eq(info.login.clone()))
        .get_result_async::<User>(&app_data.pool)
        .await
        .optional()?;
    log::debug!("Matching user records found: {:?}", result);

    if let Some(_) = result {
        log::info!("User already registered.");
        return Err(AppError::AlreadyRegistered);
    }

    let quote = info.quote.clone();
    let nonce = info.nonce.clone();
    task::spawn_blocking(move || verify_quote(&quote, nonce.as_ref())).await??;

    // Extract pub_key from Quote
    let pub_key_ = pub_key_from_quote(&info.quote);
    log::debug!("Extracted public key (base64 encoded): {:?}", pub_key_);

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

pub async fn get_auth(session: Session, identity: Identity) -> impl Responder {
    if let Some(id) = identity.identity() {
        log::info!("User '{}' already authenticated!", id);
        return Err(AppError::AlreadyAuthenticated);
    }

    log::info!("Received challenge request.");

    // Send challenge.
    let challenge = match session.get::<String>("challenge") {
        Err(_) => return Err(AppError::InvalidCookie),
        Ok(Some(challenge)) => challenge,
        Ok(None) => {
            let mut blob = [0u8; 64];
            getrandom::getrandom(&mut blob)?;
            let challenge = base64::encode(&blob[..]);

            log::debug!("Generated challenge: {}", challenge);

            if let Err(_) = session.set("challenge", challenge.clone()) {
                return Err(AppError::InvalidCookie);
            }

            challenge
        }
    };

    Ok(HttpResponse::Ok().json(json!({ "challenge": challenge })))
}

#[derive(Deserialize)]
pub struct AuthChallengeResponse {
    login: String,
    response: String, // base64 encoded
}

pub async fn auth(
    response: web::Json<AuthChallengeResponse>,
    app_data: web::Data<AppData>,
    session: Session,
    identity: Identity,
) -> impl Responder {
    if let Some(ident) = identity.identity() {
        log::info!("User '{}' already authenticated!", ident);
        return Err(AppError::AlreadyAuthenticated);
    }

    use crate::schema::users::dsl::*;

    log::info!(
        "Received challenge response from user with login '{}'.",
        response.login
    );

    // Fetch user's record and extract their pub_key.
    let record = users
        .filter(login.eq(response.login.clone()))
        .get_result_async::<User>(&app_data.pool)
        .await
        .optional()?;
    let record = match record {
        Some(record) => record,
        None => return Err(AppError::NotRegistered),
    };

    let blob = match session
        .get::<String>("challenge")
        .map_err(|_| AppError::InvalidCookie)?
    {
        Some(challenge) => base64::decode(challenge)?,
        None => return Err(AppError::InvalidChallenge),
    };

    let pub_key_ = ed25519_dalek::PublicKey::from_bytes(&base64::decode(&record.pub_key)?)?;
    let enc_blob = base64::decode(&response.response)?;
    let signature = ed25519_dalek::Signature::from_bytes(&enc_blob)?;
    pub_key_.verify(&blob, &signature)?;

    identity.remember(response.login.clone());
    session.purge();

    Ok(HttpResponse::Ok())
}

pub async fn index(identity: Identity) -> impl Responder {
    match identity.identity() {
        Some(_) => Ok(HttpResponse::Ok().json(json!({"description": "You are authenticated!"}))),
        None => Err(AppError::NotAuthenticated),
    }
}
