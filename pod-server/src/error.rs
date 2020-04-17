use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("User already registered")]
    AlreadyRegistered,
    #[error("User not registered yet")]
    NotRegistered,
    #[error("Invalid challenge")]
    InvalidChallenge,
    #[error("Invalid cookie")]
    InvalidCookie,
    #[error("User not authenticated")]
    NotAuthenticated,
    #[error("User already authenticated")]
    AlreadyAuthenticated,
    #[error("tokio_diesel async op failed with error: {:?}", _0)]
    TokioDieselAsync(#[from] tokio_diesel::AsyncError),
    #[error("rust_sgx_util error: {:?}", _0)]
    RustSgxUtil(#[from] rust_sgx_util::Error),
    #[error("r2d2 pool error {:?}", _0)]
    R2d2Pool(#[from] diesel::r2d2::PoolError),
    #[error("diesel result error {:?}", _0)]
    DieselResult(#[from] diesel::result::Error),
    #[error("var not found in env {:?}", _0)]
    Var(#[from] std::env::VarError),
    #[error("blocking operation was canceled prematurely")]
    ActixBlockingCanceled,
    #[error("decoding base64 to blob: {:?}", _0)]
    Base64Decode(#[from] base64::DecodeError),
    #[error("parsing ed25519 signature: {:?}", _0)]
    Ed25519Signature(#[from] ed25519_dalek::SignatureError),
    #[error("fetching entropy: {:?}", _0)]
    GetRandom(#[from] getrandom::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let code = match self {
            AppError::AlreadyRegistered => StatusCode::BAD_REQUEST,
            AppError::NotRegistered => StatusCode::BAD_REQUEST,
            AppError::InvalidChallenge => StatusCode::BAD_REQUEST,
            AppError::InvalidCookie => StatusCode::BAD_REQUEST,
            AppError::NotAuthenticated => StatusCode::UNAUTHORIZED,
            AppError::AlreadyAuthenticated => StatusCode::BAD_REQUEST,
            AppError::TokioDieselAsync(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // TODO map rust-sgx-util errors to status codes
            AppError::RustSgxUtil(_) => StatusCode::BAD_REQUEST,
            AppError::R2d2Pool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DieselResult(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Var(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ActixBlockingCanceled => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Base64Decode(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Ed25519Signature(_) => StatusCode::BAD_REQUEST,
            AppError::GetRandom(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = format!("{}", self);
        HttpResponseBuilder::new(code).json(json!({ "description": body }))
    }
}
