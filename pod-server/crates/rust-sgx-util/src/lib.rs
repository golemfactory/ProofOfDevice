//! A safe wrapper around Graphene's [`sgx_util`] C-library.
//!
//! [`sgx_util`]: https://github.com/oscarlab/graphene/tree/master/Pal/src/host/Linux-SGX/tools
//!
//! # Prerequisites
//!
//! Currently, this crate requires you compile and install `sgx_util` as
//! a shared library.
//!
//! # Usage examples
//!
//! You can find usage examples in the `examples` dir of the crate.
//!
//! # Enabling `serde` support
//!
//! By default, `serde` support for serialization/deserialization of client-side structures
//! such as [`Quote`] is not enabled. To enable it, specify `with_serde` feature flag in your
//! `Cargo.toml`:
//!
//! ```toml
//! rust-sgx-util = { version = "0.2", features = ["with_serde"] }
//! ```
//!
mod c;
mod ias;
#[cfg(feature = "with_serde")]
mod ser_de;

pub use ias::*;

#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Convenience wrapper around fallible operation.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type thrown by fallible operations in this crate.
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
///
/// # Accessing the underlying bytes buffer
///
/// `Quote` implements `Deref<Target=[u8]>`, therefore dereferencing it will
/// yield its inner buffer of bytes.
///
/// # Serializing/deserializing
///
/// With `with_serde` feature enabled, `Quote` can be serialized and deserialized
/// as base64 `String`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
pub struct Quote(#[cfg_attr(feature = "with_serde", serde(with = "ser_de"))] Vec<u8>);

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
///
/// # Accessing the underlying bytes buffer
///
/// `Nonce` implements `Deref<Target=[u8]>`, therefore dereferencing it will
/// yield its inner buffer of bytes.
/// 
/// # Serializing/deserializing
///
/// With `with_serde` feature enabled, `Nonce` can be serialized and deserialized
/// as base64 `String`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "with_serde", derive(Serialize, Deserialize))]
pub struct Nonce(#[cfg_attr(feature = "with_serde", serde(with = "ser_de"))] Vec<u8>);

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
