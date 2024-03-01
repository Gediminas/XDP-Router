#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]

mod xdp_process;

use crate::xdp_process::XdpError;
use aya_bpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_log_ebpf::error;

#[xdp]
pub fn router_xdp(ctx: XdpContext) -> u32 {
    match xdp_process::process(&ctx) {
        Ok(ret) => ret,
        Err(e) => {
            let msg = match e {
                XdpError::Outside => "Offset is outside of the packet",
            };
            error!(&ctx, "{} => XDP_ABORTED", msg);
            xdp_action::XDP_ABORTED
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
