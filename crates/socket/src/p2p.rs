use async_std::{
    future,
    net::{SocketAddr, ToSocketAddrs},
};
use async_trait::async_trait;
use howler::{
    consts::{ERR_CONNECTION, ERR_PIPE_BROKE, ERR_VALIDATION},
    Error, Result,
};
use log::{error, info, trace, warn};
use std::time::Duration;

use crate::SocketHandle;

const P2P_REQ_TAG: &[u8] = b"p2p\0req\0";
const REQUEST_MSG_TTL: u32 = 32;

#[async_trait(?Send)]
trait SocketAddrsUtil {
    async fn contains(&self, dest: &SocketAddr) -> bool;
}

#[async_trait(?Send)]
impl<T: ToSocketAddrs> SocketAddrsUtil for &T {
    async fn contains(&self, dest: &SocketAddr) -> bool {
        self.to_socket_addrs().await.unwrap().any(|c| c == *dest)
    }
}

#[async_trait(?Send)]
pub(super) trait P2P {
    async fn try_nat_tr<A: ToSocketAddrs>(
        &self,
        addr: &A,
        retries: u16,
        timeout: Duration,
    ) -> Result<()>;
}

// TODO: add NAT type check, see https://github.com/Azure/RDS-Templates/tree/master/AVD-TestShortpath
#[async_trait(?Send)]
impl P2P for SocketHandle {
    async fn try_nat_tr<A: ToSocketAddrs>(
        &self,
        addr: &A,
        retries: u16,
        timeout: Duration,
    ) -> Result<()> {
        let ttl = self.socket.get_ttl().await?;
        self.socket.set_ttl(REQUEST_MSG_TTL).await?;

        let stage_a = [b"a\0".as_ref(), P2P_REQ_TAG].concat();
        let stage_b = [b"b\0".as_ref(), P2P_REQ_TAG].concat();
        let stage_c = [b"c\0".as_ref(), P2P_REQ_TAG].concat();

        info!(
            "hole punching to {:?}",
            addr.to_socket_addrs()
                .await
                .unwrap()
                .map(|c| c as SocketAddr)
                .collect::<Vec<_>>()
        );

        let mut msg = &stage_a;
        let mut iter = 0..retries;

        let res = loop {
            self.push_to(msg, addr).await?;
            if let Ok((res, dest)) = self.poll_at().await {
                if addr.contains(&dest).await {
                    match res {
                        res if res == stage_a => {
                            trace!("moving up sync to 'stage_b' as got 'stage_a' message");
                            msg = &stage_b
                        }
                        res if res == stage_b => {
                            trace!("moving up sync to 'stage_c' as got 'stage_b' message");
                            msg = &stage_c
                        }
                        res if res == stage_c => {
                            if msg == &stage_c {
                                trace!("sync of 'stage_c' done - beginning socket buffer cleanup");
                                self.push_to(msg, addr).await?;
                                break Ok(());
                            } else {
                                trace!("got 'stage_c' message out of order - syncing 'msg'");
                                msg = &stage_c
                            }
                        }
                        _ => break Err(ERR_VALIDATION),
                    }
                    iter = 0..retries
                }
            }

            if iter.next().is_none() {
                error!("failed {} retries - can't establish connection", retries);
                break Err(ERR_CONNECTION);
            }
        };

        let res = match res {
            Ok(_) => loop {
                let res = future::timeout(timeout, self.peek_at()).await;
                if res.is_err() {
                    break Ok(());
                }
                let (res, dest) = res.unwrap()?;
                if addr.contains(&dest).await {
                    match res {
                        res if res == stage_a || res == stage_b => break Err(ERR_PIPE_BROKE),
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

        self.socket.set_ttl(ttl).await?;
        res.map_err(Error::into)
    }
}
