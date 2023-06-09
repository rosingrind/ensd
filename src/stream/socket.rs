use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::stream::IOStream;

const MAX_BUF: usize = 250;

pub(super) struct Socket {
    socket: UdpSocket,
}

// TODO: implement address struct?
impl Socket {
    pub fn new<A: ToSocketAddrs>(addr: &A) -> Result<Socket, io::Error> {
        match UdpSocket::bind(addr) {
            Ok(socket) => Ok(Socket { socket }),
            Err(err) => Err(err),
        }
    }
}

impl IOStream for Socket {
    fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        self.socket.connect(addr)
    }

    fn poll(&self) -> io::Result<Vec<u8>> {
        let mut buf = [0; MAX_BUF];

        match self.socket.recv(buf.as_mut()) {
            Ok(len) => Ok(buf[..len].into()),
            Err(err) => Err(err),
        }
    }

    fn push(&self, buf: &[u8]) -> io::Result<()> {
        match self.socket.send(buf) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
