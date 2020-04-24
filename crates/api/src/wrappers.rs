use super::{c, QuoteType};
use anyhow::{anyhow, Result};
use rust_sgx_util::Quote;
use std::convert::TryFrom;
use std::ffi::CString;
use std::path::Path;

const MAX_SEALED_KEYS_SIZE: usize = 1024; // Should be plenty for ed25519 private key
const MAX_QUOTE_SIZE: usize = 2048; // Usually around 1200 bytes required
const EC_SIGNATURE: usize = 64; // EC signature should always be 64bytes long!

pub fn set_verbose(verbose: bool) {
    unsafe { c::set_verbose(verbose) }
}

pub(super) fn init_enclave<P: AsRef<Path>>(enclave_path: P) -> Result<Vec<u8>> {
    let enclave_path = path_to_c_string(enclave_path)?;
    let sealed_keys_buffer = &mut [0u8; MAX_SEALED_KEYS_SIZE];
    let ret = unsafe {
        c::pod_init_enclave(
            enclave_path.as_ptr(),
            sealed_keys_buffer.as_mut_ptr(),
            sealed_keys_buffer.len(),
        )
    };

    if ret < 0 {
        return Err(anyhow!(
            "pod_init_enclave returned non-zero exit code: {}",
            ret
        ));
    }

    let sealed_keys_size = usize::try_from(ret)?;
    Ok(Vec::from(&sealed_keys_buffer[..sealed_keys_size]))
}

pub(super) fn load_enclave<P: AsRef<Path>, B: AsRef<[u8]>>(
    enclave_path: P,
    sealed_keys: B,
) -> Result<()> {
    let enclave_path = path_to_c_string(enclave_path)?;
    let sealed_keys = sealed_keys.as_ref();
    let ret = unsafe {
        c::pod_load_enclave(
            enclave_path.as_ptr(),
            sealed_keys.as_ptr(),
            sealed_keys.len(),
        )
    };
    if ret < 0 {
        Err(anyhow!(
            "pod_load_enclave returned non-zero exit code: {}",
            ret
        ))
    } else {
        Ok(())
    }
}

pub(super) fn unload_enclave() -> Result<()> {
    let ret = unsafe { c::pod_unload_enclave() };
    if ret != 0 {
        Err(anyhow!(
            "pod_unload_enclave returned non-zero exit code: {}",
            ret
        ))
    } else {
        Ok(())
    }
}

pub(super) fn get_quote<S: AsRef<str>>(spid: S, quote_type: QuoteType) -> Result<Quote> {
    let spid = CString::new(spid.as_ref())?;
    let quote_type = match quote_type {
        QuoteType::Linkable => CString::new("l")?,
        QuoteType::Unlinkable => CString::new("u")?,
    };
    let quote_buffer = &mut [0u8; MAX_QUOTE_SIZE];
    let ret = unsafe {
        c::pod_get_quote(
            spid.as_ptr(),
            quote_type.as_ptr(),
            quote_buffer.as_mut_ptr(),
            MAX_QUOTE_SIZE,
        )
    };

    if ret < 0 {
        return Err(anyhow!(
            "pod_init_enclave returned non-zero exit code: {}",
            ret
        ));
    }
    let quote_size = usize::try_from(ret)?;
    Ok(Quote::from(&quote_buffer[..quote_size]))
}

pub(super) fn sign_buffer<B: AsRef<[u8]>>(message: B) -> Result<Vec<u8>> {
    let message = message.as_ref();
    let signature = &mut [0u8; EC_SIGNATURE];
    let ret = unsafe {
        c::pod_sign_buffer(
            message.as_ptr() as *const _,
            message.len(),
            signature.as_mut_ptr() as *mut _,
            signature.len(),
        )
    };
    if ret != 0 {
        return Err(anyhow!(
            "pod_sign_buffer returned non-zero exit code: {}",
            ret
        ));
    }
    Ok(signature.to_vec())
}

#[cfg(unix)]
fn path_to_c_string<P: AsRef<Path>>(path: P) -> Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    let s = CString::new(path.as_ref().as_os_str().as_bytes())?;
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
