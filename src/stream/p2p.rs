use async_trait::async_trait;
use log::{trace, warn};
use std::io;
use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};

use crate::consts::{P2P_REQ_TAG, TRAVERSAL_MSG_DUR, TRAVERSAL_MSG_TTL};
use crate::stream::StreamHandle;

trait SocketAddrsUtil {
    fn contains(&self, dest: &SocketAddr) -> bool;
}

impl<T: ToSocketAddrs> SocketAddrsUtil for &T {
    fn contains(&self, dest: &SocketAddr) -> bool {
        self.to_socket_addrs()
            .unwrap()
            .find(|c| c == dest)
            .is_some()
    }
}

#[async_trait]
pub(super) trait P2P {
    async fn try_nat_tr<A: ToSocketAddrs + Sync>(&self, addr: &A, retries: u16) -> io::Result<()>;
}

#[async_trait]
impl P2P for StreamHandle {
    async fn try_nat_tr<A: ToSocketAddrs + Sync>(&self, addr: &A, retries: u16) -> io::Result<()> {
        let ttl = self.socket.get_ttl()?;
        let dur = self.socket.get_timeout()?;
        self.socket.set_ttl(TRAVERSAL_MSG_TTL)?;
        self.socket.set_timeout(TRAVERSAL_MSG_DUR)?;

        let err_connection = Error::new(
            ErrorKind::TimedOut,
            format!("can't reach remote host in {retries} attempts"),
        );

        let err_validation = Error::new(
            ErrorKind::InvalidInput,
            "remote host returned non-stage message",
        );

        let err_pipe_broke = Error::new(
            ErrorKind::BrokenPipe,
            "incorrect traversal procedure ordering",
        );

        let stage_a = [b"a\0".as_ref(), P2P_REQ_TAG].concat();
        let stage_b = [b"b\0".as_ref(), P2P_REQ_TAG].concat();
        let stage_c = [b"c\0".as_ref(), P2P_REQ_TAG].concat();

        trace!(
            "hole punching to {:?}",
            addr.to_socket_addrs()
                .unwrap()
                .map(|c| c as SocketAddr)
                .collect::<Vec<_>>()
        );

        let mut msg = stage_a.clone();
        let mut iter = (0..retries).into_iter();

        let res = loop {
            self.push_to(&msg, addr).await?;
            if let Ok((res, dest)) = self.poll_at().await {
                if addr.contains(&dest) {
                    match res {
                        res if res == stage_a => msg = stage_b.clone(),
                        res if res == stage_b => msg = stage_c.clone(),
                        res if res == stage_c => {
                            if msg == stage_c {
                                self.push_to(&msg, addr).await?;
                                break Ok(());
                            } else {
                                msg = stage_c.clone()
                            }
                        }
                        _ => break Err(err_validation),
                    }
                    iter = (0..retries).into_iter()
                }
            }
            if iter.next().is_none() {
                break Err(err_connection);
            }
        };

        let res = match res {
            Ok(_) => loop {
                let res = self.peek_at().await;
                if res.is_err() {
                    break Ok(());
                }
                let (res, dest) = res?;
                if addr.contains(&dest) {
                    match res {
                        res if res == stage_a || res == stage_b => break Err(err_pipe_broke),
                        res if res == stage_c => {
                            self.poll().await?;
                            warn!("received a message after hole punching stages");
                        }
                        _ => break Ok(()),
                    }
                }
            },
            res => res,
        };
        self.socket.set_ttl(ttl)?;
        self.socket.set_timeout(dur)?;
        res
    }
}
