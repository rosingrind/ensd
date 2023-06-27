mod err;

use async_std::{
    channel, fs, io,
    io::WriteExt,
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::Arc,
    task,
};
use common::{
    cipher::{AppRng, CipherHandle, Encryption, SeedableRng},
    socket::{SocketHandle, LOOPBACK_IP},
    stream::{mic_stream, out_stream},
};
use futures::TryFutureExt;
use log::{debug, error, info, trace};
use serde::Deserialize;
use std::env;

use crate::err::Error;

const RESOURCES_PATH: &str = "res";
const UNICODE_WHITE_SQUARE: char = '\u{25A0}';
const UNICODE_BLACK_SQUARE: char = '\u{25A1}';
const SOFTWARE_TAG: Option<&str> = Some("ensd");

#[derive(Debug, Deserialize)]
struct Config {
    encryption: Encryption,
    msg_addr: UDP,
    snd_addr: UDP,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Deserialize)]
struct UDP {
    ip: IpAddr,
    port: u16,
}

pub type Result<T> = core::result::Result<T, Error>;

async fn request_phrase() -> String {
    let msg = format!("[{UNICODE_WHITE_SQUARE}] enter seed phrase: ");
    let mut out = io::stdout();
    out.write_all(msg.as_ref()).await.unwrap();
    out.flush().await.unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).await.unwrap();

    phrase
}

async fn request_named_remote(
    name: &str,
) -> core::result::Result<SocketAddr, impl std::error::Error> {
    let msg = format!("[{UNICODE_WHITE_SQUARE}] enter remote addr for socket '{name}': ");
    let mut out = io::stdout();
    out.write_all(msg.as_ref()).await.unwrap();
    out.flush().await.unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).await.unwrap();

    phrase.trim().parse()
}

#[async_std::main]
async fn main() {
    let path = Path::new(RESOURCES_PATH).join("log.toml");
    log4rs::init_file(path, Default::default()).unwrap();

    let path = Path::new(RESOURCES_PATH).join("cfg.toml");
    let conf = toml::from_str::<Config>(&fs::read_to_string(path).await.unwrap()).unwrap();

    debug!("{:?}", conf.encryption);

    let cipher = Arc::new(CipherHandle::new(
        &conf.encryption,
        AppRng::from_seed(request_phrase().await.into()),
    ));

    let msg_addr = SocketAddr::new(conf.msg_addr.ip, conf.msg_addr.port);
    let snd_addr = SocketAddr::new(conf.snd_addr.ip, conf.snd_addr.port);

    let msg_stream = Arc::new(SocketHandle::new(&msg_addr, None, SOFTWARE_TAG).await);
    let snd_stream = Arc::new(SocketHandle::new(&snd_addr, None, SOFTWARE_TAG).await);

    info!(
        "socket 'msg' at :{} extern address: {:?}",
        msg_addr.port(),
        msg_stream.pub_ip.unwrap_or(msg_addr)
    );
    info!(
        "socket 'snd' at :{} extern address: {:?}",
        snd_addr.port(),
        snd_stream.pub_ip.unwrap_or(snd_addr)
    );

    let args = env::args().collect::<Vec<String>>();
    let arg_mode = args.get(1).map(|c| c.trim());

    let (msg_remote, snd_remote) = if let Some("loopback") = arg_mode {
        let msg_remote = SocketAddr::new(LOOPBACK_IP, conf.msg_addr.port);
        let snd_remote = SocketAddr::new(LOOPBACK_IP, conf.snd_addr.port);
        (msg_remote, snd_remote)
    } else {
        let msg_remote = loop {
            match request_named_remote("msg").await {
                Ok(res) => break res,
                Err(e) => error!("invalid address: {e}"),
            }
        };
        let snd_remote = loop {
            match request_named_remote("snd").await {
                Ok(res) => break res,
                Err(e) => error!("invalid address: {e}"),
            }
        };
        (msg_remote, snd_remote)
    };

    futures::try_join!(msg_stream.bind(&msg_remote), snd_stream.bind(&snd_remote)).unwrap();

    println!();

    let (tx, rx) = channel::unbounded();

    // encode to 44100, encrypt and send
    let t1 = task::spawn(snd_put_loop(cipher.clone(), snd_stream.clone(), rx));

    // collect samples to buf
    let t2 = task::spawn(mic_stream(tx)).map_err(|e| e.into());

    let (tx, rx) = channel::unbounded();

    // get samples, decrypt and decode to tgt_rate
    let t3 = task::spawn(snd_get_loop(cipher.clone(), snd_stream.clone(), tx));

    // collect and play samples
    let t4 = task::spawn(out_stream(rx)).map_err(|e| e.into());

    let t5 = task::spawn(msg_put_loop(
        cipher.clone(),
        msg_stream.clone(),
        format!("[{UNICODE_WHITE_SQUARE}] TX: "),
    ));

    let t6 = task::spawn(msg_get_loop(
        cipher.clone(),
        msg_stream.clone(),
        format!("[{UNICODE_WHITE_SQUARE}] TX: "),
    ));

    futures::try_join!(t1, t2, t3, t4, t5, t6).unwrap();
}

