use anyhow::Context;
use aya::{
    include_bytes_aligned,
    maps::{Array, HashMap, MapData},
    programs::{Xdp, XdpFlags},
    Bpf,
};
use aya_log::BpfLogger;
use log::debug;
use router_common::{GlobalRule, HalfRoute, RouteCmd};

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub fn load_xdp(iface: &str) -> Result<Bpf> {
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!("../target/bpfel-unknown-none/debug/router-xdp"))?;

    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!("../target/bpfel-unknown-none/release/router-xdp"))?;

    BpfLogger::init(&mut bpf)?;
    let prog: &mut Xdp = bpf.program_mut("router_xdp").context("XDP prog failed")?.try_into()?;
    prog.load()?;
    prog.attach(iface, XdpFlags::default())?; //XdpFlags::SKB_MODE
    Ok(bpf)
}

#[allow(clippy::unwrap_used)]
pub fn exec(bpf: &mut Bpf, cmd: RouteCmd) -> Result<String> {
    match cmd {
        RouteCmd::SetPolicy { policy } => {
            let mut xdp_global: Array<&mut MapData, u8> = Array::try_from(bpf.map_mut("XDP_ROUTER_GLOBAL").unwrap())?;
            debug!("New policy: {policy:?}");
            xdp_global.set(GlobalRule::Policy as u32, policy as u8, 0)?;
            Ok("OK".to_owned())
        }
        RouteCmd::AddMirror { port } => {
            let mut xdp_mirrors: HashMap<&mut MapData, u16, u8> =
                HashMap::try_from(bpf.map_mut("XDP_ROUTER_MIRRORS").unwrap())?;
            xdp_mirrors.insert(port.to_be(), 1u8, 0)?;
            Ok("OK".to_owned())
        }

        RouteCmd::RemMirror { port } => {
            let mut xdp_mirrors: HashMap<&mut MapData, u16, u8> =
                HashMap::try_from(bpf.map_mut("XDP_ROUTER_MIRRORS").unwrap())?;
            xdp_mirrors.remove(&port.to_be())?;
            Ok("OK".to_owned())
        }
        RouteCmd::ListMirrors => Ok("Not implemented".to_owned()),
        RouteCmd::AddRoute { half1, half2 } => {
            let mut xdp_routes: HashMap<&mut MapData, HalfRoute, HalfRoute> =
                HashMap::try_from(bpf.map_mut("XDP_ROUTER_ROUTES").unwrap())?;
            xdp_routes.insert(half1.to_be(), half2.to_be(), 0)?;
            xdp_routes.insert(half2.to_be(), half1.to_be(), 0)?;
            Ok("OK".to_owned())
        }
        RouteCmd::RemRoute { half1, half2 } => {
            let mut xdp_routes: HashMap<&mut MapData, HalfRoute, HalfRoute> =
                HashMap::try_from(bpf.map_mut("XDP_ROUTER_ROUTES").unwrap())?;
            xdp_routes.remove(&half1.to_be())?;
            xdp_routes.remove(&half2.to_be())?;
            Ok("OK".to_owned())
        }
    }
}
