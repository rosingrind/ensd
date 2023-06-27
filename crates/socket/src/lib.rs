mod p2p;
mod udp;

use async_std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use async_trait::async_trait;
use howler::Result;
use log::{error, info, trace};
use std::time::Duration;

use crate::p2p::P2P;
use crate::udp::UdpSocketHandle;

const REQUEST_RETRIES: u16 = 1000;
const REQUEST_MSG_DUR: Duration = Duration::from_millis(25);

pub const LOOPBACK_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

/// `IOSocket` trait for heterogeneous transport implementation.
/// Assumes method implementations to [`connect`][IOSocket::connect], [`poll`][IOSocket::poll]
/// and [`push`][IOSocket::push] data through channel.
#[async_trait]
pub trait IOSocket {
    async fn bind(&self, addr: &[SocketAddr]) -> Result<()>;

    async fn peer(&self) -> Result<SocketAddr>;

    async fn poll(&self) -> Result<Vec<u8>>;

    async fn poll_at(&self) -> Result<(Vec<u8>, SocketAddr)>;

    async fn peek(&self) -> Result<Vec<u8>>;

    async fn peek_at(&self) -> Result<(Vec<u8>, SocketAddr)>;

    async fn push(&self, buf: &[u8]) -> Result<()>;

    async fn push_to(&self, buf: &[u8], addr: &[SocketAddr]) -> Result<()>;

    async fn get_ttl(&self) -> Result<u32>;

    async fn set_ttl(&self, ttl: u32) -> Result<()>;

    async fn get_ext_ip(&self, retries: u16) -> Result<SocketAddr>;
}

pub struct SocketHandle {
    socket: Box<dyn IOSocket + Sync + Send>,
    pub pub_ip: Option<SocketAddr>,
}

#[allow(dead_code)]
impl SocketHandle {
    pub async fn new<A: ToSocketAddrs>(
        addr: &A,
        ttl: Option<u32>,
        sw_tag: Option<&'static str>,
    ) -> SocketHandle {
        // TODO: WebRTC socket backend implementation
        let socket = get_udp_socket(addr, ttl, sw_tag).await;
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
        info!(
            "made instance of socket handle with parameters '{:?}'",
            (ttl, sw_tag)
        );

        SocketHandle { socket, pub_ip }
    }

    pub async fn bind<A: ToSocketAddrs>(&self, addr: &A) -> Result<()> {
        self.try_nat_tr(addr, REQUEST_RETRIES).await?;
        let addr = &addr.to_socket_addrs().await.unwrap().collect::<Vec<_>>();
        self.socket.bind(addr).await
    }

    pub async fn peer(&self) -> Result<SocketAddr> {
        self.socket.peer().await
    }

    pub async fn poll(&self) -> Result<Vec<u8>> {
        self.socket.poll().await
    }

    pub async fn poll_at(&self) -> Result<(Vec<u8>, SocketAddr)> {
        self.socket.poll_at().await
    }

    pub async fn peek(&self) -> Result<Vec<u8>> {
        self.socket.peek().await
    }

    pub async fn peek_at(&self) -> Result<(Vec<u8>, SocketAddr)> {
        self.socket.peek_at().await
    }

    pub async fn push(&self, buf: &[u8]) -> Result<()> {
        self.socket.push(buf).await
    }

    pub async fn push_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: &A) -> Result<()> {
        let addr = &addr.to_socket_addrs().await.unwrap().collect::<Vec<_>>();
        self.socket.push_to(buf, addr).await
    }
}

/// A thread-safe `Socket` constructor.
/// Returns [`Box`][Box] wrapped trait object interfaced with abstract [`IOSocket`][IOSocket]
/// trait.
///
/// `Socket` builder is meant to be configurable and used with many transports, so it may be
/// possible to use it with `WebRTC` transport in future.
///
/// Current implementation relies on `UDP`'s [`UdpSocket`][std::net::UdpSocket]
/// opened with any address of [`SocketAddr`][std::net::SocketAddr] type.
pub async fn get_udp_socket<A: ToSocketAddrs>(
    addr: &A,
    ttl: Option<u32>,
    sw_tag: Option<&'static str>,
) -> Box<dyn IOSocket + Sync + Send> {
    trace!("building UDP socket instance");

    Box::new(
        UdpSocketHandle::new(
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
    async fn socket_works() {
        let _guard = TEST_MUTEX.lock().await;

        let addr_a = SocketAddr::new(TEST_MACHINE_IP, PORT_A);
        let addr_b = SocketAddr::new(TEST_MACHINE_IP, PORT_B);

        let socket_a = SocketHandle::new(&addr_a, None, None).await;
        let socket_b = SocketHandle::new(&addr_b, None, None).await;

        let addr_a = SocketAddr::new(LOOPBACK_IP, PORT_B);
        let addr_b = SocketAddr::new(LOOPBACK_IP, PORT_A);

        let handle = futures::try_join!(socket_a.bind(&addr_a), socket_b.bind(&addr_b));
        // hole punching through NAT in `bind` works
        assert!(handle.is_ok());

        let handle = futures::try_join!(
            socket_a.push(TEST_STRING.as_ref()),
            socket_b.push(TEST_STRING.as_ref())
        );
        assert!(handle.is_ok());
        let res = socket_b.poll().await.unwrap();
        // bound connection works
        assert_eq!(res.as_slice(), TEST_STRING.as_bytes());
    }

    #[async_std::test]
    #[cfg_attr(feature = "ci", ignore)]
    async fn stun_works() {
        let _guard = TEST_MUTEX.lock().await;

        let socket = UdpSocketHandle::new(&[SocketAddr::new(TEST_MACHINE_IP, PORT_A)], None, None)
            .await
            .unwrap();

        assert!(socket.get_ext_ip(REQUEST_RETRIES).await.is_ok());
    }
}
