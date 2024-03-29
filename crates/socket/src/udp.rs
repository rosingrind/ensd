use async_std::{
    future,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    sync::Arc,
};
use async_trait::async_trait;
use bytecodec::{DecodeExt, EncodeExt};
use err::{
    consts::{ERR_CONNECTION, ERR_STUN_QUERY},
    Error, Result,
};
use log::{info, trace};
use rand::Rng;
use stun_codec::{
    rfc5389::{
        attributes::{MappedAddress, Software, XorMappedAddress},
        methods::BINDING,
        Attribute,
    },
    Message, MessageClass, MessageDecoder, MessageEncoder, TransactionId,
};

use crate::{IOSocket, SocketConfig};

pub const MSG_END_TAG: &[u8] = b"end\0msg\0";
pub const PACKET_BUF_SIZE: usize = 256;
pub const STUN_ADDRESS: &str = "stun.l.google.com:19302";

pub(super) struct UdpSocketHandle {
    socket: UdpSocket,
    socket_cfg: Arc<SocketConfig>,
    sw_tag: Option<String>,
}

impl UdpSocketHandle {
    #[allow(dead_code)]
    pub async fn new(
        addr: Vec<SocketAddr>,
        ttl: Option<u32>,
        sw_tag: Option<String>,
        socket_cfg: Arc<SocketConfig>,
    ) -> Result<UdpSocketHandle> {
        let socket = UdpSocket::bind(&*addr).await?;
        if let Some(ttl) = ttl {
            socket.set_ttl(ttl)?;
        }

        Ok(UdpSocketHandle {
            socket,
            socket_cfg,
            sw_tag,
        })
    }
}

// TODO: implement abstract buffer tagging utils
#[async_trait]
impl IOSocket for UdpSocketHandle {
    async fn bind(&self, addr: &[SocketAddr]) -> Result<()> {
        self.socket.connect(addr).await?;
        info!(
            "UDP socket at :{} is connected to {:?}",
            self.socket.local_addr()?.port(),
            self.socket.peer_addr()?
        );
        Ok(())
    }

    async fn peer(&self) -> Result<SocketAddr> {
        self.socket.peer_addr().map_err(Error::from)
    }

    async fn poll(&self) -> Result<Vec<u8>> {
        trace!("polling a message from connected socket");

        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        loop {
            let fut = self.socket.recv(&mut buf);
            let len = if res.is_empty() {
                fut.await
            } else {
                future::timeout(self.socket_cfg.timeout, fut).await?
            }?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break;
            }
        }
        Ok(res[..res.len() - MSG_END_TAG.len()].to_vec())
    }

    async fn poll_at(&self) -> Result<(Vec<u8>, SocketAddr)> {
        trace!("polling a message from any socket");

        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        let addr = loop {
            let fut = self.socket.recv_from(&mut buf);
            let (len, addr) = if res.is_empty() {
                fut.await
            } else {
                future::timeout(self.socket_cfg.timeout, fut).await?
            }?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break addr;
            }
        };
        Ok((res[..res.len() - MSG_END_TAG.len()].to_vec(), addr))
    }

    async fn peek(&self) -> Result<Vec<u8>> {
        trace!("peeking a message from connected socket");

        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        loop {
            let fut = self.socket.peek(&mut buf);
            let len = if res.is_empty() {
                fut.await
            } else {
                future::timeout(self.socket_cfg.timeout, fut).await?
            }?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break;
            }
        }
        Ok(res[..res.len() - MSG_END_TAG.len()].to_vec())
    }

    async fn peek_at(&self) -> Result<(Vec<u8>, SocketAddr)> {
        trace!("peeking a message from any socket");

        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        let addr = loop {
            let fut = self.socket.peek_from(&mut buf);
            let (len, addr) = if res.is_empty() {
                fut.await
            } else {
                future::timeout(self.socket_cfg.timeout, fut).await?
            }?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break addr;
            }
        };
        Ok((res[..res.len() - MSG_END_TAG.len()].to_vec(), addr))
    }

    async fn push(&self, buf: &[u8]) -> Result<()> {
        trace!("pushing {} bytes to connected socket", buf.len());

        for buf in [buf, MSG_END_TAG].concat().chunks(PACKET_BUF_SIZE) {
            self.socket.send(buf).await?;
        }
        Ok(())
    }

    async fn push_to(&self, buf: &[u8], addr: &[SocketAddr]) -> Result<()> {
        trace!("pushing {} bytes to desired socket", buf.len());

        for buf in [buf, MSG_END_TAG].concat().chunks(PACKET_BUF_SIZE) {
            for addr in addr {
                self.socket.send_to(buf, addr).await?;
            }
        }
        Ok(())
    }

    async fn get_ttl(&self) -> Result<u32> {
        self.socket.ttl().map_err(Error::from)
    }

    async fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.socket.set_ttl(ttl).map_err(Error::from)
    }

    async fn get_lan_ip(&self) -> Result<SocketAddr> {
        self.socket.local_addr().map_err(Error::from)
    }

    async fn get_wan_ip(&self) -> Result<SocketAddr> {
        trace!("querying STUN at '{}'", STUN_ADDRESS);

        let stun_addr = STUN_ADDRESS
            .to_socket_addrs()
            .await?
            .find(|c| c.is_ipv4())
            .unwrap();

        let msg = build_request(&self.sw_tag)?;
        let mut buf = [0u8; 256];
        let mut iter = 0..self.socket_cfg.retries;

        loop {
            self.socket.send_to(msg.as_ref(), stun_addr).await?;
            if let Ok(res) =
                future::timeout(self.socket_cfg.timeout, self.socket.recv_from(&mut buf)).await
            {
                match res? {
                    (len, addr) if addr == stun_addr => {
                        let buf = &buf[0..len];
                        let external_addr = decode_address(buf)?;
                        return Ok(external_addr);
                    }
                    _ => {}
                }
                iter = 0..self.socket_cfg.retries;
            }
            if iter.next().is_none() {
                return Err(ERR_CONNECTION.into());
            }
        }
    }
}

#[inline]
fn build_request(software: &Option<String>) -> Result<Vec<u8>> {
    let random_bytes = rand::thread_rng().gen::<[u8; 12]>();

    let mut message = Message::new(
        MessageClass::Request,
        BINDING,
        TransactionId::new(random_bytes),
    );

    if let Some(s) = software {
        message.add_attribute(Attribute::Software(Software::new(s.to_owned())?));
    }

    let mut encoder = MessageEncoder::new();
    let bytes = encoder.encode_into_bytes(message.clone())?;
    Ok(bytes)
}

#[inline]
fn decode_address(buf: &[u8]) -> Result<SocketAddr> {
    let mut decoder = MessageDecoder::<Attribute>::new();
    let decoded = decoder.decode_from_bytes(buf)??;

    let external_addr1 = decoded
        .get_attribute::<XorMappedAddress>()
        .map(|x| x.address());
    let external_addr3 = decoded
        .get_attribute::<MappedAddress>()
        .map(|x| x.address());

    external_addr1
        .or(external_addr3)
        .ok_or_else(|| ERR_STUN_QUERY.into())
}
