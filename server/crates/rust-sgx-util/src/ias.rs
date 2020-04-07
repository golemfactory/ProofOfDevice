use crate::{c, Error, Result};
use std::ffi::CString;
use std::ptr::{self, NonNull};
use std::str::FromStr;
use std::{fmt, slice, u32};

const IAS_VERIFY_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/report";
const IAS_SIGRL_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/sigrl";

pub struct IasHandle {
    api_key: CString,
    verify_url: CString,
    sigrl_url: CString,
    context: NonNull<c::IasContext>,
}

impl IasHandle {
    // TODO API key should probably have its own type that does
    // at the very least some length validation
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
            api_key,
            verify_url,
            sigrl_url,
            context,
        })
    }

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
#[derive(Debug)]
pub struct GroupId {
    inner: [u8; 4],
}

impl GroupId {
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
