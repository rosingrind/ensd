mod udp;

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use crate::stream::udp::UdpStream;

/// `IOStream` trait for heterogeneous transport implementation.
/// Assumes method implementations to [`connect`][IOStream::connect], [`poll`][IOStream::poll]
/// and [`push`][IOStream::push] data through channel.
pub(super) trait IOStream {
    fn connect(&self, addr: &SocketAddr) -> io::Result<()>;

    fn poll(&self) -> io::Result<Vec<u8>>;

    fn push(&self, buf: &[u8]) -> io::Result<()>;
}

/// A thread-safe `Stream` constructor.
/// Returns [`Arc`][Arc] wrapped trait object interfaced with abstract [`IOStream`][IOStream]
/// trait.
///
/// `Stream` builder is meant to be configurable and used with many transports, so it may be
/// possible to use it with `WebRTC` transport in future.
///
/// Current implementation relies on `UDP`'s [`UdpSocket`][std::net::UdpSocket]
/// opened with any address of [`SocketAddr`][std::net::SocketAddr] type.
pub(super) fn get_udp_stream<A: ToSocketAddrs>(addr: &A) -> Arc<dyn IOStream + Sync + Send> {
    Arc::new(UdpStream::new(addr).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr};

    #[test]
    fn stream_works() {
        let ip: Ipv4Addr = Ipv4Addr::from([127, 0, 0, 1]);
        const PORT_A: u16 = 34254;
        const PORT_B: u16 = 34250;

        let addr_a: SocketAddr = SocketAddr::new(ip.into(), PORT_A);
        let addr_b: SocketAddr = SocketAddr::new(ip.into(), PORT_B);

        let stream_a = get_udp_stream(&addr_a);
        let stream_b = get_udp_stream(&addr_b);

        stream_a.connect(&addr_b).unwrap();
        stream_b.connect(&addr_a).unwrap();

        let buf = "data";

        stream_a.push(buf.as_ref()).unwrap();
        let res = stream_b.poll().unwrap();
        assert_eq!(res.as_slice(), buf.as_bytes());
    }
}
