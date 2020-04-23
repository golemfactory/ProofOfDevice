use pod_api::{QuoteType, PodEnclave};
use rust_sgx_util::Quote;

const SEALED_KEYS_PATH: &str = "pod_data.sealed";
const ENCLAVE_PATH: &str = "../c-api/pod-enclave/pod_enclave.signed.so";

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
struct NativeMessage {
    GetQuote,
    Quote(Quote),
    #[serde(deserialize)]
    Challenge(String),

}

fn main() {
    println!("Hello, world!");
}
