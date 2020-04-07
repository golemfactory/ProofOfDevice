mod c;
pub mod ias;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to initialize IasHandle")]
    IasInitNullPtr,
    #[error("get_sigrl returned nonzero return code: {}", 0)]
    GetSigrlNonZero(i32),
}