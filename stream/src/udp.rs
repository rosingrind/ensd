use async_std::{
    future,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};
use async_trait::async_trait;
use bytecodec::{DecodeExt, EncodeExt};
use log::{info, trace};
use rand::Rng;
use std::io;
use std::io::{Error, ErrorKind};
use stun_codec::{
    rfc5389::{
        attributes::{MappedAddress, Software, XorMappedAddress},
        methods::BINDING,
        Attribute,
    },
    Message, MessageClass, MessageDecoder, MessageEncoder, TransactionId,
};

use crate::err::{ERR_CONNECTION, ERR_STUN_QUERY};
use crate::IOStream;
use crate::REQUEST_MSG_DUR;

pub const MSG_END_TAG: &[u8] = b"end\0msg\0";
pub const PACKET_BUF_SIZE: usize = 256;
pub const STUN_ADDRESS: &str = "stun.l.google.com:19302";

pub(super) struct UdpStream {
    socket: UdpSocket,
    sw_tag: Option<&'static str>,
}

impl UdpStream {
    #[allow(dead_code)]
    pub async fn new(
        addr: &[SocketAddr],
        ttl: Option<u32>,
        sw_tag: Option<&'static str>,
    ) -> io::Result<UdpStream> {
        let socket = UdpSocket::bind(addr).await?;
        if let Some(ttl) = ttl {
            socket.set_ttl(ttl)?;
        }

        Ok(UdpStream { socket, sw_tag })
    }
}

// TODO: implement abstract buffer tagging utils
#[async_trait]
impl IOStream for UdpStream {
    async fn bind(&self, addr: &[SocketAddr]) -> io::Result<()> {
        self.socket.connect(addr).await?;
        info!(
            "UDP socket at :{} is connected to {:?}",
            self.socket.local_addr()?.port(),
            addr
        );
        Ok(())
    }

    async fn peer(&self) -> io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

    async fn poll(&self) -> io::Result<Vec<u8>> {
        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        loop {
            let len = self.socket.recv(&mut buf).await?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break;
            }
        }
        Ok(res[..res.len() - MSG_END_TAG.len()].to_vec())
    }

    async fn poll_at(&self) -> io::Result<(Vec<u8>, SocketAddr)> {
        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        let addr = loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break addr;
            }
        };
        Ok((res[..res.len() - MSG_END_TAG.len()].to_vec(), addr))
    }

    async fn peek(&self) -> io::Result<Vec<u8>> {
        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        loop {
            let len = self.socket.peek(&mut buf).await?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break;
            }
        }
        Ok(res[..res.len() - MSG_END_TAG.len()].to_vec())
    }

    async fn peek_at(&self) -> io::Result<(Vec<u8>, SocketAddr)> {
        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        let addr = loop {
            let (len, addr) = self.socket.peek_from(&mut buf).await?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break addr;
            }
        };
        Ok((res[..res.len() - MSG_END_TAG.len()].to_vec(), addr))
    }

    async fn push(&self, buf: &[u8]) -> io::Result<()> {
        for buf in [buf, MSG_END_TAG].concat().chunks(PACKET_BUF_SIZE) {
            self.socket.send(buf).await?;
        }
        Ok(())
    }

    async fn push_to(&self, buf: &[u8], addr: &[SocketAddr]) -> io::Result<()> {
        for buf in [buf, MSG_END_TAG].concat().chunks(PACKET_BUF_SIZE) {
            for addr in addr {
                self.socket.send_to(buf, addr).await?;
            }
        }
        Ok(())
    }

    async fn get_ttl(&self) -> io::Result<u32> {
        self.socket.ttl()
    }

    async fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.socket.set_ttl(ttl)
    }

    async fn get_ext_ip(&self, retries: u16) -> io::Result<SocketAddr> {
        trace!("querying STUN at '{}'", STUN_ADDRESS);
        let stun_addr = STUN_ADDRESS
            .to_socket_addrs()
            .await?
            .find(|c| c.is_ipv4())
            .unwrap();

        let msg = build_request(self.sw_tag)?;
        let mut buf = [0u8; 256];
        let mut iter = 0..retries;

        loop {
            self.socket.send_to(msg.as_ref(), stun_addr).await?;
            if let Ok(res) =
                future::timeout(REQUEST_MSG_DUR.unwrap(), self.socket.recv_from(&mut buf)).await
            {
                match res? {
                    (len, addr) if addr == stun_addr => {
                        let buf = &buf[0..len];
                        let external_addr = decode_address(buf)?;
                        return Ok(external_addr);
                    }
                    _ => {}
                }
                iter = 0..retries;
            }
            if iter.next().is_none() {
                break Err(ERR_CONNECTION.into());
            }
        }
    }
}

#[inline]
fn build_request(software: Option<&str>) -> Result<Vec<u8>, Error> {
    let random_bytes = rand::thread_rng().gen::<[u8; 12]>();

    let mut message = Message::new(
        MessageClass::Request,
        BINDING,
        TransactionId::new(random_bytes),
    );

    if let Some(s) = software {
        message.add_attribute(Attribute::Software(
            Software::new(s.to_owned()).map_err(|e| Error::new(ErrorKind::InvalidInput, e))?,
        ));
    }

    let mut encoder = MessageEncoder::new();
    let bytes = encoder
        .encode_into_bytes(message.clone())
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(bytes)
}

#[inline]
fn decode_address(buf: &[u8]) -> Result<SocketAddr, Error> {
    let mut decoder = MessageDecoder::<Attribute>::new();
    let decoded = decoder
        .decode_from_bytes(buf)
        .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?
        .map_err(|e| Error::new(ErrorKind::Other, format!("{:?}", e)))?;

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
