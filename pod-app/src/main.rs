mod messages;

use anyhow::{anyhow, Context, Result};
use messages::{reply, OutgoingMessage};
use nix::sys::signal::{self, SigHandler, Signal};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

const DEFAULT_PRIVATE_KEY_PATH: &str = "private_key.sealed";
const DEFAULT_ENCLAVE_PATH: &str = "../pod-enclave/pod_enclave.signed.so";

static SIGNALED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_signals(signal: libc::c_int) {
    let signal = Signal::try_from(signal).expect("valid raw signal value");
    SIGNALED.store(
        signal == Signal::SIGINT || signal == Signal::SIGTERM,
        Ordering::Relaxed,
    );
}

#[derive(Debug, Deserialize)]
struct Config {
    private_key: PathBuf,
    enclave: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let enclave = PathBuf::from(DEFAULT_ENCLAVE_PATH);
        let private_key = PathBuf::from(DEFAULT_PRIVATE_KEY_PATH);
        Self {
            enclave,
            private_key,
        }
    }
}

fn run() -> Result<()> {
    let config = config()?;
    // Install signal handler for SIGINT and SIGTERM
    let handler = SigHandler::Handler(handle_signals);
    unsafe { signal::signal(Signal::SIGTERM, handler)? };
    unsafe { signal::signal(Signal::SIGINT, handler)? };
    // Ok, first pass will load and unload enclave at each message received
    // event. However, this is definitely not optimal and we should spawn
    // it only once and hand out Arc<> to the spawned instance instead.
    // This will require some Mutexing though since PodEnclave is not safe to send
    // across thread boundaries.
    loop {
        if SIGNALED.load(Ordering::Relaxed) {
            break;
        }

        let mut msg_len = [0u8; 4];

        if let Err(err) = io::stdin().read_exact(&mut msg_len) {
            match err.kind() {
                io::ErrorKind::BrokenPipe | io::ErrorKind::UnexpectedEof => {
                    // Unless we received a SIGTERM and the other closed the pipe
                    // on purpose, we need to throw an error!
                    if !SIGNALED.load(Ordering::Relaxed) {
                        return Err(anyhow!("Unexpected EOF or a broken pipe!"));
                    }
                }
                _ => {
                    return Err(anyhow!(
                        "failed to read message len from stdin (first 4 bytes): {}",
                        err
                    ))
                }
            }
        }

        let msg_len = u32::from_ne_bytes(msg_len);
        let msg_len = usize::try_from(msg_len).expect("u32 should fit into usize");

        let mut msg = Vec::new();
        msg.resize(msg_len as usize, 0);
        io::stdin()
            .read_exact(&mut msg)
            .context("failed to read message from stdin")?;

        let reply = match reply(msg, &config) {
            Ok(reply) => reply,
            Err(err) => serde_json::to_vec(&OutgoingMessage::error(err))
                .context("converting error to JSON failed")?,
        };
        let reply_len: u32 = reply
            .len()
            .try_into()
            .context("reply len overflew 32bit register")?;
        io::stdout()
            .write_all(&reply_len.to_ne_bytes())
            .context("failed to write reply length to stdout (first 4 bytes)")?;
        io::stdout()
            .write_all(&reply)
            .context("failed to write reply to stdout")?;
        io::stdout().flush()?;
    }

    Ok(())
}

fn config() -> Result<Config> {
    // Firstly, check in xdg config folder.
    let dirs = xdg::BaseDirectories::with_prefix("pod-app")
        .context("couldn't create xdg base dir instance")?;
    let config = match dirs.find_data_file("pod_enclave.signed.so") {
        Some(enclave) => {
            let private_key = dirs.get_data_home().join("private_key.sealed");
            Config {
                private_key,
                enclave,
            }
        }
        None => Config::default(),
    };
    Ok(config)
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Unexpected error occurred: {}", err);
        process::exit(1);
    }
}
