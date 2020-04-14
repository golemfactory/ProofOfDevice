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
    #[error("spawning task failed with error: {:?}", _0)]
    TokioJoin(#[from] tokio::task::JoinError),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (code, body) = match self {
            AppError::AlreadyRegistered => (
                StatusCode::BAD_REQUEST,
                "user already registered".to_string(),
            ),
            AppError::NotRegistered => (
                StatusCode::BAD_REQUEST,
                "user not registered yet".to_string(),
            ),
            AppError::TokioDieselAsync(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err))
            }
            // TODO map rust-sgx-util errors to status codes
            AppError::RustSgxUtil(err) => (StatusCode::BAD_REQUEST, format!("{}", err)),
            AppError::R2d2Pool(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)),
            AppError::DieselResult(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)),
            AppError::Var(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)),
            AppError::TokioJoin(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err)),
        };
        HttpResponseBuilder::new(code).json(json!({ "description": body }))
    }
}
