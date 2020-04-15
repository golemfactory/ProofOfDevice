mod entrypoints;
mod error;
mod models;
mod schema;

#[macro_use]
extern crate diesel;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{middleware, web, App, HttpServer};
use anyhow::anyhow;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use dotenv::dotenv;
use serde::Deserialize;
use std::path::PathBuf;
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
struct ServerConfig {
    api_key: String,
    cookie_key: String,
    bind: Option<BindAddress>,
}

#[derive(Deserialize)]
struct BindAddress {
    address: String,
    port: u16,
}

pub struct AppData {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    dotenv()?;
    // Enable info logging by default.
    env::set_var("RUST_LOG", "info");
    if opt.verbose {
        env::set_var("RUST_LOG", "info,rust_sgx_util=debug,pod_server=debug");
        rust_sgx_util::set_verbose(true);
    }
    pretty_env_logger::init();
    // Read config file
    let config_file = fs::read(&opt.config_path)?;
    let config: ServerConfig = toml::from_slice(&config_file)?;
    let (address, port) = match &config.bind {
        Some(server_config) => (server_config.address.clone(), server_config.port),
        None => ("127.0.0.1".to_string(), 8088),
    };
    let address_port = [address, port.to_string()].join(":");
    // Set POD_SERVER_API_KEY env variable
    env::set_var("POD_SERVER_API_KEY", config.api_key);

    let db_url = env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().build(manager)?;
    let data = web::Data::new(AppData { pool });
    // Cookie config
    let cookie_key = config.cookie_key.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(
                CookieSession::signed(cookie_key.as_bytes())
                    .name("session")
                    .secure(false),
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(cookie_key.as_bytes())
                    .name("auth")
                    .secure(false),
            ))
            .wrap(middleware::Logger::default())
            .app_data(data.clone())
            .route("/", web::get().to(entrypoints::index))
            .route("/register", web::post().to(entrypoints::register))
            .service(
                web::resource("/auth")
                    .route(web::get().to(entrypoints::get_auth))
                    .route(web::post().to(entrypoints::auth)),
            )
    })
    .bind(address_port)?
    .run()
    .await
    .map_err(|err| anyhow!("HttpServer errored out with {:?}", err))?;

    Ok(())
}
