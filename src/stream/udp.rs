use crate::consts::{MSG_END_TAG, PACKET_BUF_SIZE, STUN_ADDRESS};
use log::{info, trace};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;
use stunclient::{Error as StunError, StunClient};

use crate::stream::IOStream;

pub(super) struct UdpStream {
    socket: UdpSocket,
}

pub(super) type StunResult<T> = Result<T, StunError>;

impl UdpStream {
    #[allow(dead_code)]
    pub fn new(addr: &[SocketAddr], ttl: Option<u32>) -> io::Result<UdpStream> {
        let socket = UdpSocket::bind(addr)?;
        if let Some(ttl) = ttl {
            socket.set_ttl(ttl)?;
        }

        Ok(UdpStream { socket })
    }
}

impl IOStream for UdpStream {
    fn bind(&self, addr: &[SocketAddr]) -> io::Result<()> {
        self.socket.connect(addr)?;
        info!(
            "UDP socket at :{} is connected to {:?}",
            self.socket.local_addr()?.port(),
            addr
        );
        Ok(())
    }

    fn peer(&self) -> io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

    fn poll(&self) -> io::Result<Vec<u8>> {
        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        loop {
            let len = self.socket.recv(&mut buf)?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break;
            }
        }
        Ok(res[..res.len() - MSG_END_TAG.len()].to_vec())
    }

    fn poll_at(&self) -> io::Result<(Vec<u8>, SocketAddr)> {
        let mut buf = [0; PACKET_BUF_SIZE];
        let mut res = Vec::<u8>::with_capacity(PACKET_BUF_SIZE * 4);

        let addr = loop {
            let (len, addr) = self.socket.recv_from(&mut buf)?;
            res.append(&mut buf[..len].to_vec());
            let tail = &res[res.len() - MSG_END_TAG.len()..];

            if tail == MSG_END_TAG {
                break addr;
            }
        };
        Ok((res[..res.len() - MSG_END_TAG.len()].to_vec(), addr))
    }

    fn push(&self, buf: &[u8]) -> io::Result<()> {
        for buf in [buf, MSG_END_TAG.as_ref()].concat().chunks(PACKET_BUF_SIZE) {
            self.socket.send(buf)?;
        }
        Ok(())
    }

    fn push_to(&self, buf: &[u8], addr: &[SocketAddr]) -> io::Result<()> {
        for buf in [buf, MSG_END_TAG.as_ref()].concat().chunks(PACKET_BUF_SIZE) {
            for addr in addr {
                self.socket.send_to(buf, addr)?;
            }
        }
        Ok(())
    }

    fn get_timeout(&self) -> io::Result<Option<Duration>> {
        self.socket.read_timeout()
    }

    fn set_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.socket.set_read_timeout(dur)
    }

    fn get_ttl(&self) -> io::Result<u32> {
        self.socket.ttl()
    }

    fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.socket.set_ttl(ttl)
    }

    fn get_ext_ip(&self) -> StunResult<SocketAddr> {
        trace!("querying STUN at '{}'", STUN_ADDRESS);
        let stun_addr = STUN_ADDRESS
            .to_socket_addrs()
            .unwrap()
            .find(|x| x.is_ipv4())
            .unwrap();
        let c = StunClient::new(stun_addr);

        c.query_external_address(&self.socket)
    }
}
