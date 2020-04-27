use actix_web::client::Client;
use actix_web::HttpMessage;
use anyhow::anyhow;
use pod_api::{PodEnclave, QuoteType};
use rust_sgx_util::{Nonce, Quote};
use serde::Serialize;
use structopt::StructOpt;

const SEALED_KEYS_PATH: &str = "pod_data.sealed";
const ENCLAVE_PATH: &str = "../c-api/pod-enclave/pod_enclave.signed.so";

#[derive(Debug, StructOpt)]
struct Opt {
    /// Server address to connect to (defaults to 127.0.0.1).
    #[structopt(long)]
    address: Option<String>,
    /// Server port to connect to (defaults to 8080).
    #[structopt(long)]
    port: Option<u16>,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Register with the service.
    Register {
        /// Your username.
        login: String,
        /// Service Provider's ID (SPID) as given by the SP.
        spid: String,
    },
    /// Authenticate with the service.
    Authenticate {
        /// Your username.
        login: String,
    },
}

#[derive(Serialize)]
struct RegisterInfo {
    login: String,
    quote: Quote,
    nonce: Option<Nonce>,
}

#[derive(Serialize)]
struct ChallengeResponse {
    login: String,
    response: String,
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Turn on enclave logging
    pod_api::set_verbose(true);

    let address = opt.address.unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = opt.port.unwrap_or(8080);
    let base_uri = format!("http://{}:{}", address, port);
    let client = Client::default();
    let pod_enclave = PodEnclave::new(ENCLAVE_PATH, SEALED_KEYS_PATH)?;

    match opt.cmd {
        Command::Register { login, spid } => {
            // Get the quote
            let quote = pod_enclave.get_quote(&spid, QuoteType::Unlinkable)?;

            println!("POST /register");
            let mut response = client
                .post(format!("{}/register", base_uri))
                .header("User-Agent", "TestClient")
                .send_json(&RegisterInfo {
                    login: login.clone(),
                    quote,
                    nonce: None,
                })
                .await
                .map_err(|err| anyhow!("{:?}", err))?;
            println!("    | status_code: {}", response.status());
            let body = response.body().await.map_err(|err| anyhow!("{:?}", err))?;
            let content_length = body.len();
            println!("    | content-length: {}", content_length);

            if content_length > 0 {
                let json: serde_json::Value = serde_json::from_slice(&body)?;
                println!("    | body: {}", json);
            }
        }
        Command::Authenticate { login } => {
            println!("GET /auth");
            let mut response = client
                .get(format!("{}/auth", base_uri))
                .header("User-Agent", "TestClient")
                .send()
                .await
                .map_err(|err| anyhow!("{:?}", err))?;
            let status_code = response.status();
            println!("    | status_code: {}", status_code);
            let body = response.body().await.map_err(|err| anyhow!("{:?}", err))?;
            let content_length = body.len();
            println!("    | content-length: {}", content_length);
            let cookies = response.cookies()?.clone();
            for cookie in &cookies {
                println!("    | cookie: {}", cookie);
            }
            if status_code != 200 {
                return Err(anyhow!("Expected GET /auth to return 200"));
            }
            let json: serde_json::Value = serde_json::from_slice(&body)?;
            println!("    | body: {}", json);

            // Process challenge
            let challenge = json["challenge"]
                .as_str()
                .ok_or(anyhow!("invalid String for challenge"))?;
            let challenge = base64::decode(challenge)?;
            let response = pod_enclave.sign(&challenge)?;
            let response = base64::encode(&response[..]);

            println!("\nPOST /auth");
            let mut builder = client
                .post(format!("{}/auth", base_uri))
                .header("User-Agent", "TestClient");
            for cookie in cookies {
                builder = builder.cookie(cookie);
            }
            let mut response = builder
                .send_json(&ChallengeResponse { login, response })
                .await
                .map_err(|err| anyhow!("{:?}", err))?;
            println!("    | status_code: {}", response.status());
            let body = response.body().await.map_err(|err| anyhow!("{:?}", err))?;
            let content_length = body.len();
            println!("    | content-length: {}", content_length);
            let cookies = response.cookies()?.clone();
            for cookie in &cookies {
                println!("    | cookie: {}", cookie);
            }

            if content_length > 0 {
                let json: serde_json::Value = serde_json::from_slice(&body)?;
                println!("    | body: {}", json);
            }

            println!("\n GET /");
            let mut builder = client.get(&base_uri).header("User-Agent", "TestClient");
            for cookie in cookies {
                builder = builder.cookie(cookie);
            }
            let mut response = builder.send().await.map_err(|err| anyhow!("{:?}", err))?;
            println!("    | status_code: {}", response.status());
            let body = response.body().await.map_err(|err| anyhow!("{:?}", err))?;
            let content_length = body.len();
            println!("    | content-length: {}", content_length);
            let json: serde_json::Value = serde_json::from_slice(&body)?;
            println!("    | body: {}", json);
        }
    }

    Ok(())
}
