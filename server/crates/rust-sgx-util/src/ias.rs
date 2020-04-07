use crate::{c, Error, Result};
use std::ptr::{self, NonNull};
use std::slice;

const IAS_VERIFY_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/report";
const IAS_SIGRL_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/sigrl";

pub struct IasHandle {
    context: NonNull<c::IasContext>,
}

impl IasHandle {
    // TODO API key should probably have its own type that does
    // at the very least some length validation
    pub fn new(api_key: &str, verify_url: Option<&str>, sigrl_url: Option<&str>) -> Result<Self> {
        let verify_url = verify_url.unwrap_or(IAS_VERIFY_URL);
        let sigrl_url = sigrl_url.unwrap_or(IAS_SIGRL_URL);
        let raw_context = unsafe {
            c::ias_init(
                api_key.as_ptr() as _,
                verify_url.as_ptr() as _,
                sigrl_url.as_ptr() as _,
            )
        };
        let context = NonNull::new(raw_context).ok_or(Error::IasInitNullPtr)?;
        Ok(Self { context })
    }

    pub fn get_sigrl(&self, group_id: &GroupId) -> Result<Option<Sigrl>> {
        let mut size: usize = 0;
        let mut raw = ptr::null_mut();
        let ret = unsafe {
            c::ias_get_sigrl(
                self.context.as_ptr(),
                group_id.0.as_ptr(),
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

#[derive(Debug)]
pub struct GroupId([u8; 4]);
