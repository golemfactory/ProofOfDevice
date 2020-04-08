use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use structopt::StructOpt;
use std::io;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Address to bind to (defaults to 127.0.0.1).
    #[structopt(long)]
    address: Option<String>,
    /// Port to bind to (defaults to 8088).
    #[structopt(long)]
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    quote: rust_sgx_util::Quote,
    nonce: Option<rust_sgx_util::Nonce>,
}

async fn register(info: web::Json<Quote>) -> impl Responder {
    log::info!("Received data = {:?}", info);
    HttpResponse::Ok()
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let address = opt.address.unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = opt.port.unwrap_or(8088);
    let address_port = [address, port.to_string()].join(":");

    HttpServer::new(|| {
        App::new().service(web::resource("/register").route(web::post().to(register)))
    })
    .bind(address_port)?
    .run()
    .await
}
