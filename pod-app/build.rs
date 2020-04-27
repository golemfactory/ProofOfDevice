use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

const LIB_ROOT: &str = "../lib/c-api";
const ENCLAVE_ROOT: &str = "../pod-enclave";

fn main() {
    // Run Makefile for `pod-enclave` first as we need it to build the
    // c-api lib.
    let mut cmd = Command::new("make");
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir("../pod-enclave");
    cmd.output().expect("make pod-enclave should succeed");
    // Get some basic conf vars
    let sgx_sdk = env::var("SGX_SDK").unwrap_or("/opt/intel/sgxsdk".to_string());
    let sgx_sdk_path = Path::new(&sgx_sdk);
    let sgx_ssl = env::var("SGX_SSL").unwrap_or("/opt/intel/sgxssl".to_string());
    let lib_root = Path::new(LIB_ROOT);
    let enclave_root = Path::new(ENCLAVE_ROOT);
    // Build c-api
    cc::Build::new()
        .files(&[
            lib_root.join("pod_log.c"),
            lib_root.join("pod_sgx.c"),
            enclave_root.join("pod_enclave_u.c"),
        ])
        .include(sgx_sdk_path.join("include"))
        .include(enclave_root)
        .flag_if_supported("-D_GNU_SOURCE")
        .flag_if_supported("-Wno-attributes")
        .flag_if_supported("-std=c99")
        .compile("libpod_sgx.a");
    // Linker flags
    println!("cargo:rustc-flags=-l crypto");
    println!("cargo:rustc-flags=-l sgx_urts");
    println!("cargo:rustc-flags=-l sgx_uae_service");
    println!("cargo:rustc-flags=-l sgx_usgxssl");
    println!("cargo:rustc-flags=-L {}/lib64", sgx_sdk);
    println!("cargo:rustc-flags=-L {}/lib64", sgx_ssl);
}