#[inline]
async fn msg_put_loop(
    cipher: Arc<CipherHandle>,
    stream: Arc<SocketHandle>,
    prompt: String,
) -> Result<()> {
    loop {
        let mut buf = String::new();
        let mut out = io::stdout();
        let del = prompt.chars().map(|_| '\u{8}').collect::<String>();

        out.write_all(del.as_bytes()).await?;
        out.write_all(prompt.as_bytes()).await?;
        out.flush().await?;
        drop(out);

        io::stdin().read_line(&mut buf).await?;

        match cipher.encrypt(buf.trim().as_ref()).await {
            Ok(msg) => {
                if let Err(e) = stream.push(msg.as_slice()).await {
                    error!("failed to push packets with text data: {e}")
                }
            }
            Err(e) => error!("failed to encrypt data for network stream: {e}"),
        }

        log::logger().flush();
    }
}

#[inline]
async fn snd_put_loop(
    cipher: Arc<CipherHandle>,
    stream: Arc<SocketHandle>,
    rx: channel::Receiver<Vec<u8>>,
) -> Result<()> {
    loop {
        match rx.recv().await {
            Ok(res) => match cipher.encrypt(res.as_ref()).await {
                Ok(res) => {
                    if let Err(err) = stream.push(res.as_ref()).await {
                        error!("failed to push packets with audio data: {err}");
                    }
                }
                Err(err) => error!("failed to encrypt data received from channel: {err}"),
            },
            Err(err) => error!("failed to receive from async channel: {err}"),
        }
    }
}

#[inline]
async fn msg_get_loop(
    cipher: Arc<CipherHandle>,
    stream: Arc<SocketHandle>,
    prompt: String,
) -> Result<()> {
    loop {
        match stream.poll().await {
            Ok(buf) => {
                let mut out = io::stdout();
                trace!("RX-CRYPT*: '{:?}'", buf);

                match cipher.decrypt(buf.as_ref()).await {
                    Ok(msg) => {
                        let del = prompt.chars().map(|_| '\u{8}').collect::<String>();
                        let msg = format!(
                            "[{UNICODE_BLACK_SQUARE}] RX: '{}'\n\n",
                            String::from_utf8(msg)?
                        );
                        out.write_all(del.as_bytes()).await?;
                        out.write_all(msg.as_bytes()).await?;
                        out.write_all(prompt.as_bytes()).await?;
                    }
                    Err(e) => error!("failed to decrypt data from network stream: {e}"),
                }
                out.flush().await?;
                drop(out);
            }
            Err(e) => error!("failed to poll data from network stream: {e}"),
        }
        log::logger().flush();
    }
}

#[inline]
async fn snd_get_loop(
    cipher: Arc<CipherHandle>,
    stream: Arc<SocketHandle>,
    tx: channel::Sender<Vec<u8>>,
) -> Result<()> {
    loop {
        match stream.poll().await {
            Ok(res) => match cipher.decrypt(res.as_ref()).await {
                Ok(res) => {
                    if let Err(err) = tx.send(res).await {
                        error!("failed to send audio data to async channel: {err}");
                    }
                }
                Err(err) => error!("failed to decrypt packets with audio data: {err}"),
            },
            Err(err) => error!("failed to poll data from network stream: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn config_is_valid() {
        let path = Path::new(RESOURCES_PATH).join("cfg.toml");
        toml::from_str::<Config>(&fs::read_to_string(path).await.unwrap()).unwrap();
    }
}
