mod err;

use async_std::{channel, fs, io, io::WriteExt, net::SocketAddr, path::Path, sync::Arc, task};
use common::{
    cipher::{AppRng, CipherHandle, Encryption, SeedableRng},
    socket::{Client, SocketConfig, SocketHandle, LOOPBACK_IP},
    stream::{DeviceType, StreamHandle},
};
use log::{debug, error, info, trace};
use serde::Deserialize;
use std::env;
use std::time::Duration;

use crate::err::{Error, Result};

const RESOURCES_PATH: &str = "res";
const UNICODE_WHITE_SQUARE: char = '\u{25A0}';
const UNICODE_BLACK_SQUARE: char = '\u{25A1}';

#[derive(Debug, Deserialize)]
struct Config {
    encryption: Encryption,
    client: ClientConfig,
    socket: SocketConfigRaw,
}

#[derive(Debug, Deserialize)]
struct ClientConfig {
    msg: Client,
    snd: Client,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SocketConfigRaw {
    retries: u16,
    timeout: u64,
}

impl From<SocketConfigRaw> for SocketConfig {
    fn from(value: SocketConfigRaw) -> Self {
        SocketConfig::new(value.retries, Duration::from_millis(value.timeout))
    }
}

async fn request_phrase() -> String {
    let msg = format!("[{UNICODE_WHITE_SQUARE}] enter seed phrase: ");
    let mut out = io::stdout();
    out.write_all(msg.as_ref()).await.unwrap();
    out.flush().await.unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).await.unwrap();

    phrase
}

async fn request_named_remote(name: &str) -> Result<SocketAddr> {
    let msg = format!("[{UNICODE_WHITE_SQUARE}] enter remote addr for socket '{name}': ");
    let mut out = io::stdout();
    out.write_all(msg.as_ref()).await.unwrap();
    out.flush().await.unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).await.unwrap();

    phrase.trim().parse().map_err(Error::from)
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

    let msg_stream = Arc::new(
        SocketHandle::new(conf.client.msg, conf.socket.clone().into())
            .await
            .unwrap(),
    );
    let snd_stream = Arc::new(
        SocketHandle::new(conf.client.snd, conf.socket.into())
            .await
            .unwrap(),
    );

    info!(
        "socket 'msg' at :{} extern address: {:?}",
        msg_stream.loc_ip.port(),
        msg_stream.pub_ip
    );
    info!(
        "socket 'snd' at :{} extern address: {:?}",
        snd_stream.loc_ip.port(),
        snd_stream.pub_ip
    );

    let args = env::args().collect::<Vec<String>>();
    let arg_mode = args.get(1).map(|c| c.trim());

    let (msg_remote, snd_remote) = if let Some("loopback") = arg_mode {
        let msg_remote = SocketAddr::new(LOOPBACK_IP, msg_stream.loc_ip.port());
        let snd_remote = SocketAddr::new(LOOPBACK_IP, snd_stream.loc_ip.port());
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

    let t1 = task::spawn(snd_put_loop(cipher.clone(), snd_stream.clone(), rx));
    let t2 = task::spawn(run_stream(StreamHandle::new(DeviceType::Mic(tx)).unwrap()));

    let (tx, rx) = channel::unbounded();

    let t3 = task::spawn(snd_get_loop(cipher.clone(), snd_stream.clone(), tx));
    let t4 = task::spawn(run_stream(StreamHandle::new(DeviceType::Out(rx)).unwrap()));

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
async fn run_stream(stream: StreamHandle) -> Result<()> {
    stream.play().await.map_err(Error::from)
}

#[inline]
async fn msg_put_loop(
    cipher: Arc<CipherHandle>,
    socket: Arc<SocketHandle>,
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
                if let Err(e) = socket.push(msg.as_slice()).await {
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
    socket: Arc<SocketHandle>,
    rx: channel::Receiver<Vec<u8>>,
) -> Result<()> {
    loop {
        match rx.recv().await {
            Ok(res) => match cipher.encrypt(res.as_ref()).await {
                Ok(res) => {
                    if let Err(err) = socket.push(res.as_ref()).await {
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
    socket: Arc<SocketHandle>,
    prompt: String,
) -> Result<()> {
    loop {
        match socket.poll().await {
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
    socket: Arc<SocketHandle>,
    tx: channel::Sender<Vec<u8>>,
) -> Result<()> {
    loop {
        match socket.poll().await {
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

        let res = fs::read_to_string(path).await;
        // config is a present resource
        assert!(res.is_ok());
        let res = toml::from_str::<Config>(&res.unwrap());
        // config is a valid `.toml`
        assert!(res.is_ok());
    }
}
