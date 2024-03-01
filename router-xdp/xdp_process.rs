use aya_bpf::{
    bindings::xdp_action,
    macros::map,
    maps::{Array, HashMap},
    programs::XdpContext,
};
use aya_log_ebpf::{debug, trace, warn};
use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    udp::UdpHdr,
};
use router_common::{GlobalRule, Policy};

pub enum XdpError {
    Outside,
}

type XdpResult = Result<xdp_action::Type, XdpError>;

pub const GLOBAL_MAP_SIZE: u32 = GlobalRule::Size as u32;
pub const MIRROR_MAP_SIZE: u32 = u16::MAX as u32;

#[map(name = "XDP_ROUTER_GLOBAL")]
pub static mut XDP_ROUTER_GLOBAL: Array<u8> = Array::<u8>::with_max_entries(GLOBAL_MAP_SIZE, 0);

#[map(name = "XDP_ROUTER_MIRRORS")]
pub static mut XDP_ROUTER_MIRRORS: HashMap<u16, u8> = HashMap::<u16, u8>::with_max_entries(MIRROR_MAP_SIZE, 0);

#[inline(always)]
pub fn process(ctx: &XdpContext) -> XdpResult {
    let eth: &mut EthHdr = get_at_mut(&ctx, 0)?;
    match eth.ether_type {
        EtherType::Ipv4 => process_ipv4(ctx, EthHdr::LEN, eth),
        _ => Ok(xdp_action::XDP_PASS),
    }
}
#[inline(always)]
fn process_ipv4(ctx: &XdpContext, offset: usize, eth: &mut EthHdr) -> XdpResult {
    let ip: &mut Ipv4Hdr = get_at_mut(&ctx, offset)?;

    match ip.proto {
        IpProto::Udp => process_udp(&ctx, offset + Ipv4Hdr::LEN, ip, eth), //FIX: IPv4 length is dynamic perhaps...
        _ => return Ok(xdp_action::XDP_PASS),
    }
}

#[inline(always)]
fn process_udp(ctx: &XdpContext, offset: usize, ip: &mut Ipv4Hdr, eth: &mut EthHdr) -> XdpResult {
    let udp: &mut UdpHdr = get_at_mut(ctx, offset)?;

    #[rustfmt::skip]
    trace!(ctx, "inbound UDP {:i}:{} --> {:i}:{} [0x{:x}] TTL:{}", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u8::from_be(ip.ttl), u16::from_be(ip.check) );

    // TEST: Mirror
    if udp.dest == 65500_u16.to_be() {
        warn!(ctx, "TEST on 65500: Mirror/pong packet back");
        mem::swap(&mut eth.src_addr, &mut eth.dst_addr);
        mem::swap(&mut ip.src_addr, &mut ip.dst_addr);
        mem::swap(&mut udp.source, &mut udp.dest);

        #[rustfmt::skip]
        warn!(ctx, " > XDP_TX   {:i}:{} --> {:i}:{} [0x{:x}]", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check), );
        return Ok(xdp_action::XDP_TX);
    }

    match unsafe { XDP_ROUTER_MIRRORS.get(&udp.dest) } {
        Some(found) => {
            trace!(ctx, "Mirror found {}", *found as u8);

            if found == &mut 1u8 {
                mem::swap(&mut eth.src_addr, &mut eth.dst_addr);
                mem::swap(&mut ip.src_addr, &mut ip.dst_addr);
                mem::swap(&mut udp.source, &mut udp.dest);

                #[rustfmt::skip]
                warn!(ctx, " > XDP_TX   {:i}:{} --> {:i}:{} [0x{:x}]", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check), );
                return Ok(xdp_action::XDP_TX);
            }
        }
        _ => {}
    }

    match unsafe { XDP_ROUTER_GLOBAL.get(GlobalRule::Policy as u32) } {
        Some(policy) if *policy == Policy::Drop as u8 => {
            trace!(ctx, "DROP (managed)");
            return Ok(xdp_action::XDP_DROP);
        }
        _ => {}
    }

    trace!(ctx, "PASS (default)");
    #[rustfmt::skip]
    debug!(ctx, " > xdp_pass {:i}:{} --> {:i}:{} [0x{:x}]", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check), );
    return Ok(xdp_action::XDP_PASS);
}

#[allow(dead_code)]
#[inline(always)]
fn get_at<'a, T>(ctx: &'a XdpContext, offset: usize) -> Result<&'a T, XdpError> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(XdpError::Outside);
    }

    let ptr = (start + offset) as *const T;
    unsafe { Ok(&*ptr) }
}

#[inline(always)]
fn get_at_mut<T>(ctx: &XdpContext, offset: usize) -> Result<&mut T, XdpError> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(XdpError::Outside);
    }

    let ptr = (start + offset) as *mut T;
    unsafe { Ok(&mut *ptr) }
}
