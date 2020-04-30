mod messages;

use anyhow::{anyhow, Context, Result};
use messages::reply;
use nix::sys::signal::{self, SigHandler, Signal};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, process};

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
    log_output: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let enclave = PathBuf::from(DEFAULT_ENCLAVE_PATH);
        let private_key = PathBuf::from(DEFAULT_PRIVATE_KEY_PATH);
        let log_output = PathBuf::from(log_filename());
        Self {
            enclave,
            private_key,
            log_output,
        }
    }
}

enum Interrupt {
    Break,
    Continue,
}

enum Stdio {
    Stdin,
    Stdout,
}

fn stdio_with_interrupt(handle: Stdio, buffer: &mut [u8]) -> Result<Interrupt> {
    let res = match handle {
        Stdio::Stdin => io::stdin().read_exact(buffer),
        Stdio::Stdout => io::stdout().write_all(buffer),
    };
    match res {
        Ok(_) => Ok(Interrupt::Continue),
        Err(err) => {
            match err.kind() {
                io::ErrorKind::BrokenPipe | io::ErrorKind::UnexpectedEof => {
                    // Unless we received a SIGTERM and the other closed the pipe
                    // on purpose, we need to throw an error!
                    if !SIGNALED.load(Ordering::Relaxed) {
                        Err(anyhow!("Unexpected EOF or a broken pipe!"))
                    } else {
                        log::debug!("Received termination signal, exiting...");
                        Ok(Interrupt::Break)
                    }
                }
                _ => Err(anyhow!(
                    "failed to read message len from stdin (first 4 bytes): {}",
                    err
                )),
            }
        }
    }
}

fn run(config: Config) -> Result<()> {
    // Install signal handler for SIGINT and SIGTERM
    let handler = SigHandler::Handler(handle_signals);
    unsafe { signal::signal(Signal::SIGTERM, handler)? };
    unsafe { signal::signal(Signal::SIGINT, handler)? };
    log::debug!("Registered SIGTERM and SIGINT signal handlers");
    // Ok, first pass will load and unload enclave at each message received
    // event. However, this is definitely not optimal and we should spawn
    // it only once and hand out Arc<> to the spawned instance instead.
    // This will require some Mutexing though since PodEnclave is not safe to send
    // across thread boundaries.
    loop {
        if SIGNALED.load(Ordering::Relaxed) {
            log::debug!("Received termination signal, exiting...");
            break;
        }

        let mut msg_len = [0u8; 4];
        match stdio_with_interrupt(Stdio::Stdin, &mut msg_len) {
            Err(err) => return Err(err),
            Ok(Interrupt::Break) => break,
            Ok(Interrupt::Continue) => {},
        };
        let msg_len = u32::from_ne_bytes(msg_len);
        let msg_len = usize::try_from(msg_len).expect("u32 should fit into usize");

        if msg_len == 0 {
            if SIGNALED.load(Ordering::Relaxed) {
                log::debug!("Received termination signal, exiting...");
                break;
            }

            // TODO is this correct behaviour? Should we actually simply carry on listening?
            continue;
        }

        log::debug!("Received message len (first 4 bytes): {}", msg_len);

        let mut msg = Vec::new();
        msg.resize(msg_len as usize, 0);
        match stdio_with_interrupt(Stdio::Stdin, &mut msg) {
            Err(err) => return Err(err),
            Ok(Interrupt::Break) => break,
            Ok(Interrupt::Continue) => {},
        };

        log::debug!("Message received");

        let mut reply = reply(msg, &config)?;
        let reply_len: u32 = reply
            .len()
            .try_into()
            .context("reply len overflew 32bit register")?;
        match stdio_with_interrupt(Stdio::Stdout, &mut reply_len.to_ne_bytes()) {
            Err(err) => return Err(err),
            Ok(Interrupt::Break) => break,
            Ok(Interrupt::Continue) => {},
        };

        log::debug!("Sent reply len: {}", reply_len);

        match stdio_with_interrupt(Stdio::Stdout, &mut reply) {
            Err(err) => return Err(err),
            Ok(Interrupt::Break) => break,
            Ok(Interrupt::Continue) => {},
        };
        io::stdout().flush()?;

        log::debug!("Reply sent");
    }

    Ok(())
}

fn config() -> Config {
    // Firstly, check in xdg config folder.
    let dirs = xdg::BaseDirectories::with_prefix("pod-app")
        .expect("couldn't create xdg base dir instance");

    match dirs.find_data_file("pod_enclave.signed.so") {
        Some(enclave) => {
            let data_home = dirs.get_data_home();
            let private_key = data_home.join("private_key.sealed");
            let log_output = data_home.join(log_filename());

            Config {
                private_key,
                enclave,
                log_output,
            }
        }
        None => Config::default(),
    }
}

fn log_filename() -> String {
    // TODO add some stronger uniqueness to log names
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("could get current system time");
    format!("pod-app_{}.log", now.as_secs_f64())
}

fn main() {
    let config = config();
    let log_output =
        fs::File::create(&config.log_output).expect("could initialize file for logger output");

    simplelog::WriteLogger::init(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        log_output,
    )
    .expect("could initialize logger");

    log::debug!("Application started...");

    if let Err(err) = run(config) {
        log::error!("Unexpected error occurred: {}", err);
        process::exit(1);
    }

    log::debug!("Application successfully stopped...");
}
