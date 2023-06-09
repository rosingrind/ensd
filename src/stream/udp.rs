use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::stream::IOStream;

const MAX_BUF: usize = 256;

pub(super) struct UdpStream {
    socket: UdpSocket,
}

impl UdpStream {
    pub fn new<A: ToSocketAddrs>(addr: &A) -> Result<UdpStream, io::Error> {
        Ok(UdpStream {
            socket: UdpSocket::bind(addr)?,
        })
    }
}

impl IOStream for UdpStream {
    fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        self.socket.connect(addr)
    }

    fn poll(&self) -> io::Result<Vec<u8>> {
        let mut buf = [0; MAX_BUF];

        let len = self.socket.recv(buf.as_mut())?;
        Ok(buf[..len].into())
    }

    fn push(&self, buf: &[u8]) -> io::Result<()> {
        self.socket.send(buf)?;
        Ok(())
    }
}
