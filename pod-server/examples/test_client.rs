use actix_web::client::Client;
use actix_web::http::header::CONTENT_LENGTH;
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
    /// Server port to connect to (defaults to 8088).
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
    let port = opt.port.unwrap_or(8088);
    let base_uri = format!("http://{}:{}", address, port);
    let client = Client::default();

    match opt.cmd {
        Command::Register {
            login,
            quote_path,
            nonce,
        } => {
            let quote = Quote::from(fs::read(&quote_path)?);
            let nonce = nonce.as_ref().map(|x| Nonce::from(x.as_bytes()));

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
        Command::Authenticate { .. } => {
            // let cookies = response.cookies()?.clone();
            // let cookie = cookies
            //     .into_iter()
            //     .find(|c| c.name() == "actix-session")
            //     .ok_or(anyhow!("cookie actix-session not found"))?;
            // println!("    | cookie: {}", cookie);
            unimplemented!()
        }
    }

    Ok(())
}
