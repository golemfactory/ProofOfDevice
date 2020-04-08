use anyhow::Result;
use rust_sgx_util::{set_verbose, IasHandle, Nonce, Quote};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// IAS API key.
    api_key: String,
    /// Path to quote to verify.
    #[structopt(parse(from_os_str))]
    quote_path: PathBuf,
    /// Nonce to use.
    #[structopt(long)]
    nonce: Option<String>,
    /// Toggle verbose mode.
    #[structopt(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    set_verbose(opt.verbose);
    let ias_handle = IasHandle::new(&opt.api_key, None, None)?;
    let quote = Quote::from(fs::read(&opt.quote_path)?);
    let nonce = opt.nonce.as_ref().map(|x| Nonce::from(x.as_bytes()));
    match ias_handle.verify_quote(&quote, nonce.as_ref(), None, None, None, None) {
        Err(_) => println!("Verification of quote unsuccessful!"),
        Ok(()) => {}
    };
    Ok(())
}
