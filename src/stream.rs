mod udp;

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

use crate::stream::udp::UdpStream;

/// `IOStream` trait for heterogeneous transport implementation.
/// Assumes method implementations to [`connect`][IOStream::connect], [`poll`][IOStream::poll]
/// and [`push`][IOStream::push] data through channel.
pub(super) trait IOStream {
    fn connect(&self, addr: &[SocketAddr]) -> io::Result<()>;

    fn poll(&self) -> io::Result<Vec<u8>>;

    fn push(&self, buf: &[u8]) -> io::Result<()>;
}

pub(super) struct StreamHandle {
    socket: Box<dyn IOStream + Sync + Send>,
}

impl StreamHandle {
    pub fn new<A: ToSocketAddrs>(addr: &A) -> StreamHandle {
        let socket = get_udp_stream(addr);

        StreamHandle { socket }
    }

    pub async fn connect<A: ToSocketAddrs>(&self, addr: &A) -> io::Result<()> {
        let addr = &*addr.to_socket_addrs().unwrap().collect::<Vec<_>>();
        self.socket.connect(addr)
    }

    pub async fn poll(&self) -> io::Result<Vec<u8>> {
        self.socket.poll()
    }

    pub async fn push(&self, buf: &[u8]) -> io::Result<()> {
        self.socket.push(buf)
    }
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
pub(super) fn get_udp_stream<A: ToSocketAddrs>(addr: &A) -> Box<dyn IOStream + Sync + Send> {
    Box::new(UdpStream::new(&*addr.to_socket_addrs().unwrap().collect::<Vec<_>>()).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_STRING;

    use futures::executor::block_on;
    use std::net::{Ipv4Addr, SocketAddr};

    #[test]
    fn stream_works() {
        let ip: Ipv4Addr = Ipv4Addr::from([127, 0, 0, 1]);
        const PORT_A: u16 = 34254;
        const PORT_B: u16 = 34250;

        let addr_a: SocketAddr = SocketAddr::new(ip.into(), PORT_A);
        let addr_b: SocketAddr = SocketAddr::new(ip.into(), PORT_B);

        let stream_a = StreamHandle::new(&addr_a);
        let stream_b = StreamHandle::new(&addr_b);

        block_on(stream_a.connect(&addr_b)).unwrap();
        block_on(stream_b.connect(&addr_a)).unwrap();

        block_on(stream_a.push(TEST_STRING.as_ref())).unwrap();
        let res = block_on(stream_b.poll()).unwrap();
        assert_eq!(res.as_slice(), TEST_STRING.as_bytes());
    }
}
