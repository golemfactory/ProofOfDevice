use actix_web::client::Client;
use actix_web::HttpMessage;
use anyhow::anyhow;
use rust_sgx_util::{Nonce, Quote};
use serde::Serialize;
use std::convert::TryFrom;
use std::ffi::CString;
use std::path::Path;
use structopt::StructOpt;

const SEALED_KEYS_PATH: &str = "pod_data.sealed";
const ENCLAVE_PATH: &str = "../pod-client/pod_enclave/pod_enclave.signed.so";
const MAX_QUOTE_SIZE: usize = 2048;

#[link(name = "pod_sgx")]
extern "C" {
    fn pod_init_enclave(
        enclave_path: *const libc::c_char,
        sealed_state_path: *const libc::c_char,
    ) -> libc::c_int;
    fn pod_load_enclave(
        enclave_path: *const libc::c_char,
        sealed_state_path: *const libc::c_char,
    ) -> libc::c_int;
    fn pod_unload_enclave() -> libc::c_int;
    fn pod_get_quote(
        sp_id_str: *const libc::c_char,
        sp_quote_type_str: *const libc::c_char,
        quote_buffer: *mut u8,
        quote_buffer_size: usize,
    ) -> libc::c_int;
    fn pod_sign_buffer(
        data: *const libc::c_void,
        data_size: usize,
        signature: *mut libc::c_void,
        signature_size: usize,
    ) -> libc::c_int;
}

enum QuoteType {
    #[allow(dead_code)]
    Linkable,
    Unlinkable,
}

#[cfg(unix)]
fn path_to_c_string<P: AsRef<Path>>(path: P) -> anyhow::Result<CString> {
    use std::os::unix::ffi::OsStrExt;
    let s = CString::new(path.as_ref().as_os_str().as_bytes())?;
    Ok(s)
}

#[cfg(windows)]
fn path_to_c_string(path: &Path) -> anyhow::Result<CString> {
    use std::os::windows::ffi::OsStringExt;
    let utf16: Vec<_> = path.as_os_str().encode_wide().collect();
    let s = String::from_utf16(utf16)?;
    let s = CString::new(s.as_bytes())?;
    Ok(s)
}

fn init_enclave<P: AsRef<Path>>(enclave_path: P, sealed_state_path: P) -> anyhow::Result<()> {
    let enclave_path = path_to_c_string(enclave_path)?;
    let sealed_state_path = path_to_c_string(sealed_state_path)?;
    let ret = unsafe { pod_init_enclave(enclave_path.as_ptr(), sealed_state_path.as_ptr()) };

    if ret < 0 {
        Err(anyhow!(
            "pod_init_enclave returned non-zero exit code: {}",
            ret
        ))
    } else {
        Ok(())
    }
}

fn get_quote(spid: &str, quote_type: QuoteType) -> anyhow::Result<Quote> {
    let spid = CString::new(spid)?;
    let quote_type = match quote_type {
        QuoteType::Linkable => CString::new("l")?,
        QuoteType::Unlinkable => CString::new("u")?,
    };
    let quote_buffer = &mut [0u8; MAX_QUOTE_SIZE];
    let ret = unsafe {
        pod_get_quote(
            spid.as_ptr(),
            quote_type.as_ptr(),
            quote_buffer.as_mut_ptr(),
            MAX_QUOTE_SIZE,
        )
    };

    if ret < 0 {
        return Err(anyhow!(
            "pod_init_enclave returned non-zero exit code: {}",
            ret
        ));
    }
    let quote_size = usize::try_from(ret)?;
    Ok(Quote::from(&quote_buffer[..quote_size]))
}

fn load_enclave<P: AsRef<Path>>(enclave_path: P, sealed_state_path: P) -> anyhow::Result<()> {
    let enclave_path = path_to_c_string(enclave_path)?;
    let sealed_state_path = path_to_c_string(sealed_state_path)?;
    let ret = unsafe { pod_load_enclave(enclave_path.as_ptr(), sealed_state_path.as_ptr()) };
    if ret != 0 {
        Err(anyhow!(
            "pod_load_enclave returned non-zero exit code: {}",
            ret
        ))
    } else {
        Ok(())
    }
}

fn unload_enclave() -> anyhow::Result<()> {
    let ret = unsafe { pod_unload_enclave() };
    if ret != 0 {
        Err(anyhow!(
            "pod_unload_enclave returned non-zero exit code: {}",
            ret
        ))
    } else {
        Ok(())
    }
}

fn sign_buffer(message: &[u8], signature: &mut [u8]) -> anyhow::Result<()> {
    let ret = unsafe {
        pod_sign_buffer(
            message.as_ptr() as *const _,
            message.len(),
            signature.as_mut_ptr() as *mut _,
            signature.len(),
        )
    };
    if ret != 0 {
        Err(anyhow!(
            "pod_sign_buffer returned non-zero exit code: {}",
            ret
        ))
    } else {
        Ok(())
    }
}

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

    let address = opt.address.unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = opt.port.unwrap_or(8080);
    let base_uri = format!("http://{}:{}", address, port);
    let client = Client::default();

    if !Path::new(SEALED_KEYS_PATH).exists() {
        // Initialize enclave for the first time
        init_enclave(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
        unload_enclave()?;
    }

    match opt.cmd {
        Command::Register { login, spid } => {
            // Get the quote
            load_enclave(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
            let quote = get_quote(&spid, QuoteType::Unlinkable)?;
            unload_enclave()?;

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
            load_enclave(ENCLAVE_PATH, SEALED_KEYS_PATH)?;
            let challenge = json["challenge"]
                .as_str()
                .ok_or(anyhow!("invalid String for challenge"))?;
            let challenge = base64::decode(challenge)?;
            let response = &mut [0u8; 64];
            sign_buffer(&challenge, response)?;
            let response = base64::encode(&response[..]);
            unload_enclave()?;

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
