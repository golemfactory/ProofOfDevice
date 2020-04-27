use anyhow::{anyhow, Result};
use nix::sys::signal::{self, SigHandler, Signal};
use pod_api::{PodEnclave, QuoteType};
use rust_sgx_util::Quote;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Write};
use std::process;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};

static SIGNALED: AtomicBool = AtomicBool::new(false);

const SEALED_KEYS_PATH: &str = "pod_data.sealed";
const ENCLAVE_PATH: &str = "../pod-enclave/pod_enclave.signed.so";

extern "C" fn handle_signals(signal: libc::c_int) {
    let signal = Signal::try_from(signal).expect("valid raw signal value");
    SIGNALED.store(
        signal == Signal::SIGINT || signal == Signal::SIGTERM,
        Ordering::Relaxed,
    );
}

#[derive(Debug, Deserialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
enum IncomingMessage {
    GetQuote {
        spid: String,
    },
    SignChallenge {
        #[serde(with = "base_64")]
        challenge: Vec<u8>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
#[allow(dead_code)]
enum OutgoingMessage {
    GetQuote {
        quote: Quote,
    },

    SignChallenge {
        #[serde(with = "base_64")]
        signed: Vec<u8>,
    },
    Error {
        description: String,
    },
}

impl OutgoingMessage {
    fn get_quote(quote: Quote) -> Self {
        Self::GetQuote { quote }
    }

    fn sign_challenge<B: AsRef<[u8]>>(signed: B) -> Self {
        Self::SignChallenge {
            signed: signed.as_ref().to_vec(),
        }
    }

    fn error<S: ToString>(desc: S) -> Self {
        Self::Error {
            description: desc.to_string(),
        }
    }
}

mod base_64 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(blob: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::encode(blob))
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        base64::decode(&string).map_err(serde::de::Error::custom)
    }
}

fn reply<B: AsRef<[u8]>>(msg: B) -> Result<Vec<u8>> {
    let msg: IncomingMessage = serde_json::from_slice(msg.as_ref())?;
    let reply = match msg {
        IncomingMessage::GetQuote { spid } => {
            let pod_enclave = PodEnclave::new(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
            let quote = pod_enclave.get_quote(spid, QuoteType::Unlinkable)?;
            let reply = OutgoingMessage::get_quote(quote);
            let serialized = serde_json::to_vec(&reply)?;
            serialized
        }
        IncomingMessage::SignChallenge { challenge } => {
            let pod_enclave = PodEnclave::new(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
            let signature = pod_enclave.sign(challenge)?;
            let reply = OutgoingMessage::sign_challenge(signature);
            let serialized = serde_json::to_vec(&reply)?;
            serialized
        }
    };
    Ok(reply)
}

fn run() -> Result<()> {
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
                _ => return Err(err.into()),
            }
        }

        let msg_len = u32::from_ne_bytes(msg_len);
        let msg_len = usize::try_from(msg_len).expect("u32 should fit into usize");

        let mut msg = Vec::new();
        msg.resize(msg_len as usize, 0);
        io::stdin().read_exact(&mut msg)?;

        let reply = match reply(msg) {
            Ok(reply) => reply,
            Err(err) => serde_json::to_vec(&OutgoingMessage::error(err))?,
        };
        let reply_len: u32 = reply.len().try_into()?;
        io::stdout().write_all(&reply_len.to_ne_bytes())?;
        io::stdout().write_all(&reply)?;
        io::stdout().flush()?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Unexpected error occurred: {}", err);
        process::exit(1);
    }
}
