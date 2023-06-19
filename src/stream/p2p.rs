use async_trait::async_trait;
use log::trace;
use std::io;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};

use crate::consts::{TRAVERSAL_DUR, TRAVERSAL_TTL, P2P_REQ_TAG};
use crate::stream::StreamHandle;

#[async_trait]
pub(super) trait P2P {
    async fn try_nat_tr<A: ToSocketAddrs + Sync>(&self, addr: &A, retries: u8) -> io::Result<()>;
}

#[async_trait]
impl P2P for StreamHandle {
    async fn try_nat_tr<A: ToSocketAddrs + Sync>(&self, addr: &A, retries: u8) -> io::Result<()> {
        let ttl = self.socket.get_ttl()?;
        let dur = self.socket.get_timeout()?;
        self.socket.set_ttl(TRAVERSAL_TTL)?;
        self.socket.set_timeout(TRAVERSAL_DUR)?;
        trace!(
            "hole punching to {:?}",
            addr.to_socket_addrs()
                .unwrap()
                .map(|c| c as SocketAddr)
                .collect::<Vec<_>>()
        );

        let mut iter = (0..retries).into_iter();
        let res = loop {
            self.push_to(P2P_REQ_TAG, addr).await?;
            if let Ok(res) = self.poll_at().await {
                if res.0 == P2P_REQ_TAG
                    && addr
                        .to_socket_addrs()
                        .unwrap()
                        .find(|c| *c == res.1)
                        .is_some()
                {
                    break Ok(());
                }
            }
            if iter.next().is_none() {
                break Err(Error::new(ErrorKind::TimedOut, "can't reach remote host"));
            }
        };
        self.socket.set_ttl(ttl)?;
        self.socket.set_timeout(dur)?;
        res
    }
}
