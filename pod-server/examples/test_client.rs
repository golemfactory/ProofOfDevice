use actix_web::client::Client;
use actix_web::HttpMessage;
use anyhow::anyhow;
use rust_sgx_util::{Nonce, Quote};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

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
        /// Path to quote to verify.
        #[structopt(parse(from_os_str))]
        quote_path: PathBuf,
        /// Nonce to use.
        #[structopt(long)]
        nonce: Option<String>,
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

    let address = opt.address.unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = opt.port.unwrap_or(8080);
    let base_uri = format!("http://{}:{}", address, port);
    let client = Client::default();

    // Try loading keys from test_key
    let keypair = match fs::read("test_key") {
        Ok(bytes) => {
            let keypair = ed25519_dalek::Keypair::from_bytes(&bytes)?;
            keypair
        }
        Err(_) => {
            let mut csprng = rand::rngs::OsRng;
            let keypair = ed25519_dalek::Keypair::generate(&mut csprng);
            fs::write("test_key", &keypair.to_bytes()[..])?;
            keypair
        }
    };

    match opt.cmd {
        Command::Register {
            login,
            quote_path,
            nonce,
        } => {
            let quote = Quote::from(fs::read(&quote_path)?);
            let nonce = nonce.as_ref().map(|x| Nonce::from(x.as_bytes()));

            // Inject an actual ED25519 key into the quote
            let mut quote = quote.to_vec();
            let (start, stop) = (48 + 320, 48 + 320 + 32);
            let pub_key = keypair.public.as_bytes();
            &mut quote[start..stop].copy_from_slice(pub_key);
            let quote = Quote::from(quote);

            println!("POST /register");
            let mut response = client
                .post(format!("{}/register", base_uri))
                .header("User-Agent", "TestClient")
                .send_json(&RegisterInfo {
                    login: login.clone(),
                    quote,
                    nonce,
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
            let response = keypair.sign(&challenge);
            let response = base64::encode(&response.to_bytes()[..]);

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
