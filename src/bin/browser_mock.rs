use anyhow::{anyhow, Result};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use serde_json::Value;
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// Path to native messaging enabled binary.
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let mut child = Command::new(&opt.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut json = String::new();
        io::stdin().read_line(&mut json)?;
        let json = json.trim();

        if json.is_empty() {
            continue;
        }

        if json == "exit" {
            break;
        }

        let msg_len = json.len();
        let msg_len = u32::try_from(msg_len)?;

        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or(anyhow!("failed to open stdin"))?;
            stdin.write_all(&msg_len.to_ne_bytes())?;
            stdin.write_all(json.as_bytes())?;
        }
        println!("Message sent.");

        {
            let stdout = child
                .stdout
                .as_mut()
                .ok_or(anyhow!("failed to open stdout"))?;
            let mut msg_len = [0u8; 4];
            stdout.read_exact(&mut msg_len)?;
            let msg_len = u32::from_ne_bytes(msg_len);
            let msg_len = usize::try_from(msg_len).expect("u32 should fit into usize");

            let mut msg = Vec::new();
            msg.resize(msg_len, 0);
            stdout.read_exact(&mut msg)?;
            let output: Value = serde_json::from_slice(&msg)?;
            println!("{}", output);
        }
    }
    signal::kill(Pid::from_raw(child.id() as i32), Some(Signal::SIGTERM))?;
    let status_code = child.wait()?;
    println!("Child exited with {}", status_code);

    Ok(())
}
