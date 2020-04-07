mod c;
pub mod ias;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to initialize IasHandle")]
    IasInitNullPtr,
    #[error("get_sigrl returned nonzero return code: {}", _0)]
    GetSigrlNonZero(i32),
    #[error("parsing int from string: {:?}", _0)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("unexpected interior nul byte: {:?}", _0)]
    Nul(#[from] std::ffi::NulError),
}
