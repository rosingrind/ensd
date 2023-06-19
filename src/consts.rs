use cpal::FrameCount;
#[cfg(test)]
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

pub const SINC_RING_SIZE: usize = 2;
pub const TRANSPORT_SRATE: u64 = 32000;
pub const CPAL_BUF_SIZE: FrameCount = (PACKET_BUF_SIZE * 8) as FrameCount;
pub const WHITE_SQUARE: char = '\u{25A0}';
pub const BLACK_SQUARE: char = '\u{25A1}';
pub const MSG_END_TAG: &[u8] = b"end\0msg\0";
pub const P2P_REQ_TAG: &[u8] = b"p2p\0req\0";
pub const PACKET_BUF_SIZE: usize = 256;
pub const RESOURCES_PATH: &str = "res";
pub const STUN_ADDRESS: &str = "stun.l.google.com:19302";
pub const TRAVERSAL_MSG_TTL: u32 = 32;
pub const TRAVERSAL_MSG_DUR: Option<Duration> = Some(Duration::from_millis(25));
pub const TRAVERSAL_RETRIES: u16 = 1000;

#[cfg(test)]
pub const TEST_PHRASE: &str = "alpha test phrase";
#[cfg(test)]
pub const TEST_STRING: &str = "alpha test string";
#[cfg(test)]
pub const TEST_LOOPBACK_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
#[cfg(test)]
pub const TEST_MACHINE_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
