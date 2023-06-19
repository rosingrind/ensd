mod ice;
mod udp;

use log::{error, trace};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

use crate::stream::ice::ICE;
use crate::stream::udp::{StunResult, UdpStream};

const PUNCH_HOLE_RETRIES: u8 = 10;

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

    fn get_timeout(&self) -> io::Result<Option<Duration>>;

    fn set_timeout(&self, dur: Option<Duration>) -> io::Result<()>;

    fn get_ttl(&self) -> io::Result<u32>;

    fn set_ttl(&self, ttl: u32) -> io::Result<()>;

    fn get_ip_stun(&self) -> StunResult<SocketAddr>;
}

pub(super) struct StreamHandle {
    socket: Box<dyn IOStream + Sync + Send>,
    pub pub_ip: Option<SocketAddr>,
}

#[allow(dead_code)]
impl StreamHandle {
    pub fn new<A: ToSocketAddrs + Sync>(addr: &A, ttl: Option<u32>) -> StreamHandle {
        let socket = get_udp_stream(addr, ttl);
        let pub_ip = match socket.get_ip_stun() {
            Ok(ip) => {
                trace!(
                    "socket on {:?} allocated with public address {:?}",
                    addr.to_socket_addrs().unwrap().collect::<Vec<SocketAddr>>(),
                    ip
                );
                Some(ip)
            }
            Err(_) => {
                error!(
                    "can't query public address for socket on {:?}",
                    addr.to_socket_addrs().unwrap().collect::<Vec<SocketAddr>>()
                );
                None
            }
        };
        StreamHandle { socket, pub_ip }
    }

    pub async fn bind<A: ToSocketAddrs + Sync>(&self, addr: &A) -> io::Result<()> {
        self.punch_hole(addr, PUNCH_HOLE_RETRIES).await?;
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
    ttl: Option<u32>,
) -> Box<dyn IOStream + Sync + Send> {
    Box::new(UdpStream::new(&*addr.to_socket_addrs().unwrap().collect::<Vec<_>>(), ttl).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{TEST_LOOPBACK_IP, TEST_MACHINE_IP, TEST_STRING};

    use futures::executor::block_on;
    use std::net::SocketAddr;
    use std::sync::Mutex;
    use std::thread;

    static TEST_MUTEX: Mutex<Option<bool>> = Mutex::new(None);

    const PORT_A: u16 = 34254;
    const PORT_B: u16 = 34250;

    #[test]
    fn stream_works() {
        let _guard = TEST_MUTEX.lock().unwrap();

        let addr_a: SocketAddr = SocketAddr::new(TEST_MACHINE_IP, PORT_A);
        let addr_b: SocketAddr = SocketAddr::new(TEST_MACHINE_IP, PORT_B);

        let stream_a = StreamHandle::new(&addr_a, None);
        let stream_b = StreamHandle::new(&addr_b, None);

        let stream_a_pool = &[stream_b.pub_ip.unwrap(), (TEST_LOOPBACK_IP, PORT_B).into()][..];
        let stream_b_pool = &[stream_a.pub_ip.unwrap(), (TEST_LOOPBACK_IP, PORT_A).into()][..];

        let f_a = stream_a.bind(stream_a_pool.last().unwrap());
        let f_b = stream_b.bind(stream_b_pool.last().unwrap());

        let (f_a, f_b) = thread::scope(|s| {
            let t1 = s.spawn(|| block_on(f_a));
            let t2 = s.spawn(|| block_on(f_b));

            (t1.join().unwrap(), t2.join().unwrap())
        });
        // hole punching through NAT in `bind` works
        assert!(f_a.is_ok());
        assert!(f_b.is_ok());

        block_on(stream_b.push(TEST_STRING.as_ref())).unwrap();
        block_on(stream_a.push(TEST_STRING.as_ref())).unwrap();
        let res = block_on(stream_b.poll()).unwrap();
        // bound connection works
        assert_eq!(res.as_slice(), TEST_STRING.as_bytes());
    }

    #[test]
    fn stun_works() {
        let _guard = TEST_MUTEX.lock().unwrap();

        let stream = UdpStream::new(&[SocketAddr::new(TEST_MACHINE_IP, PORT_A)], None).unwrap();
        assert!(stream.get_ip_stun().is_ok());
    }
}
