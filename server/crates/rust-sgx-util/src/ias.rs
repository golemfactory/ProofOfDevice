use crate::{c, Error, Result};
use std::ffi::CString;
use std::ptr::{self, NonNull};
use std::str::FromStr;
use std::{fmt, slice, u32};

const IAS_VERIFY_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/report";
const IAS_SIGRL_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/sigrl";

/// Represents a handle to Intel's Attestation Service. It allows the user
/// to perform operations such as getting a SigRL for a specified `GroupId`,
/// or verifying a specified quote with the IAS.
pub struct IasHandle {
    // We need to store `verify_url` and `sigrl_url` due to a bug in the current
    // implementation of `sgx_util` lib which does not copy out the buffers
    // passed in as args to `ias_init` function.
    verify_url: CString,
    sigrl_url: CString,
    context: NonNull<c::IasContext>,
}

impl IasHandle {
    // TODO API key should probably have its own type that does
    // at the very least some length validation
    /// Create new `IasHandle` with the specified `api_key` API key,
    /// IAS verification URL `verify_url`, and IAS SigRL URL `sigrl_url`.
    /// 
    /// By default, the following URLs are used:
    /// * IAS verification - [dev/attestation/v3/report]
    /// * IAS SigRL - [dev/attestation/v3/sigrl]
    /// 
    /// [dev/attestation/v3/report]: https://api.trustedservices.intel.com/sgx/dev/attestation/v3/report
    /// [dev/attestation/v3/sigrl]: https://api.trustedservices.intel.com/sgx/dev/attestation/v3/sigrl
    /// 
    /// # Errors
    /// 
    /// This function will fail with `Error::IasInitNullPtr` if initialisation
    /// of the handle is unsuccessful.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use rust_sgx_util::ias::*;
    /// # fn main() -> anyhow::Result<()> {
    /// let _handle = IasHandle::new("012345abcdef", None, None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(api_key: &str, verify_url: Option<&str>, sigrl_url: Option<&str>) -> Result<Self> {
        let api_key = CString::new(api_key)?;
        let verify_url = verify_url.unwrap_or(IAS_VERIFY_URL);
        let verify_url = CString::new(verify_url)?;
        let sigrl_url = sigrl_url.unwrap_or(IAS_SIGRL_URL);
        let sigrl_url = CString::new(sigrl_url)?;
        let raw_context =
            unsafe { c::ias_init(api_key.as_ptr(), verify_url.as_ptr(), sigrl_url.as_ptr()) };
        let context = NonNull::new(raw_context).ok_or(Error::IasInitNullPtr)?;
        Ok(Self {
            verify_url,
            sigrl_url,
            context,
        })
    }

    /// Obtain SigRL for the given `GroupId` `group_id`.
    /// 
    /// # Errors
    /// 
    /// This function will fail with `Error::GetSigrlNonZero(_)` if the
    /// `group_id` is invalid, or the `IasHandle` was created with an
    /// invalid IAS verification URL.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use rust_sgx_util::ias::*;
    /// use std::str::FromStr;
    /// # fn main() -> anyhow::Result<()> {
    /// let handle = IasHandle::new("012345abcdef", None, None)?;
    /// let group_id = GroupId::from_str("01234567")?;
    /// let res = handle.get_sigrl(&group_id);
    /// assert!(res.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_sigrl(&self, group_id: &GroupId) -> Result<Option<Sigrl>> {
        let mut size: usize = 0;
        let mut raw = ptr::null_mut();
        let ret = unsafe {
            c::ias_get_sigrl(
                self.context.as_ptr(),
                group_id.as_bytes().as_ptr(),
                &mut size,
                &mut raw,
            )
        };
        if ret == 0 {
            if size == 0 {
                // No SigRL for given EPID group id
                Ok(None)
            } else {
                let sigrl = unsafe { Sigrl::new(raw as *const u8, size) };
                Ok(Some(sigrl))
            }
        } else {
            Err(Error::GetSigrlNonZero(ret))
        }
    }

    pub fn verify_quote(&self) -> Result<()> {
        unimplemented!("verify_quote")
    }
}

impl Drop for IasHandle {
    fn drop(&mut self) {
        unsafe { c::ias_cleanup(self.context.as_ptr()) }
    }
}

/// Stores the result of `IasHandle::get_sigrl` function call, i.e., the SigRL
/// for the specified `GroupId`.
#[derive(Debug)]
pub struct Sigrl {
    buffer: Vec<u8>,
}

impl Sigrl {
    unsafe fn new(raw: *const u8, size: usize) -> Self {
        let slice = slice::from_raw_parts(raw, size);
        let buffer = slice.to_vec();
        Self { buffer }
    }

    /// Return SigRL as pure bytes slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

impl fmt::Display for Sigrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Sigrl(")?;
        for b in &self.buffer {
            f.write_fmt(format_args!("{:#b}", b))?;
        }
        f.write_str(")")
    }
}

/// Represents EPID group ID.
/// 
/// This structure is necessary to invoke `IasHandle::get_sigrl` function.
/// 
/// # Creating `GroupId`
/// 
/// Currently, the only way to create an instance of `GroupId`, is from `&str`
/// slice via the `std::str::FromStr::from_str` method. Note also that currently
/// prepending "0x" to the string is invalid, and will result in `Error::ParseInt(_)`
/// error.
/// 
/// ```
/// # use rust_sgx_util::ias::GroupId;
/// use std::str::FromStr;
/// assert!(GroupId::from_str("01234567").is_ok());
/// assert!(GroupId::from_str("0x01234567").is_err()); // prepending "0x" is currently invalid
/// ```
#[derive(Debug)]
pub struct GroupId {
    inner: [u8; 4],
}

impl GroupId {
    /// Return `GroupId` as pure bytes slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl FromStr for GroupId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parsed = u32::from_str_radix(s, 16)?;
        let inner = parsed.to_le_bytes();
        Ok(GroupId { inner })
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GroupId({:#010x})", u32::from_le_bytes(self.inner))
    }
}
