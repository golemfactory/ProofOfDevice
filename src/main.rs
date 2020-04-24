use anyhow::Result;
use futures::stream;
use pod_api::{PodEnclave, QuoteType};
use rust_sgx_util::Quote;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::{env, str};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::signal;

const SEALED_KEYS_PATH: &str = "pod_data.sealed";
const ENCLAVE_PATH: &str = "crates/c-api/pod-enclave/pod_enclave.signed.so";

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "message", rename_all = "snake_case")]
enum IncomingMessage {
    GetQuote(String),
    #[serde(with = "base_64")]
    Challenge(Vec<u8>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message", rename_all = "snake_case")]
enum OutgoingMessage {
    Quote(Quote),
    #[serde(with = "base_64")]
    Response(Vec<u8>),
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

#[tokio::main]
async fn main() -> Result<()> {
    // Enable info logging by default.
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    pod_api::set_verbose(true);

    // TODO
    // Ok, first pass will load and unload enclave at each message received
    // event. However, this is definitely not optimal and we should spawn
    // it only once and hand out Arc<> to the spawned instance instead.
    // This will require some Mutexing though since PodEnclave is stateful
    // (well, it's not itself, but the c-api that is uses underneath is).

    let mut msg_len = [0u8; 4];
    io::stdin().read_exact(&mut msg_len).await?;
    let msg_len = u32::from_ne_bytes(msg_len);
    let msg_len = usize::try_from(msg_len).expect("u32 should fit into usize");
    log::info!("msg_len = {}", msg_len);

    let mut msg = Vec::new();
    msg.resize(msg_len as usize, 0);
    io::stdin().read_exact(&mut msg).await?;

    let msg: IncomingMessage = serde_json::from_slice(&msg)?;
    log::info!("msg = {:?}", msg);

    let reply = match msg {
        IncomingMessage::GetQuote(spid) => {
            let pod_enclave = PodEnclave::new(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
            let quote = pod_enclave.get_quote(spid, QuoteType::Unlinkable)?;
            let reply = OutgoingMessage::Quote(quote);
            log::info!("reply = {}", serde_json::to_string(&reply)?);
            let serialized = serde_json::to_vec(&reply)?;
            serialized
        }
        IncomingMessage::Challenge(challenge) => {
            let pod_enclave = PodEnclave::new(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
            let signature = pod_enclave.sign(challenge)?;
            let reply = OutgoingMessage::Response(signature);
            log::info!("reply = {}", serde_json::to_string(&reply)?);
            let serialized = serde_json::to_vec(&reply)?;
            log::info!("reply = {:?}", serde_json::to_vec(&reply)?);
            serialized
        }
    };
    let reply_len: u32 = reply.len().try_into()?;
    log::info!("reply_len = {}", reply_len);
    io::stdout().write_all(&reply_len.to_ne_bytes()).await?;
    io::stdout().write_all(&reply).await?;

    Ok(())
}
