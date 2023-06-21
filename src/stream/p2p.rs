use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::prelude::FutureExt;
use async_trait::async_trait;
use log::{trace, warn};
use std::io;

use crate::consts::{P2P_REQ_TAG, REQUEST_MSG_DUR, REQUEST_MSG_TTL};
use crate::stream::err::{ERR_CONNECTION, ERR_PIPE_BROKE, ERR_VALIDATION};
use crate::stream::StreamHandle;

#[async_trait(?Send)]
trait SocketAddrsUtil {
    async fn contains(&self, dest: &SocketAddr) -> bool;
}

#[async_trait(?Send)]
impl<T: ToSocketAddrs> SocketAddrsUtil for &T {
    async fn contains(&self, dest: &SocketAddr) -> bool {
        self.to_socket_addrs()
            .await
            .unwrap()
            .find(|c| c == dest)
            .is_some()
    }
}

#[async_trait(?Send)]
pub(super) trait P2P {
    async fn try_nat_tr<A: ToSocketAddrs>(&self, addr: &A, retries: u16) -> io::Result<()>;
}

#[async_trait(?Send)]
impl P2P for StreamHandle {
    async fn try_nat_tr<A: ToSocketAddrs>(&self, addr: &A, retries: u16) -> io::Result<()> {
        let ttl = self.socket.get_ttl().await?;
        self.socket.set_ttl(REQUEST_MSG_TTL).await?;

        let stage_a = [b"a\0".as_ref(), P2P_REQ_TAG].concat();
        let stage_b = [b"b\0".as_ref(), P2P_REQ_TAG].concat();
        let stage_c = [b"c\0".as_ref(), P2P_REQ_TAG].concat();

        trace!(
            "hole punching to {:?}",
            addr.to_socket_addrs()
                .await
                .unwrap()
                .map(|c| c as SocketAddr)
                .collect::<Vec<_>>()
        );

        let mut msg = stage_a.clone();
        let mut iter = (0..retries).into_iter();

        let res = loop {
            self.push_to(&msg, addr).await?;
            if let Ok((res, dest)) = self.poll_at().await {
                if addr.contains(&dest).await {
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
                        _ => break Err(ERR_VALIDATION),
                    }
                    iter = (0..retries).into_iter()
                }
            }
            if iter.next().is_none() {
                break Err(ERR_CONNECTION);
            }
        };

        let res = match res {
            Ok(_) => loop {
                let res = self.peek_at().timeout(REQUEST_MSG_DUR.unwrap()).await;
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
        res.map_err(|e| e.into())
    }
}
