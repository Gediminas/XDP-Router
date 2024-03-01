#![no_std]

#[repr(u32)]
pub enum GlobalRule {
    Policy,
    Size,
}

#[repr(u8)]
#[derive(Debug)]
#[cfg_attr(feature = "user", derive(serde::Deserialize, serde::Serialize))]
pub enum Policy {
    Accept = 0,
    Drop,
}

#[cfg(feature = "user")]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum RouteCmd {
    SetPolicy {
        policy: Policy,
    },
    AddMirror {
        port: u16,
    },
    RemMirror {
        port: u16,
    },
    ListMirrors,

    AddRedirect {
        addr: u32,
        port: u16,
    },
    AddRoute {
        proxy_port: u16,
        dest_addr: u32,
        dest_port: u16,
    },
}
