use crate::{c, Error, Nonce, Quote, Result};
use std::ffi::CString;
use std::ops::Deref;
use std::path::Path;
use std::ptr::{self, NonNull};
use std::str::FromStr;
use std::{fmt, slice, u32};

const IAS_VERIFY_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/report";
const IAS_SIGRL_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/sigrl";

/// Represents a handle to Intel's Attestation Service. It allows the user
/// to perform operations such as getting a SigRL for a specified [`GroupId`],
/// or verifying a specified quote with the IAS.
/// 
/// [`GroupId`]: struct.GroupId.html
pub struct IasHandle {
    // We need to store `verify_url` and `sigrl_url` due to a bug in the current
    // implementation of `sgx_util` lib which does not copy out the buffers
    // passed in as args to `ias_init` function.
    #[allow(dead_code)]
    verify_url: CString,
    #[allow(dead_code)]
    sigrl_url: CString,
    context: NonNull<c::IasContext>,
}

impl IasHandle {
    // TODO API key should probably have its own type that does
    // at the very least some length validation
    /// Create new instance with the specified `api_key` API key,
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
    /// This function will fail with [`Error::IasInitNullPtr`] if initialisation
    /// of the handle is unsuccessful.
    /// 
    /// [`Error::IasInitNullPtr`]: enum.Error.html#variant.IasInitNullPtr
    ///
    /// # Examples
    ///
    /// ```
    /// # use rust_sgx_util::*;
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

    /// Obtain SigRL for the given `group_id`.
    ///
    /// # Errors
    ///
    /// This function will fail with [`Error::IasGetSigrlNonZero(_)`] if the
    /// `group_id` is invalid, or the `IasHandle` was created with an
    /// invalid IAS verification URL.
    /// 
    /// [`Error::IasGetSigrlNonZero(_)`]: enum.Error.html#variant.IasGetSigrlNonZero
    ///
    /// # Examples
    ///
    /// ```
    /// # use rust_sgx_util::*;
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
                group_id.as_ptr(),
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
            Err(Error::IasGetSigrlNonZero(ret))
        }
    }

    /// Verify provided quote.
    /// 
    /// # Errors
    /// 
    /// This function will fail with [`Error::IasVerifyQuoteNonZero(_)`] if the
    /// provided `quote` is invalid, or the `nonce`, or if the IAS server
    /// returns a non 200 status code.
    /// 
    /// [`Error::IasVerifyQuoteNonZero(_)`]: enum.Error.html#variant.IasVerifyQuoteNonZero
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use rust_sgx_util::*;
    /// # fn main() -> anyhow::Result<()> {
    /// let handle = IasHandle::new("012345abcdef", None, None)?;
    /// let quote = Quote::from(vec![0u8; 100]);
    /// let res = handle.verify_quote(&quote, None, None, None, None, None);
    /// assert!(res.is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_quote(
        &self,
        quote: &Quote,
        nonce: Option<&Nonce>,
        report_path: Option<&Path>,
        sig_path: Option<&Path>,
        cert_path: Option<&Path>,
        advisory_path: Option<&Path>,
    ) -> Result<()> {
        let empty: &[u8] = &[];
        let nonce = match nonce {
            Some(nonce) => CString::new(nonce.deref())?,
            None => CString::new(empty)?,
        };
        let report_path = match report_path {
            Some(path) => path_to_c_string(path)?,
            None => CString::new(empty)?,
        };
        let sig_path = match sig_path {
            Some(path) => path_to_c_string(path)?,
            None => CString::new(empty)?,
        };
        let cert_path = match cert_path {
            Some(path) => path_to_c_string(path)?,
            None => CString::new(empty)?,
        };
        let advisory_path = match advisory_path {
            Some(path) => path_to_c_string(path)?,
            None => CString::new(empty)?,
        };
        let ret = unsafe {
            c::ias_verify_quote(
                self.context.as_ptr(),
                quote.as_ptr() as *const _,
                quote.len(),
                nonce.as_ptr(),
                report_path.as_ptr(),
                sig_path.as_ptr(),
                cert_path.as_ptr(),
                advisory_path.as_ptr(),
            )
        };
        if ret == 0 {
            Ok(())
        } else {
            Err(Error::IasVerifyQuoteNonZero(ret))
        }
    }
}

impl Drop for IasHandle {
    fn drop(&mut self) {
        unsafe { c::ias_cleanup(self.context.as_ptr()) }
    }
}

#[cfg(unix)]
fn path_to_c_string(path: &Path) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    let s = CString::new(path.as_os_str().as_bytes())?;
    Ok(s)
}

#[cfg(windows)]
fn path_to_c_string(path: &Path) -> Result<CString> {
    use std::os::windows::ffi::OsStringExt;
    let utf16: Vec<_> = path.as_os_str().encode_wide().collect();
    let s = String::from_utf16(utf16)?;
    let s = CString::new(s.as_bytes())?;
    Ok(s)
}

/// A thin wrapper around vector of bytes. Stores the result of
/// [`IasHandle::get_sigrl`] function call, i.e., the SigRL
/// for the specified [`GroupId`].
/// 
/// [`IasHandle::get_sigrl`]: struct.IasHandle.html#method.get_sigrl
/// [`GroupId`]: struct.GroupId.html
/// 
/// # Accessing the underlying bytes buffer
/// 
/// `Sigrl` implements `Deref<Target=[u8]>`, therefore dereferencing it will
/// yield its inner buffer of bytes.
#[derive(Debug)]
pub struct Sigrl(Vec<u8>);

impl Sigrl {
    unsafe fn new(raw: *const u8, size: usize) -> Self {
        let slice = slice::from_raw_parts(raw, size);
        Self(slice.to_vec())
    }
}

impl Deref for Sigrl {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Sigrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Sigrl(")?;
        for b in &self.0 {
            f.write_fmt(format_args!("{:#b}", b))?;
        }
        f.write_str(")")
    }
}

/// Represents EPID group ID.
///
/// This structure is necessary to invoke [`IasHandle::get_sigrl`] function.
///
/// [`IasHandle::get_sigrl`]: struct.IasHandle.html#method.get_sigrl
/// 
/// # Creating `GroupId`
///
/// Currently, the only way to create an instance of `GroupId`, is from `&str`
/// slice via the `std::str::FromStr::from_str` method. Note also that currently
/// prepending "0x" to the string is invalid, and will result in `Error::ParseInt(_)`
/// error.
///
/// ```
/// # use rust_sgx_util::GroupId;
/// use std::str::FromStr;
/// assert!(GroupId::from_str("01234567").is_ok());
/// assert!(GroupId::from_str("0x01234567").is_err()); // prepending "0x" is currently invalid
/// ```
/// 
/// # Accessing the underlying bytes buffer 
/// 
/// `GroupId` implements `Deref<Target=[u8]>`, therefore dereferencing it will
/// yield its inner buffer of bytes.
#[derive(Debug)]
pub struct GroupId([u8; 4]);

impl Deref for GroupId {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for GroupId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parsed = u32::from_str_radix(s, 16)?;
        Ok(GroupId(parsed.to_le_bytes()))
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GroupId({:#010x})", u32::from_le_bytes(self.0))
    }
}
