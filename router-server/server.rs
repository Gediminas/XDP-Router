#![deny(clippy::unwrap_used)]

mod args;
mod logger;
mod xdp_ctl;

use anyhow::bail;
use aya::Bpf;
use log::info;
use logger::Logger;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub type Result<T> = std::result::Result<T, anyhow::Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::parse();
    let logger = Logger::build(&args.log_level, &args.log_colored)?;
    log::info!("{args:?}");

    if let Err(e) = run_server(&args.iface, &args.bind).await {
        log::error!("FATAL: {e}");
    }

    logger.close().await;
    Ok(())
}

pub async fn run_server(iface: &str, bind: &str) -> Result<()> {
    if sudo::check() != sudo::RunningAs::Root {
        bail!("Root privileges required!");
    }

    log::info!("Loading XDP on '{iface}'...");
    let bpf = Arc::new(Mutex::new(xdp_ctl::load_xdp(iface)?));
    log::info!("XDP loaded");

    let listener = TcpListener::bind(bind).await?;
    log::info!("Listening tcp://{bind}");

    loop {
        let (mut stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => {
                log::warn!("TCP accept failed: {e:?}");
                continue;
            }
        };

        tokio::spawn({
            let bpf = bpf.clone();
            async move {
                if let Err(e) = serve_request(&mut stream, bpf).await {
                    log::warn!("Request failed: {e:?}");
                }
            }
        });
    }
}

async fn serve_request(stream: &mut TcpStream, bpf: Arc<Mutex<Bpf>>) -> Result<()> {
    let remote = stream.peer_addr()?;
    let mut json = String::new();
    stream.read_to_string(&mut json).await?;

    info!("From {remote}: {json}");

    let cmd = serde_json::from_str(&json)?;
    let res = xdp_ctl::exec(&mut *bpf.lock().await, cmd)?;

    stream.write_all(res.as_bytes()).await?;
    Ok(())
}
