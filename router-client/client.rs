#![deny(clippy::unwrap_used)]

use anyhow::bail;
use router_common::{Policy, RouteCmd};
use std::{
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

pub type Result<T> = std::result::Result<T, anyhow::Error>;

const STREAM_READ_TIMEOUT: Option<Duration> = Some(Duration::from_secs(3));
const STREAM_WRITE_TIMEOUT: Option<Duration> = Some(Duration::from_secs(3));

fn parse_args() -> Result<(SocketAddr, RouteCmd)> {
    let mut args = env::args().skip(1);
    let endpoint = args.next().expect("Router IP").parse::<SocketAddr>()?;
    let action = args.next();
    let object = args.next();

    let command = match (action.as_deref(), object.as_deref()) {
        (Some("set"), Some("policy")) => {
            let policy = match args.next().as_deref() {
                Some("accept") => Policy::Accept,
                Some("drop") => Policy::Drop,
                _ => bail!("Expected drop/accept"),
            };
            RouteCmd::SetPolicy { policy }
        }
        (Some("add"), Some("mirror")) => {
            let port = match args.next() {
                Some(p) => p.parse::<u16>()?,
                None => bail!("Expected port"),
            };
            RouteCmd::AddMirror { port }
        }
        (Some("rem"), Some("mirror")) => {
            let port = match args.next() {
                Some(p) => p.parse::<u16>()?,
                None => bail!("Expected port"),
            };
            RouteCmd::RemMirror { port }
        }
        (Some("list"), Some("mirrors")) => RouteCmd::ListMirrors,
        _ => bail!("Unknown or incomplete command provided"),
    };

    Ok((endpoint, command))
}

fn main() -> Result<()> {
    let (router, command) = parse_args()?;
    let json = serde_json::to_string(&command)?;

    println!("Connecting to {router}");

    let mut stream = TcpStream::connect(router)?;
    stream.set_write_timeout(STREAM_WRITE_TIMEOUT)?;
    stream.set_read_timeout(STREAM_READ_TIMEOUT)?;

    println!("Sending: {json}");

    stream.write_all(json.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?; //Sends EOF for read_to_string

    let mut res = String::new();
    stream.read_to_string(&mut res)?;

    println!("{res}");

    Ok(())
}
