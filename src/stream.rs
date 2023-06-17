mod ice;
mod udp;

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

use crate::stream::udp::UdpStream;

/// `IOStream` trait for heterogeneous transport implementation.
/// Assumes method implementations to [`connect`][IOStream::connect], [`poll`][IOStream::poll]
/// and [`push`][IOStream::push] data through channel.
pub(super) trait IOStream {
    fn bind(&self, addr: &[SocketAddr]) -> io::Result<()>;

    fn peer(&self) -> io::Result<SocketAddr>;

    fn poll(&self) -> io::Result<Vec<u8>>;

    fn poll_at(&self) -> io::Result<(Vec<u8>, SocketAddr)>;

    fn push(&self, buf: &[u8]) -> io::Result<()>;

    fn push_to(&self, buf: &[u8], addr: &[SocketAddr]) -> io::Result<()>;
}

pub(super) struct StreamHandle {
    socket: Box<dyn IOStream + Sync + Send>,
    pub pub_ip: Option<SocketAddr>,
}

#[allow(dead_code)]
impl StreamHandle {
    pub fn new<A: ToSocketAddrs>(addr: &A, dur: Option<Duration>) -> StreamHandle {
        let pub_ip = match ice::query_stun(addr) {
            Ok(ip) => Some(ip),
            Err(_) => None,
        };

        let socket = get_udp_stream(addr, dur);
        StreamHandle { socket, pub_ip }
    }

    pub async fn bind<A: ToSocketAddrs>(&self, addr: &A) -> io::Result<()> {
        let addr = &*addr.to_socket_addrs().unwrap().collect::<Vec<_>>();
        self.socket.bind(addr)
    }

    pub async fn peer(&self) -> io::Result<SocketAddr> {
        self.socket.peer()
    }

    pub async fn poll(&self) -> io::Result<Vec<u8>> {
        self.socket.poll()
    }

    pub async fn poll_at(&self) -> io::Result<(Vec<u8>, SocketAddr)> {
        self.socket.poll_at()
    }

    pub async fn push(&self, buf: &[u8]) -> io::Result<()> {
        self.socket.push(buf)
    }

    pub async fn push_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: &A) -> io::Result<()> {
        let addr = &*addr.to_socket_addrs().unwrap().collect::<Vec<_>>();
        self.socket.push_to(buf, addr)
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
pub(super) fn get_udp_stream<A: ToSocketAddrs>(
    addr: &A,
    dur: Option<Duration>,
) -> Box<dyn IOStream + Sync + Send> {
    Box::new(UdpStream::new(&*addr.to_socket_addrs().unwrap().collect::<Vec<_>>(), dur).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{CONNECT_REQ, TEST_LOOPBACK_IP, TEST_MACHINE_IP, TEST_STRING};

    use futures::executor::block_on;
    use std::net::SocketAddr;
    use std::time::Duration;

    const DUR: Option<Duration> = Some(Duration::from_secs(1));
    const PORT_A: u16 = 34254;
    const PORT_B: u16 = 34250;

    #[test]
    fn stream_works() {
        let addr_a: SocketAddr = SocketAddr::new(TEST_MACHINE_IP, PORT_A);
        let addr_b: SocketAddr = SocketAddr::new(TEST_MACHINE_IP, PORT_B);

        let stream_a = StreamHandle::new(&addr_a, DUR);
        let stream_b = StreamHandle::new(&addr_b, DUR);

        let stream_a_pool = &[stream_b.pub_ip.unwrap(), (TEST_LOOPBACK_IP, PORT_B).into()][..];
        let stream_b_pool = &[stream_a.pub_ip.unwrap(), (TEST_LOOPBACK_IP, PORT_A).into()][..];

        block_on(stream_a.bind(&stream_a_pool.last().unwrap())).unwrap();
        block_on(stream_b.bind(&stream_b_pool.last().unwrap())).unwrap();

        block_on(stream_b.push(TEST_STRING.as_ref())).unwrap();
        block_on(stream_a.push(TEST_STRING.as_ref())).unwrap();
        let res = block_on(stream_b.poll()).unwrap();
        // bound connection works
        assert_eq!(res.as_slice(), TEST_STRING.as_bytes());

        block_on(stream_a.push_to(CONNECT_REQ, &stream_a_pool)).unwrap();
        block_on(stream_b.push_to(CONNECT_REQ, &stream_b_pool)).unwrap();
        let (res, addr) = loop {
            block_on(stream_a.push_to(CONNECT_REQ, &stream_a_pool)).unwrap();
            let res = block_on(stream_b.poll_at());
            if let Ok(res) = res {
                break res;
            }
        };
        // NAT hole punching works
        assert_eq!(res.as_slice(), CONNECT_REQ);
        assert!(stream_b_pool.contains(&addr));
    }

    #[test]
    fn stun_works() {
        let addr = &(TEST_MACHINE_IP, 0);
        assert!(ice::query_stun(addr).is_ok());
    }
}
