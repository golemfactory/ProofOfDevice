//! A safe wrapper around Graphene's [`sgx_util`] C-library.
//!
//! [`sgx_util`]: https://github.com/oscarlab/graphene/tree/master/Pal/src/host/Linux-SGX/tools
//!
//! ```toml
//! rust-sgx-util = "0.2"
//! ```
//!
//! For `serde` support, you can enable it with `with_serde` feature:
//!
//! ```toml
//! rust-sgx-util = { version = "0.2", features = ["with_serde"] }
//! ```
//!
//! ## Prerequisites
//!
//! Currently, this crate requires you compile and install `sgx_util` as
//! a shared library.
//!
//! ## Usage examples
//!
//! You can find usage examples in the `examples` dir of the crate.
//!
mod c;
mod ias;
#[cfg(feature = "with_serde")]
mod ser_de;

pub use ias::*;

#[cfg(feature = "with_serde")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::ops::Deref;

/// Convenience wrapper around fallible operation.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type thrown by fallible operations in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to initialize `IasHandle`.
    #[error("failed to initialize IasHandle")]
    IasInitNullPtr,
    /// `Quote`'s size is too small.
    #[error("quote's size is too small")]
    QuoteTooShort,
    /// `Nonce` exceeded 32 bytes.
    #[error("nonce exceeded 32 bytes")]
    NonceTooLong,
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

impl Quote {
    const REPORT_BEGIN: usize = 368;
    const REPORT_END: usize = 432;

    /// Returns `report_data` bytes embedded within this `Quote`.
    ///
    /// The size of the returned slice is 64 bytes.
    ///
    /// # Errors
    ///
    /// This function will fail with [`Error::QuoteTooShort`] if `Quote`
    /// is shorter than `432` bytes. Note that this is only a quick check
    /// that we can extract the region in `Quote`'s buffer where we expect
    /// the `report_data` to lie in. We don't do any validations on the
    /// `Quote` in this method.
    ///
    /// [`Error::QuoteTooShort`]: enum.Error.html#variant.QuoteTooShort
    ///
    /// # Examples
    ///
    /// ```
    /// # use rust_sgx_util::Quote;
    /// let quote = Quote::from(&[0u8; 438][..]);
    /// assert_eq!(quote.report_data().unwrap().len(), 64);
    ///
    /// let quote = Quote::from(&[0u8; 10][..]);
    /// assert!(quote.report_data().is_err());
    /// ```
    pub fn report_data(&self) -> Result<&[u8]> {
        self.0
            .get(Self::REPORT_BEGIN..Self::REPORT_END)
            .ok_or(Error::QuoteTooShort)
    }
}

impl From<&[u8]> for Quote {
    fn from(bytes: &[u8]) -> Self {
        Self::from(bytes.to_vec())
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
/// Nonce cannot be longer than 32 bytes.
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

impl Nonce {
    fn new<B: Into<Vec<u8>>>(bytes: B) -> Result<Self> {
        let bytes = bytes.into();
        if bytes.len() > 32 {
            return Err(Error::NonceTooLong);
        }
        Ok(Self(bytes))
    }
}

impl TryFrom<&[u8]> for Nonce {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::new(bytes)
    }
}

impl Deref for Nonce {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
