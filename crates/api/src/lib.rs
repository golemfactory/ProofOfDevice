mod c;
mod wrappers;

use anyhow::Result;
use rust_sgx_util::Quote;
use std::cell::RefCell;
use std::fs;
use std::path::Path;

pub use wrappers::set_verbose;

pub enum QuoteType {
    Linkable,
    Unlinkable,
}

pub struct PodEnclave {
    quote: RefCell<Option<Quote>>,
}

impl PodEnclave {
    pub fn new<P: AsRef<Path>>(enclave_path: P, sealed_keys_path: P) -> Result<Self> {
        if !sealed_keys_path.as_ref().exists() {
            // Initialize enclave for the first time
            let sealed_keys = wrappers::init_enclave(enclave_path)?;
            // Save state to file
            fs::write(sealed_keys_path.as_ref(), &sealed_keys)?;
            sealed_keys
        } else {
            // Load state from file
            let sealed_keys = fs::read(sealed_keys_path.as_ref())?;
            wrappers::load_enclave(enclave_path, &sealed_keys)?;
            sealed_keys
        };
        Ok(Self {
            quote: RefCell::new(None),
        })
    }

    pub fn get_quote<S: AsRef<str>>(&self, spid: S, quote_type: QuoteType) -> Result<Quote> {
        let quote = match self.quote.borrow_mut().take() {
            Some(quote) => quote,
            None => {
                let quote = wrappers::get_quote(spid.as_ref(), quote_type)?;
                quote
            }
        };
        let ret = quote.clone();
        *self.quote.borrow_mut() = Some(quote);
        Ok(ret)
    }

    pub fn sign<B: AsRef<[u8]>>(&self, message: B) -> Result<Vec<u8>> {
        let signature = wrappers::sign_buffer(message)?;
        Ok(signature)
    }
}

impl Drop for PodEnclave {
    fn drop(&mut self) {
        wrappers::unload_enclave().expect("unloading enclave should succeed")
    }
}
