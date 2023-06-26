use ::cipher::{AppRng, CipherHandle, Encryption, SeedableRng};
use ::socket::{SocketHandle, LOOPBACK_IP};
use async_std::{
    fs, io,
    io::WriteExt,
    net::{AddrParseError, IpAddr, SocketAddr},
    path::Path,
    task,
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, FrameCount, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
};
use dasp_interpolate::sinc::Sinc;
use dasp_signal::{self as signal, Signal};
use log::{debug, error, info, trace};
use serde::Deserialize;
use std::{
    env, mem,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        Arc,
    },
};

const RESOURCES_PATH: &str = "res";
const SINC_RING_SIZE: usize = 2;
const UNICODE_WHITE_SQUARE: char = '\u{25A0}';
const UNICODE_BLACK_SQUARE: char = '\u{25A1}';
const PACKET_BUF_SIZE: usize = 256;
const CPAL_BUF_SIZE: FrameCount = (PACKET_BUF_SIZE * 8) as FrameCount;
const SOFTWARE_TAG: Option<&str> = Some("ensd");
const TRANSPORT_SRATE: u64 = 32000;

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

// FIXME: terminate CPAL as it's NOT allocating required buf size
fn parse_cfg(cfg: &SupportedStreamConfig) -> StreamConfig {
    let frames = match cfg.buffer_size() {
        SupportedBufferSize::Range { max, .. } => *max.min(&CPAL_BUF_SIZE),
        SupportedBufferSize::Unknown => CPAL_BUF_SIZE,
    };

    StreamConfig {
        channels: cfg.channels(),
        sample_rate: cfg.sample_rate(),
        buffer_size: BufferSize::Fixed(frames),
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

async fn request_named_remote(name: &str) -> Result<SocketAddr, AddrParseError> {
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

    let host_id = *cpal::available_hosts().first().unwrap();
    let host = cpal::host_from_id(host_id).unwrap();

    let mic_device = host.default_input_device().unwrap();
    let out_device = host.default_output_device().unwrap();

    let mic_cfg = mic_device.default_input_config().unwrap();
    let out_cfg = out_device.default_output_config().unwrap();

    let mic_cfg = parse_cfg(&mic_cfg);
    let out_cfg = parse_cfg(&out_cfg);

    debug!("mic: {:?}", mic_cfg);
    debug!("out: {:?}", out_cfg);

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

    let (tx, rx) = mpsc::channel::<Vec<f32>>();

    // encode to 44100, encrypt and send
    task::spawn(snd_put_loop(
        cipher.clone(),
        snd_stream.clone(),
        rx,
        mic_cfg.sample_rate.0,
    ));

    // collect samples to buf
    let stream_a = {
        let src_chan = mic_cfg.channels;
        let fn_data = move |a: &[f32], _: &_| {
            let buf = a
                .chunks(src_chan as usize)
                .map(|c| c.iter().map(|s| *s / c.len() as f32).sum::<f32>())
                .collect::<Vec<_>>();

            tx.send(buf).unwrap();
        };
        let fn_err = |e: _| error!("an error occurred on stream: {}", e);
        mic_device
            .build_input_stream(&mic_cfg.clone(), fn_data, fn_err, None)
            .unwrap()
    };

    let (tx, rx) = mpsc::channel::<Vec<f32>>();

    // get samples, decrypt and decode to tgt_rate
    task::spawn(snd_get_loop(
        cipher.clone(),
        snd_stream.clone(),
        tx,
        out_cfg.sample_rate.0,
    ));

    // collect and play samples
    let stream_b = {
        let tgt_chan = out_cfg.channels;
        let fn_data = move |a: &mut [f32], _: &_| {
            let buf = rx.try_recv();
            if buf.is_err() {
                return;
            }
            let buf = buf.unwrap().into_iter();
            for (sample, channel_group) in buf.zip(a.chunks_mut(tgt_chan as usize)) {
                for channel in channel_group.iter_mut() {
                    *channel = sample;
                }
            }
        };
        let fn_err = |e: _| error!("an error occurred on stream: {}", e);
        out_device
            .build_output_stream(&out_cfg.clone(), fn_data, fn_err, None)
            .unwrap()
    };

    stream_a.play().unwrap();
    stream_b.play().unwrap();

    task::spawn(msg_put_loop(
        cipher.clone(),
        msg_stream.clone(),
        format!("[{UNICODE_WHITE_SQUARE}] TX: "),
    ));

    task::spawn(msg_get_loop(
        cipher.clone(),
        msg_stream.clone(),
        format!("[{UNICODE_WHITE_SQUARE}] TX: "),
    ));

    #[allow(clippy::empty_loop)]
    loop {}
}

#[inline]
async fn msg_put_loop(cipher: Arc<CipherHandle>, stream: Arc<SocketHandle>, prompt: String) {
    loop {
        let mut buf = String::new();
        let mut out = io::stdout();
        let deletion = prompt.chars().map(|_| '\u{8}').collect::<String>();
        out.write_all(deletion.as_bytes()).await.unwrap();

        out.write_all(prompt.as_bytes()).await.unwrap();
        out.flush().await.unwrap();
        drop(out);
        io::stdin().read_line(&mut buf).await.unwrap();

        let res = stream
            .push(
                cipher
                    .encrypt(buf.trim().as_ref())
                    .await
                    .unwrap()
                    .as_slice(),
            )
            .await;
        if res.is_err() {
            error!("failed to push packets with text data");
            continue;
        }

        log::logger().flush();
    }
}

#[inline]
async fn snd_put_loop(
    cipher: Arc<CipherHandle>,
    stream: Arc<SocketHandle>,
    rx: Receiver<Vec<f32>>,
    src_rate: u32,
) {
    loop {
        let buf = rx.recv().unwrap();
        let len = buf.len();
        let _a = *buf.first().unwrap_or(&0.0);
        let _b = *buf.last().unwrap_or(&0.0);

        let source = signal::from_iter(buf.clone().into_iter());
        let sinc = source.take(SINC_RING_SIZE).collect::<Vec<_>>();
        let source = signal::from_iter(buf.into_iter());
        let len = (len as f32 * (TRANSPORT_SRATE as f32 / src_rate as f32)) as usize;
        let res = source
            .from_hz_to_hz(
                Sinc::new(sinc.into()),
                src_rate as f64,
                TRANSPORT_SRATE as f64,
            )
            .until_exhausted()
            .take(len)
            .map(|c| c.to_ne_bytes())
            .collect::<Vec<_>>()
            .concat();

        let res = stream
            .push(cipher.encrypt(res.as_ref()).await.unwrap().as_ref())
            .await;

        if res.is_err() {
            error!("failed to push packets with audio data");
            continue;
        }
    }
}

#[inline]
async fn msg_get_loop(cipher: Arc<CipherHandle>, stream: Arc<SocketHandle>, prompt: String) {
    loop {
        let buf = task::block_on(stream.poll()).unwrap();
        let mut out = io::stdout();
        trace!("RX-CRYPT*: '{:?}'", buf);
        let deletion = prompt.chars().map(|_| '\u{8}').collect::<String>();
        out.write_all(deletion.as_bytes()).await.unwrap();

        let msg = format!(
            "[{UNICODE_BLACK_SQUARE}] RX: '{}'\n\n",
            String::from_utf8(task::block_on(cipher.decrypt(buf.as_ref())).unwrap()).unwrap()
        );
        out.write_all(msg.as_ref()).await.unwrap();
        out.write_all(prompt.as_bytes()).await.unwrap();
        out.flush().await.unwrap();
        drop(out);

        log::logger().flush();
    }
}

#[inline]
async fn snd_get_loop(
    cipher: Arc<CipherHandle>,
    stream: Arc<SocketHandle>,
    tx: Sender<Vec<f32>>,
    tgt_rate: u32,
) {
    loop {
        let res = cipher.decrypt(stream.poll().await.unwrap().as_ref()).await;
        let buf = res;
        if buf.is_err() {
            error!("failed to decrypt packets with audio data");
            continue;
        }
        let buf = buf
            .unwrap()
            .chunks(mem::size_of::<f32>())
            .map(|c| f32::from_ne_bytes(<[u8; mem::size_of::<f32>()]>::try_from(c).unwrap()))
            .collect::<Vec<_>>();
        let len = buf.len();
        let _a = *buf.first().unwrap_or(&0.0);
        let _b = *buf.last().unwrap_or(&0.0);
        let source = signal::from_iter(buf.clone().into_iter());
        let sinc = source.take(SINC_RING_SIZE).collect::<Vec<_>>();
        let source = signal::from_iter(buf.into_iter());
        let len = (len as f32 * (tgt_rate as f32 / TRANSPORT_SRATE as f32)) as usize;
        let res = source
            .from_hz_to_hz(
                Sinc::new(sinc.into()),
                TRANSPORT_SRATE as f64,
                tgt_rate as f64,
            )
            .until_exhausted()
            .take(len)
            .collect::<Vec<_>>();

        tx.send(res).unwrap();
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
