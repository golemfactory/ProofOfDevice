use anyhow::Result;
use pod_api::{PodEnclave, QuoteType};
use rust_sgx_util::Quote;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::str;

const SEALED_KEYS_PATH: &str = "pod_data.sealed";
const ENCLAVE_PATH: &str = "../c-api/pod-enclave/pod_enclave.signed.so";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
enum NativeMessage {
    GetQuote,
    Quote(Quote),
    #[serde(with = "base_64")]
    Challenge(Vec<u8>),
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

fn main() -> Result<()> {
    // let pod_enclave = PodEnclave::new(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
    loop {
        // let mut msg_len = [0u8; 4];
        // io::stdin().read_exact(&mut msg_len)?;
        // let msg_len = u32::from_ne_bytes(msg_len);
        // let msg_len = usize::try_from(msg_len).expect("u32 should fit into usize");

        let msg_len = 19;
        let mut msg = Vec::new();
        msg.resize(msg_len as usize, 0);
        io::stdin().read_exact(&mut msg)?;

        let msg: NativeMessage = serde_json::from_slice(&msg)?;
        println!("Received msg: {:?}", msg);
    }
    // TODO add signal handlers as we need to cleanup the enclave after it's been loaded!
}
