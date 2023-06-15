use crate::consts::{MSG_END_TAG, PACKET_MAX_BUF};
use std::io;
use std::net::{SocketAddr, UdpSocket};

use crate::stream::IOStream;

pub(super) struct UdpStream {
    socket: UdpSocket,
}

impl UdpStream {
    #[allow(dead_code)]
    pub fn new(addr: &[SocketAddr]) -> Result<UdpStream, io::Error> {
        Ok(UdpStream {
            socket: UdpSocket::bind(addr)?,
        })
    }
}

impl IOStream for UdpStream {
    fn connect(&self, addr: &[SocketAddr]) -> io::Result<()> {
        self.socket.connect(addr)
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

    fn push(&self, buf: &[u8]) -> io::Result<()> {
        for buf in [buf, MSG_END_TAG.as_ref()].concat().chunks(PACKET_MAX_BUF) {
            self.socket.send(buf)?;
        }
        Ok(())
    }
}
