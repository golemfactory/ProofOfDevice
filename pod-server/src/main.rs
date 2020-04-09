use actix_web::{web, App, HttpServer, Responder};
use anyhow::anyhow;
use rust_sgx_util::{IasHandle, Nonce, Quote};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::{env, fs};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "pod_server", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    /// Path to server config TOML file.
    #[structopt(parse(from_os_str))]
    config_path: PathBuf,
    /// Set verbose mode on/off.
    #[structopt(short, long)]
    verbose: bool,
}

#[derive(Deserialize)]
struct Config {
    api_key: String,
    server: Option<ServerConfig>,
}

#[derive(Deserialize)]
struct ServerConfig {
    address: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct QuoteWithNonce {
    quote: Quote,
    nonce: Option<Nonce>,
}

#[derive(Serialize)]
enum RegisterResponse {
    Registered,
    Error(String),
}

async fn register(info: web::Json<QuoteWithNonce>, handle: web::Data<IasHandle>) -> impl Responder {
    log::info!("Received register request");
    log::debug!("Received data = {:?}", info);
    // Verify the provided data with IAS.
    match handle.verify_quote(&info.quote, info.nonce.as_ref(), None, None, None, None) {
        Ok(()) => web::Json(RegisterResponse::Registered),
        // TODO Add proper mapping between error and response.
        Err(err) => web::Json(RegisterResponse::Error(err.to_string())),
    }
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    // Enable info logging by default.
    env::set_var("RUST_LOG", "info");
    if opt.verbose {
        env::set_var("RUST_LOG", "info,rust_sgx_util=debug,pod_server=debug");
        rust_sgx_util::set_verbose(true);
    }
    pretty_env_logger::init();
    // Read config file
    let config_file = fs::read(&opt.config_path)?;
    let config: Config = toml::from_slice(&config_file)?;
    let (address, port) = match &config.server {
        Some(server_config) => (server_config.address.clone(), server_config.port),
        None => ("127.0.0.1".to_string(), 8088),
    };
    let address_port = [address, port.to_string()].join(":");
    // Set POD_SERVER_API_KEY env variable
    env::set_var("POD_SERVER_API_KEY", config.api_key);

    HttpServer::new(move || {
        App::new()
            .data_factory(
                || -> Pin<Box<dyn Future<Output = anyhow::Result<IasHandle>>>> {
                    Box::pin(async move {
                        let api_key = env::var("POD_SERVER_API_KEY")?;
                        let handle = IasHandle::new(&api_key, None, None)?;
                        Ok(handle)
                    })
                },
            )
            .service(web::resource("/register").route(web::post().to(register)))
    })
    .bind(address_port)?
    .run()
    .await
    .map_err(|err| anyhow!("HttpServer errored out with {:?}", err))?;

    Ok(())
}
