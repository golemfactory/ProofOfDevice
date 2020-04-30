use super::Config;
use anyhow::Result;
use pod_api::{PodEnclave, QuoteType};
use rust_sgx_util::Quote;
use serde::{Deserialize, Serialize};

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
pub(super) enum OutgoingMessage {
    GetQuote {
        quote: Quote,
    },

    SignChallenge {
        #[serde(with = "base_64")]
        signed: Vec<u8>,
    },
}

impl OutgoingMessage {
    pub fn get_quote(quote: Quote) -> Self {
        Self::GetQuote { quote }
    }

    pub fn sign_challenge<B: AsRef<[u8]>>(signed: B) -> Self {
        Self::SignChallenge {
            signed: signed.as_ref().to_vec(),
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

pub(super) fn reply<B: AsRef<[u8]>>(msg: B, config: &Config) -> Result<Vec<u8>> {
    let msg: IncomingMessage = serde_json::from_slice(msg.as_ref())?;
    log::debug!("Received message content: {:?}", msg);

    let reply = match msg {
        IncomingMessage::GetQuote { spid } => {
            let pod_enclave = PodEnclave::new(&config.enclave, &config.private_key)?;
            let quote = pod_enclave.get_quote(spid, QuoteType::Unlinkable)?;
            let reply = OutgoingMessage::get_quote(quote);
            let serialized = serde_json::to_vec(&reply)?;
            serialized
        }
        IncomingMessage::SignChallenge { challenge } => {
            let pod_enclave = PodEnclave::new(&config.enclave, &config.private_key)?;
            let signature = pod_enclave.sign(challenge)?;
            let reply = OutgoingMessage::sign_challenge(signature);
            let serialized = serde_json::to_vec(&reply)?;
            serialized
        }
    };

    log::debug!("Reply content: {:?}", reply);
    Ok(reply)
}
