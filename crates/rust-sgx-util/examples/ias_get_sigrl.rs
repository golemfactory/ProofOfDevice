use anyhow::Result;
use rust_sgx_util::{set_verbose, GroupId, IasHandle};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// IAS API key.
    api_key: String,
    /// EPID group ID (hex string).
    group_id: String,
    /// Toggle verbose mode.
    #[structopt(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    set_verbose(opt.verbose);
    let group_id = GroupId::from_str(&opt.group_id)?;
    let ias_handle = IasHandle::new(&opt.api_key, None, None)?;
    match ias_handle.get_sigrl(&group_id)? {
        Some(sigrl) => println!("SigRL for EPID group ID {}: {}.", group_id, sigrl),
        None => println!("No SigRL for EPID group ID {}.", group_id),
    };
    Ok(())
}
