use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use stunclient::{Error, StunClient};

use crate::consts::STUN_ADDRESS;

pub(super) fn query_stun<A: ToSocketAddrs>(addr: &A) -> Result<SocketAddr, Error> {
    let stun_addr = STUN_ADDRESS
        .to_socket_addrs()
        .unwrap()
        .find(|x| x.is_ipv4())
        .unwrap();
    let udp = UdpSocket::bind(addr).map_err(Error::Socket)?;

    let c = StunClient::new(stun_addr);

    c.query_external_address(&udp)
}
