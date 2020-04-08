mod c;
mod ias;

pub use ias::*;

use std::ops::Deref;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to initialize `IasHandle`.
    #[error("failed to initialize IasHandle")]
    IasInitNullPtr,
    /// `IasHandle::get_sigrl` returned nonzero return code.
    #[error("get_sigrl returned nonzero return code: {}", _0)]
    IasGetSigrlNonZero(i32),
    /// `IasHandle::verify_quote` returned nonzero return code.
    #[error("verify_quote returned nonzero return code: {}", _0)]
    IasVerifyQuoteNonZero(i32),
    /// Error while parsing int from string.
    #[error("parsing int from string: {:?}", _0)]
    ParseInt(#[from] std::num::ParseIntError),
    /// Found unexpected interior nul byte.
    #[error("unexpected interior nul byte: {:?}", _0)]
    Nul(#[from] std::ffi::NulError),
    /// (Windows only) Encountered invalid UTF16.
    #[error("invalid UTF16 encountered: {:?}", _0)]
    Utf16(#[from] std::string::FromUtf16Error),
}

/// Set verbosity on/off.
pub fn set_verbose(verbose: bool) {
    unsafe { c::set_verbose(verbose) }
}

/// A thin wrapper around vector of bytes. Represents quote obtained
/// from the challenged enclave.
#[derive(Debug)]
pub struct Quote(Vec<u8>);

impl From<&[u8]> for Quote {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<Vec<u8>> for Quote {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl Deref for Quote {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A thin wrapper around vector of bytes. Represents nonce obtained
/// from the challenged enclave.
#[derive(Debug)]
pub struct Nonce(Vec<u8>);

impl From<&[u8]> for Nonce {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl Deref for Nonce {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}