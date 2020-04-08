use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rust_sgx_util::{IasHandle, Nonce, Quote};
use serde::Deserialize;
use std::io;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// API key to use for IAS services.
    api_key: String,
    /// Address to bind to (defaults to 127.0.0.1).
    #[structopt(long)]
    address: Option<String>,
    /// Port to bind to (defaults to 8088).
    #[structopt(long)]
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct QuoteWithNonce {
    quote: Quote,
    nonce: Option<Nonce>,
}

async fn register(info: web::Json<QuoteWithNonce>) -> impl Responder {
    log::info!("Received data = {:?}", info);
    // Verify the provided data with IAS.
    HttpResponse::Ok()
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let address = opt.address.unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = opt.port.unwrap_or(8088);
    let address_port = [address, port.to_string()].join(":");
    // Initialize handle to IAS services.
    let ias_handle = IasHandle::new(&opt.api_key, None, None).map_err(|err| {
        log::error!("Initialization of IasHandle failed with error: {}", err);
        io::Error::new(io::ErrorKind::Other, err)
    })?;
    let ias_handle = web::Data::new(ias_handle);

    HttpServer::new(move || {
        App::new()
            .app_data(ias_handle.clone())
            .service(web::resource("/register").route(web::post().to(register)))
    })
    .bind(address_port)?
    .run()
    .await
}
