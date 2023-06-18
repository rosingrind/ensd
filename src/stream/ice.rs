use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use stunclient::{Error, StunClient};

use crate::consts::STUN_ADDRESS;

/// Async `query_stun` function for querying external IP address. Allocates a dummy UDP socket
/// and performs [`STUN`][StunClient] request with
/// [`query_external_address`][StunClient::query_external_address].
pub(super) async fn query_stun<A: ToSocketAddrs>(addr: &A) -> Result<SocketAddr, Error> {
    let stun_addr = STUN_ADDRESS
        .to_socket_addrs()
        .unwrap()
        .find(|x| x.is_ipv4())
        .unwrap();
    let udp = UdpSocket::bind(addr).map_err(Error::Socket)?;

    let c = StunClient::new(stun_addr);

    c.query_external_address(&udp)
}
