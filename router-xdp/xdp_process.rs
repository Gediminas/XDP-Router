use aya_bpf::{
    bindings::xdp_action,
    macros::map,
    maps::{Array, HashMap},
    programs::XdpContext,
};
use aya_log_ebpf::{debug, error, info, trace, warn};
use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    udp::UdpHdr,
};
use router_common::{GlobalRule, HalfRoute, Policy};

pub enum XdpError {
    Outside,
}

type XdpResult = Result<xdp_action::Type, XdpError>;

pub const GLOBAL_MAP_SIZE: u32 = GlobalRule::Size as u32;
pub const MIRROR_MAP_SIZE: u32 = u16::MAX as u32;
pub const REDIR_MAP_SIZE: u32 = u16::MAX as u32;

#[map(name = "XDP_ROUTER_GLOBAL")]
pub static mut XDP_ROUTER_GLOBAL: Array<u8> = Array::<u8>::with_max_entries(GLOBAL_MAP_SIZE, 0);

#[map(name = "XDP_ROUTER_MIRRORS")]
pub static mut XDP_ROUTER_MIRRORS: HashMap<u16, u8> = HashMap::<u16, u8>::with_max_entries(MIRROR_MAP_SIZE, 0);

#[map(name = "XDP_ROUTER_ROUTES")]
pub static mut XDP_ROUTER_ROUTE: HashMap<HalfRoute, HalfRoute> =
    HashMap::<HalfRoute, HalfRoute>::with_max_entries(REDIR_MAP_SIZE, 0);

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
    trace!(ctx, "inbound UDP {:i}:{} --> {:i}:{} [0x{:x}] TTL:{}", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check), u8::from_be(ip.ttl) );

    // TEST: Mirror
    if udp.dest == 65500_u16.to_be() {
        warn!(ctx, "TEST on 65500: Mirror/pong packet back");
        mem::swap(&mut eth.src_addr, &mut eth.dst_addr);
        mem::swap(&mut ip.src_addr, &mut ip.dst_addr);
        mem::swap(&mut udp.source, &mut udp.dest);

        //TODO: Decrement TTL and adjust checksum

        #[rustfmt::skip]
        warn!(ctx, " > XDP_TX   {:i}:{} --> {:i}:{} [0x{:x}] TTL:{}", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check), u8::from_be(ip.ttl) );
        return Ok(xdp_action::XDP_TX);
    }

    match unsafe { XDP_ROUTER_MIRRORS.get(&udp.dest) } {
        Some(found) => {
            if found == &mut 1u8 {
                mem::swap(&mut eth.src_addr, &mut eth.dst_addr);
                mem::swap(&mut ip.src_addr, &mut ip.dst_addr);
                mem::swap(&mut udp.source, &mut udp.dest);

                //TODO: Decrement TTL and adjust checksum

                #[rustfmt::skip]
                warn!(ctx, " > XDP_TX   {:i}:{} --> {:i}:{} [0x{:x}] TTL:{}", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check), u8::from_be(ip.ttl) );
                return Ok(xdp_action::XDP_TX);
            }
        }
        _ => {}
    }

    let key = HalfRoute::new(ip.src_addr, udp.source, udp.dest);

    unsafe {
        match XDP_ROUTER_ROUTE.get_ptr_mut(&key) {
            Some(found) => {
                let found = *found;

                //NOTE: on testing virtual env macs are the same for peer1 and peer2
                //     real NICs ignore it maybe (to check, or add option, or collect macs, or smth)
                mem::swap(&mut eth.src_addr, &mut eth.dst_addr);

                let old_src_addr = ip.src_addr;
                ip.src_addr = ip.dst_addr;
                ip.dst_addr = found.reflexive_addr;
                ip.check = adjust_checksum_be(ip.check, old_src_addr, ip.dst_addr);

                //TODO: Decrement TTL, adjust checksum

                udp.source = found.router_port;
                udp.dest = found.reflexive_port;
                udp.check = 0; //FIX: Update checksum (2 x 16-bit blocks)

                #[rustfmt::skip]
                warn!(ctx, " > xdp_tx   {:i}:{} --> {:i}:{} [0x{:x}]", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check) );

                return Ok(xdp_action::XDP_TX);
            }
            None => {}
        }
    }

    match unsafe { XDP_ROUTER_GLOBAL.get(GlobalRule::Policy as u32) } {
        Some(policy) if *policy == Policy::Drop as u8 => {
            trace!(ctx, "DROP");
            return Ok(xdp_action::XDP_DROP);
        }
        _ => {}
    }

    #[rustfmt::skip]
    debug!(ctx, " > xdp_pass {:i}:{} --> {:i}:{} [0x{:x}]", u32::from_be(ip.src_addr), u16::from_be(udp.source), u32::from_be(ip.dst_addr), u16::from_be(udp.dest), u16::from_be(ip.check) );
    return Ok(xdp_action::XDP_PASS);
}

/// Params and result are big-endian
/// FIX: This is a back-engineered algorithm, not fully tested, so expect some packets dropped... :(
///      For some reason `wrapping_sub` & `wrapping_add` does not work as expected
///      and `aya_bpf::helpers::bpf_csum_diff` does not
///      (difference of the same value results in 0xffff... value instead of 0x0,
///      zero is not used at all, therefore adjustments needed when wrapping)
#[inline(always)]
fn adjust_checksum_be(check: u16, old_ip: u32, new_ip: u32) -> u16 {
    let old_ip = u32::from_be(old_ip);
    let new_ip = u32::from_be(new_ip);
    let n1 = (new_ip >> 16) as u16;
    let o1 = (old_ip >> 16) as u16;
    let n2 = new_ip as u16;
    let o2 = old_ip as u16;

    let mut check = u16::from_be(check);

    //1
    if check < n1 {
        check -= 1;
    }
    check -= n1;

    if check + o1 < check {
        check += 1;
    }
    check += o1;

    //2
    if check < n2 {
        check -= 1;
    }
    check -= n2;

    if check + o2 < check {
        check += 1;
    }
    check += o2;

    return check.to_be();
}

#[allow(dead_code)]
#[inline(always)]
fn test(ctx: &XdpContext, old_ip: [u8; 4], new_ip: [u8; 4], check: u16, check_expect: u16) {
    let old_ip_be = u32::from_le_bytes(old_ip);
    let new_ip_be = u32::from_le_bytes(new_ip);
    let check = adjust_checksum_be(check, old_ip_be, new_ip_be);

    if check == check_expect {
        info!(ctx, "{:i} -> {:x} ? {:x}", u32::from_be(new_ip_be), u16::from_be(check), u16::from_be(check_expect));
    } else {
        error!(ctx, "{:i} -> {:x} ? {:x}", u32::from_be(new_ip_be), u16::from_be(check), u16::from_be(check_expect));
    }
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
