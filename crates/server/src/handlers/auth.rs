use super::{AppError, Message};
use crate::models::User;
use crate::AppData;

use diesel::prelude::*;

use actix_identity::Identity;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use tokio_diesel::{AsyncRunQueryDsl, OptionalExtension};

pub async fn get(session: Session, identity: Identity) -> impl Responder {
    if let Some(id) = identity.identity() {
        log::info!("User '{}' already authenticated", id);
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

    Ok(HttpResponse::Ok().json(
        Message::ok()
            .add_param("description", "challenge successfully generated")
            .add_param("challenge", challenge),
    ))
}

#[derive(Deserialize)]
pub struct AuthChallengeResponse {
    login: String,
    response: String, // base64 encoded
}

pub async fn post(
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

    Ok(
        HttpResponse::Ok()
            .json(Message::ok().add_param("description", "authentication successful")),
    )
}
