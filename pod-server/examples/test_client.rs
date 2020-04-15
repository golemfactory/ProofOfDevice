use actix_web::client::Client;
use anyhow::{anyhow, Result};
use rust_sgx_util::{Nonce, Quote};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to quote to verify.
    #[structopt(parse(from_os_str))]
    quote_path: PathBuf,
    /// Nonce to use.
    #[structopt(long)]
    nonce: Option<String>,
    /// Server address to connect to (defaults to 127.0.0.1).
    #[structopt(long)]
    address: Option<String>,
    /// Server port to connect to (defaults to 8088).
    #[structopt(long)]
    port: Option<u16>,
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
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let login = "test-user-1".to_string();
    let quote = Quote::from(fs::read(&opt.quote_path)?);
    let nonce = opt.nonce.as_ref().map(|x| Nonce::from(x.as_bytes()));
    let address = opt.address.unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = opt.port.unwrap_or(8088);
    let client = Client::default();

    let uri = format!("http://{}:{}/register", address, port);
    let mut response = client
        .post(uri)
        .header("User-Agent", "TestClient")
        .send_json(&RegisterInfo {
            login: login.clone(),
            quote,
            nonce,
        })
        .await
        .map_err(|err| anyhow!("ClientRequest errored out with {:?}", err))?;
    println!("Response: {:?}", response);
    let body = response
        .body()
        .await
        .map_err(|err| anyhow!("ClientResponse errored out with {:?}", err))?;
    let body: serde_json::Value = serde_json::from_slice(&body)?;
    println!("Body: {:?}", body);

    let uri = format!("http://{}:{}/auth", address, port);
    let mut response = client
        .get(uri)
        .header("User-Agent", "TestClient")
        .send()
        .await
        .map_err(|err| anyhow!("ClientRequest errored out with {:?}", err))?;
    println!("Response: {:?}", response);
    let body = response
        .body()
        .await
        .map_err(|err| anyhow!("ClientResponse errored out with {:?}", err))?;
    let body: serde_json::Value = serde_json::from_slice(&body)?;
    println!("Body: {:?}", body);

    let uri = format!("http://{}:{}/auth", address, port);
    let mut response = client
        .post(uri)
        .header("User-Agent", "TestClient")
        .send_json(&ChallengeResponse { login, response: "0123456789abcdef".to_string() })
        .await
        .map_err(|err| anyhow!("ClientRequest errored out with {:?}", err))?;
    println!("Response: {:?}", response);
    let body = response
        .body()
        .await
        .map_err(|err| anyhow!("ClientResponse errored out with {:?}", err))?;
    let body: serde_json::Value = serde_json::from_slice(&body)?;
    println!("Body: {:?}", body);

    let uri = format!("http://{}:{}", address, port);
    let mut response = client
        .get(uri)
        .header("User-Agent", "TestClient")
        .send()
        .await
        .map_err(|err| anyhow!("ClientRequest errored out with {:?}", err))?;
    println!("Response: {:?}", response);
    let body = response
        .body()
        .await
        .map_err(|err| anyhow!("ClientResponse errored out with {:?}", err))?;
    let body: serde_json::Value = serde_json::from_slice(&body)?;
    println!("Body: {:?}", body);

    Ok(())
}
