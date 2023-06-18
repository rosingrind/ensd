#[cfg(test)]
use std::net::{IpAddr, Ipv4Addr};

pub const MSG_END_TAG: &[u8] = b"end\0msg\0";
pub const PACKET_BUF_SIZE: usize = 256;
pub const RESOURCES_PATH: &str = "res";
pub const STUN_ADDRESS: &str = "stun.l.google.com:19302";

#[cfg(test)]
pub const TEST_PHRASE: &str = "alpha test phrase";
#[cfg(test)]
pub const TEST_STRING: &str = "alpha test string";
#[cfg(test)]
pub const CONNECT_REQ: &[u8] = b"p2p\0req\0";
#[cfg(test)]
pub const TEST_LOOPBACK_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
#[cfg(test)]
pub const TEST_MACHINE_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
