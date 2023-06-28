mod p2p;
mod udp;

use async_std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs},
    sync::Arc,
};
use async_trait::async_trait;
use howler::Result;
use log::{error, info, trace};
use serde::Deserialize;
use std::time::Duration;

use crate::p2p::P2P;
use crate::udp::UdpSocketHandle;

pub const LOOPBACK_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Client {
    UDP {
        addr: ClientAddress,
        ttl: Option<u32>,
        sw_tag: Option<String>,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ClientAddress {
    Single(SocketAddr),
    Vector(Vec<SocketAddr>),
}

#[derive(Debug, Deserialize, Clone)]
pub struct SocketConfig {
    retries: u16,
    timeout: Duration,
}

impl SocketConfig {
    pub fn new(retries: u16, timeout: Duration) -> Self {
        SocketConfig { retries, timeout }
    }
}

impl From<ClientAddress> for Vec<SocketAddr> {
    fn from(value: ClientAddress) -> Vec<SocketAddr> {
        match value {
            ClientAddress::Single(s) => vec![s],
            ClientAddress::Vector(v) => v,
        }
    }
}

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

    async fn get_lan_ip(&self) -> Result<SocketAddr>;

    async fn get_wan_ip(&self) -> Result<SocketAddr>;
}

pub struct SocketHandle {
    socket: Box<dyn IOSocket + Sync + Send>,
    socket_cfg: Arc<SocketConfig>,
    pub pub_ip: SocketAddr,
    pub loc_ip: SocketAddr,
}

#[allow(dead_code)]
impl SocketHandle {
    pub async fn new(cfg: Client, socket_cfg: SocketConfig) -> Result<SocketHandle> {
        let socket_cfg = Arc::new(socket_cfg);
        // TODO: WebRTC socket backend implementation
        let socket = match cfg.clone() {
            Client::UDP { addr, ttl, sw_tag } => {
                get_udp_socket(addr.into(), ttl, sw_tag, socket_cfg.clone()).await
            }
        };
        let loc_ip = match socket.get_lan_ip().await {
            Ok(ip) => ip,
            Err(e) => {
                error!("can't query internal address for socket of {:?}", cfg);
                return Err(e);
            }
        };
        let pub_ip = match socket.get_wan_ip().await {
            Ok(ip) => {
                trace!("socket on {:?} is up with public address {:?}", loc_ip, ip);
                ip
            }
            Err(e) => {
                error!("can't query external address for socket on {:?}", loc_ip);
                return Err(e);
            }
        };
        info!("made instance of socket handle with parameters '{:?}'", cfg);

        Ok(SocketHandle {
            socket,
            socket_cfg,
            pub_ip,
            loc_ip,
        })
    }

    pub async fn bind<A: ToSocketAddrs>(&self, addr: &A) -> Result<()> {
        self.try_nat_tr(addr, self.socket_cfg.retries, self.socket_cfg.timeout)
            .await?;
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
pub async fn get_udp_socket(
    addr: Vec<SocketAddr>,
    ttl: Option<u32>,
    sw_tag: Option<String>,
    socket_cfg: Arc<SocketConfig>,
) -> Box<dyn IOSocket + Sync + Send> {
    trace!("building UDP socket instance");

    Box::new(
        UdpSocketHandle::new(addr, ttl, sw_tag, socket_cfg)
            .await
            .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOCKET_CFG: SocketConfig = SocketConfig {
        retries: 1000,
        timeout: Duration::from_millis(25),
    };
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

        let socket_a = SocketHandle::new(
            Client::UDP {
                addr: ClientAddress::Single(addr_a),
                ttl: None,
                sw_tag: None,
            },
            SOCKET_CFG,
        )
        .await
        .unwrap();
        let socket_b = SocketHandle::new(
            Client::UDP {
                addr: ClientAddress::Single(addr_b),
                ttl: None,
                sw_tag: None,
            },
            SOCKET_CFG,
        )
        .await
        .unwrap();

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

        let socket = UdpSocketHandle::new(
            vec![SocketAddr::new(TEST_MACHINE_IP, PORT_A)],
            None,
            None,
            Arc::new(SOCKET_CFG),
        )
        .await
        .unwrap();

        assert!(socket.get_wan_ip().await.is_ok());
    }
}
