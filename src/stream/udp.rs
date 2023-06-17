use crate::consts::{MSG_END_TAG, PACKET_MAX_BUF};
use log::info;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use crate::stream::IOStream;

pub(super) struct UdpStream {
    socket: UdpSocket,
}

impl UdpStream {
    #[allow(dead_code)]
    pub fn new(addr: &[SocketAddr], dur: Option<Duration>) -> Result<UdpStream, io::Error> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(dur)?;

        Ok(UdpStream { socket })
    }
}

impl IOStream for UdpStream {
    fn bind(&self, addr: &[SocketAddr]) -> io::Result<()> {
        info!(
            "UDP socket at :{} is connecting to {:?}",
            self.socket.local_addr()?.port(),
            addr
        );
        self.socket.connect(addr)
    }

    fn peer(&self) -> io::Result<SocketAddr> {
        self.socket.peer_addr()
    }

    fn poll(&self) -> io::Result<Vec<u8>> {
        let mut buf = [0; PACKET_MAX_BUF];
        let mut res = Vec::<u8>::with_capacity(PACKET_MAX_BUF * 4);

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
        let mut buf = [0; PACKET_MAX_BUF];
        let mut res = Vec::<u8>::with_capacity(PACKET_MAX_BUF * 4);

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
        for buf in [buf, MSG_END_TAG.as_ref()].concat().chunks(PACKET_MAX_BUF) {
            self.socket.send(buf)?;
        }
        Ok(())
    }

    fn push_to(&self, buf: &[u8], addr: &[SocketAddr]) -> io::Result<()> {
        for buf in [buf, MSG_END_TAG.as_ref()].concat().chunks(PACKET_MAX_BUF) {
            for addr in addr {
                self.socket.send_to(buf, addr)?;
            }
        }
        Ok(())
    }
}
