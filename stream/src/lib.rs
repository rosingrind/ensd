mod err;
mod p2p;
mod udp;

use async_std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use async_trait::async_trait;
use log::{error, trace};
use std::io;
use std::time::Duration;

use crate::p2p::P2P;
use crate::udp::UdpStream;

const REQUEST_RETRIES: u16 = 1000;
const REQUEST_MSG_DUR: Duration = Duration::from_millis(25);

pub const LOOPBACK_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

/// `IOStream` trait for heterogeneous transport implementation.
/// Assumes method implementations to [`connect`][IOStream::connect], [`poll`][IOStream::poll]
/// and [`push`][IOStream::push] data through channel.
#[async_trait]
pub trait IOStream {
    async fn bind(&self, addr: &[SocketAddr]) -> io::Result<()>;

    async fn peer(&self) -> io::Result<SocketAddr>;

    async fn poll(&self) -> io::Result<Vec<u8>>;

    async fn poll_at(&self) -> io::Result<(Vec<u8>, SocketAddr)>;

    async fn peek(&self) -> io::Result<Vec<u8>>;

    async fn peek_at(&self) -> io::Result<(Vec<u8>, SocketAddr)>;

    async fn push(&self, buf: &[u8]) -> io::Result<()>;

    async fn push_to(&self, buf: &[u8], addr: &[SocketAddr]) -> io::Result<()>;

    async fn get_ttl(&self) -> io::Result<u32>;

    async fn set_ttl(&self, ttl: u32) -> io::Result<()>;

    async fn get_ext_ip(&self, retries: u16) -> io::Result<SocketAddr>;
}

pub struct StreamHandle {
    socket: Box<dyn IOStream + Sync + Send>,
    pub pub_ip: Option<SocketAddr>,
}

#[allow(dead_code)]
impl StreamHandle {
    pub async fn new<A: ToSocketAddrs>(
        addr: &A,
        ttl: Option<u32>,
        sw_tag: Option<&'static str>,
    ) -> StreamHandle {
        // TODO: WebRTC socket backend implementation
        let socket = get_udp_stream(addr, ttl, sw_tag).await;
        let pub_ip = match socket.get_ext_ip(REQUEST_RETRIES).await {
            Ok(ip) => {
                trace!(
                    "socket on {:?} allocated with public address {:?}",
                    addr.to_socket_addrs()
                        .await
                        .unwrap()
                        .collect::<Vec<SocketAddr>>(),
                    ip
                );
                Some(ip)
            }
            Err(_) => {
                error!(
                    "can't query public address for socket on {:?}",
                    addr.to_socket_addrs()
                        .await
                        .unwrap()
                        .collect::<Vec<SocketAddr>>()
                );
                None
            }
        };
        StreamHandle { socket, pub_ip }
    }

    pub async fn bind<A: ToSocketAddrs>(&self, addr: &A) -> io::Result<()> {
        self.try_nat_tr(addr, REQUEST_RETRIES).await?;
        let addr = &addr.to_socket_addrs().await.unwrap().collect::<Vec<_>>();
        self.socket.bind(addr).await
    }

    pub async fn peer(&self) -> io::Result<SocketAddr> {
        self.socket.peer().await
    }

    pub async fn poll(&self) -> io::Result<Vec<u8>> {
        self.socket.poll().await
    }

    pub async fn poll_at(&self) -> io::Result<(Vec<u8>, SocketAddr)> {
        self.socket.poll_at().await
    }

    pub async fn peek(&self) -> io::Result<Vec<u8>> {
        self.socket.peek().await
    }

    pub async fn peek_at(&self) -> io::Result<(Vec<u8>, SocketAddr)> {
        self.socket.peek_at().await
    }

    pub async fn push(&self, buf: &[u8]) -> io::Result<()> {
        self.socket.push(buf).await
    }

    pub async fn push_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: &A) -> io::Result<()> {
        let addr = &addr.to_socket_addrs().await.unwrap().collect::<Vec<_>>();
        self.socket.push_to(buf, addr).await
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
pub async fn get_udp_stream<A: ToSocketAddrs>(
    addr: &A,
    ttl: Option<u32>,
    sw_tag: Option<&'static str>,
) -> Box<dyn IOStream + Sync + Send> {
    Box::new(
        UdpStream::new(
            &addr.to_socket_addrs().await.unwrap().collect::<Vec<_>>(),
            ttl,
            sw_tag,
        )
        .await
        .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STRING: &str = "alpha test string";
    const TEST_MACHINE_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    const PORT_A: u16 = 34254;
    const PORT_B: u16 = 34250;

    static TEST_MUTEX: async_std::sync::Mutex<Option<bool>> = async_std::sync::Mutex::new(None);

    #[async_std::test]
    #[cfg_attr(feature = "ci", ignore)]
    async fn stream_works() {
        let _guard = TEST_MUTEX.lock().await;

        let addr_a = SocketAddr::new(TEST_MACHINE_IP, PORT_A);
        let addr_b = SocketAddr::new(TEST_MACHINE_IP, PORT_B);

        let stream_a = StreamHandle::new(&addr_a, None, None).await;
        let stream_b = StreamHandle::new(&addr_b, None, None).await;

        let addr_a = SocketAddr::new(LOOPBACK_IP, PORT_B);
        let addr_b = SocketAddr::new(LOOPBACK_IP, PORT_A);

        let handle = futures::try_join!(stream_a.bind(&addr_a), stream_b.bind(&addr_b));
        // hole punching through NAT in `bind` works
        assert!(handle.is_ok());

        let handle = futures::try_join!(
            stream_a.push(TEST_STRING.as_ref()),
            stream_b.push(TEST_STRING.as_ref())
        );
        assert!(handle.is_ok());
        let res = stream_b.poll().await.unwrap();
        // bound connection works
        assert_eq!(res.as_slice(), TEST_STRING.as_bytes());
    }

    #[async_std::test]
    #[cfg_attr(feature = "ci", ignore)]
    async fn stun_works() {
        let _guard = TEST_MUTEX.lock().await;

        let stream = UdpStream::new(&[SocketAddr::new(TEST_MACHINE_IP, PORT_A)], None, None)
            .await
            .unwrap();

        assert!(stream.get_ext_ip(REQUEST_RETRIES).await.is_ok());
    }
}
