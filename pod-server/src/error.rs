use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("User already registered")]
    AlreadyRegistered,
    #[error("tokio_diesel async op failed with error: {:?}", _0)]
    TokioDieselAsync(#[from] tokio_diesel::AsyncError),
    #[error("rust_sgx_util error: {:?}", _0)]
    RustSgxUtil(#[from] rust_sgx_util::Error),
    #[error("r2d2 pool error {:?}", _0)]
    R2d2Pool(#[from] diesel::r2d2::PoolError),
    #[error("diesel result error {:?}", _0)]
    DieselResult(#[from] diesel::result::Error),
    #[error("oneshot canceled {:?}", _0)]
    Oneshot(#[from] futures::channel::oneshot::Canceled),
    #[error("var not found in env {:?}", _0)]
    Var(#[from] std::env::VarError),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let code = match self {
            AppError::AlreadyRegistered => StatusCode::NOT_FOUND,
            AppError::TokioDieselAsync(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // TODO map rust-sgx-util errors to status codes
            AppError::RustSgxUtil(_) => StatusCode::BAD_REQUEST,
            AppError::R2d2Pool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DieselResult(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Oneshot(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Var(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        HttpResponse::new(code)
    }
}
