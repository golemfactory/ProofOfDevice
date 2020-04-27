pub mod auth;
pub mod register;

use actix_identity::Identity;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, Responder, ResponseError};
use serde::Serialize;
use std::collections::HashMap;

pub async fn index(identity: Identity) -> impl Responder {
    match identity.identity() {
        Some(_) => Ok(HttpResponse::NoContent()),
        None => Err(AppError::NotAuthenticated),
    }
}

#[derive(Serialize)]
enum Status {
    Ok,
    Error,
}

#[derive(Serialize)]
struct Message {
    status: Status,
    #[serde(flatten)]
    params: HashMap<String, String>,
}

impl Message {
    fn ok() -> Self {
        Self {
            status: Status::Ok,
            params: HashMap::new(),
        }
    }

    fn error() -> Self {
        Self {
            status: Status::Error,
            params: HashMap::new(),
        }
    }

    fn add_param<S1: AsRef<str>, S2: AsRef<str>>(mut self, key: S1, value: S2) -> Self {
        self.params
            .insert(key.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("user not registered yet")]
    NotRegistered,
    #[error("user already registered")]
    AlreadyRegistered,
    #[error("invalid challenge")]
    InvalidChallenge,
    #[error("invalid cookie")]
    InvalidCookie,
    #[error("user not authenticated")]
    NotAuthenticated,
    #[error("user already authenticated")]
    AlreadyAuthenticated,
    #[error("tokio_diesel async op failed with error: {}", _0)]
    TokioDieselAsync(#[from] tokio_diesel::AsyncError),
    #[error("attesting quote failed with error: {}", _0)]
    RustSgxUtil(#[from] rust_sgx_util::Error),
    #[error("r2d2 pool error: {}", _0)]
    R2d2Pool(#[from] diesel::r2d2::PoolError),
    #[error("diesel result error {}", _0)]
    DieselResult(#[from] diesel::result::Error),
    #[error("var not found in env: {}", _0)]
    Var(#[from] std::env::VarError),
    #[error("blocking operation was canceled prematurely")]
    ActixBlockingCanceled,
    #[error("decoding base64 to blob: {}", _0)]
    Base64Decode(#[from] base64::DecodeError),
    #[error("parsing ed25519 signature: {}", _0)]
    Ed25519Signature(#[from] ed25519_dalek::SignatureError),
    #[error("fetching entropy: {}", _0)]
    GetRandom(#[from] getrandom::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (code, message) = match self {
            AppError::NotRegistered => (StatusCode::FORBIDDEN, Message::error()),
            AppError::AlreadyRegistered => (StatusCode::OK, Message::ok()),
            AppError::InvalidChallenge => (StatusCode::OK, Message::error()),
            AppError::InvalidCookie => (StatusCode::OK, Message::error()),
            AppError::NotAuthenticated => (StatusCode::FORBIDDEN, Message::error()),
            AppError::AlreadyAuthenticated => (StatusCode::OK, Message::ok()),
            AppError::TokioDieselAsync(_) => (StatusCode::INTERNAL_SERVER_ERROR, Message::error()),
            AppError::RustSgxUtil(_) => (StatusCode::OK, Message::error()),
            AppError::R2d2Pool(_) => (StatusCode::INTERNAL_SERVER_ERROR, Message::error()),
            AppError::DieselResult(_) => (StatusCode::INTERNAL_SERVER_ERROR, Message::error()),
            AppError::Var(_) => (StatusCode::INTERNAL_SERVER_ERROR, Message::error()),
            AppError::ActixBlockingCanceled => {
                (StatusCode::INTERNAL_SERVER_ERROR, Message::error())
            }
            AppError::Base64Decode(_) => (StatusCode::INTERNAL_SERVER_ERROR, Message::error()),
            AppError::Ed25519Signature(_) => (StatusCode::OK, Message::error()),
            AppError::GetRandom(_) => (StatusCode::INTERNAL_SERVER_ERROR, Message::error()),
        };
        HttpResponseBuilder::new(code).json(message.add_param("description", format!("{}", self)))
    }
}
