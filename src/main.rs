mod cipher;
mod consts;
mod stream;

use aead::rand_core::SeedableRng;
use async_std::{
    net::{AddrParseError, IpAddr, SocketAddr},
    task,
};
use consts::{RESOURCES_PATH, SINC_RING_SIZE};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, StreamConfig, SupportedBufferSize, SupportedStreamConfig,
};
use dasp_interpolate::sinc::Sinc;
use dasp_signal::{self as signal, Signal};
use futures::try_join;
use log::{debug, error, info, trace};
use log4rs;
use serde::Deserialize;
use std::{
    env, fs, io,
    io::Write,
    mem,
    path::Path,
    sync::{mpsc, Arc},
    thread,
};

use crate::cipher::{AesNonce, AesSpec, AppRng, ChaSpec, CipherHandle};
use crate::consts::{BLACK_SQUARE, CPAL_BUF_SIZE, SOFTWARE_TAG, TRANSPORT_SRATE, WHITE_SQUARE};
use crate::stream::StreamHandle;

#[derive(Debug, Deserialize)]
struct Config {
    encryption: Encryption,
    msg_addr: UDP,
    snd_addr: UDP,
}

#[derive(Debug, Deserialize)]
struct UDP {
    ip: IpAddr,
    port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Encryption {
    AES {
        #[serde(default)]
        cipher: AesSpec,
        #[serde(default)]
        nonce: AesNonce,
    },
    ChaCha {
        #[serde(default)]
        cipher: ChaSpec,
    },
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

fn request_phrase() -> String {
    let mut out = io::stdout().lock();
    out.write(format!("[{WHITE_SQUARE}] enter seed phrase: ").as_ref())
        .unwrap();
    out.flush().unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).unwrap();

    phrase
}

fn request_named_remote(name: &str) -> Result<SocketAddr, AddrParseError> {
    let mut out = io::stdout().lock();
    out.write(format!("[{WHITE_SQUARE}] enter remote addr for socket '{name}': ").as_ref())
        .unwrap();
    out.flush().unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).unwrap();

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
    let conf = toml::from_str::<Config>(&*fs::read_to_string(path).unwrap()).unwrap();

    debug!("{:?}", conf.encryption);

    let cipher = Arc::new(CipherHandle::new(
        &conf.encryption,
        AppRng::from_seed(request_phrase().into()),
    ));

    let msg_addr = SocketAddr::new(conf.msg_addr.ip, conf.msg_addr.port);
    let snd_addr = SocketAddr::new(conf.snd_addr.ip, conf.snd_addr.port);

    let msg_stream = Arc::new(StreamHandle::new(&msg_addr, None, SOFTWARE_TAG).await);
    let snd_stream = Arc::new(StreamHandle::new(&snd_addr, None, SOFTWARE_TAG).await);

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
        let msg_remote = "127.0.0.1:34254".parse().unwrap();
        let snd_remote = "127.0.0.1:34054".parse().unwrap();
        (msg_remote, snd_remote)
    } else {
        let msg_remote = loop {
            match request_named_remote("msg") {
                Ok(res) => break res,
                Err(e) => error!("invalid address: {e}"),
            }
        };
        let snd_remote = loop {
            match request_named_remote("snd") {
                Ok(res) => break res,
                Err(e) => error!("invalid address: {e}"),
            }
        };
        (msg_remote, snd_remote)
    };

    try_join!(msg_stream.bind(&msg_remote), snd_stream.bind(&snd_remote)).unwrap();

    println!();

    let (tx, rx) = mpsc::channel::<Vec<f32>>();

    // encode to 44100, encrypt and send
    {
        let io_cipher = cipher.clone();
        let snd_put = snd_stream.clone();
        let src_rate = mic_cfg.sample_rate.0;
        thread::spawn(move || loop {
            let buf = rx.recv().unwrap();
            let len = buf.len();
            let _a = *buf.first().unwrap_or(&0.0);
            let _b = *buf.last().unwrap_or(&0.0);

            let source = signal::from_iter(buf.clone().into_iter());
            let sinc = source.take(SINC_RING_SIZE).collect::<Vec<_>>();
            let source = signal::from_iter(buf.into_iter());
            let len = (len as f32 * (TRANSPORT_SRATE as f32 / src_rate as f32)) as usize;
            let res = source // Sinc::new([0.0; 16].into()) Linear::new(_a, _b)
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

            let res = task::block_on(async {
                // FIXME
                snd_put
                    .push(io_cipher.encrypt(res.as_ref()).await.unwrap().as_ref())
                    .await
            });

            if res.is_err() {
                error!("failed to push packets with audio data");
                continue;
            }
            task::spawn(async { log::logger().flush() });
        })
    };

    // collect samples to buf
    let stream_a = {
        let src_chan = mic_cfg.channels;
        let fn_data = move |a: &[f32], _: &_| {
            let buf = a
                .chunks(src_chan as usize)
                .map(|c| c.into_iter().map(|s| *s / c.len() as f32).sum::<f32>())
                .collect::<Vec<_>>();

            tx.send(buf).unwrap();
        };
        let fn_err = |e: _| error!("an error occurred on stream: {}", e);
        mic_device
            .build_input_stream(&mic_cfg.clone().into(), fn_data, fn_err, None)
            .unwrap()
    };

    let (tx, rx) = mpsc::channel::<Vec<f32>>();

    // get samples, decrypt and decode to tgt_rate
    {
        let tgt_rate = out_cfg.sample_rate.0;
        let io_cipher = cipher.clone();
        let snd_get = snd_stream.clone();
        thread::spawn(move || loop {
            let res = task::block_on(async {
                // FIXME
                io_cipher
                    .decrypt(snd_get.poll().await.unwrap().as_ref())
                    .await
            });
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
            task::spawn(async { log::logger().flush() });
        })
    };

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
            .build_output_stream(&out_cfg.clone().into(), fn_data, fn_err, None)
            .unwrap()
    };

    stream_a.play().unwrap();
    stream_b.play().unwrap();

    let prompt = Arc::new(format!("[{WHITE_SQUARE}] TX: "));

    let cipher_a = cipher.clone();
    let prompt_a = prompt.clone();
    let msg_put = msg_stream.clone();
    thread::spawn(move || loop {
        let mut buf = String::new();
        let mut out = io::stdout().lock();
        let deletion = prompt_a.chars().map(|_| '\u{8}').collect::<String>();
        out.write(deletion.as_bytes()).unwrap();

        out.write(prompt_a.as_bytes()).unwrap();
        out.flush().unwrap();
        drop(out);
        io::stdin().read_line(&mut buf).unwrap();

        let res = task::block_on(async {
            // FIXME
            msg_put
                .push(
                    cipher_a
                        .encrypt(buf.trim().as_ref())
                        .await
                        .unwrap()
                        .as_slice(),
                )
                .await
        });
        if res.is_err() {
            error!("failed to push packets with text data");
            continue;
        }
    });

    let cipher_b = cipher.clone();
    let prompt_b = prompt.clone();
    let msg_get = msg_stream.clone();
    thread::spawn(move || loop {
        let buf = task::block_on(msg_get.poll()).unwrap();
        let mut out = io::stdout().lock();
        trace!("RX-CRYPT*: '{:?}'", buf);
        let deletion = prompt_b.chars().map(|_| '\u{8}').collect::<String>();
        out.write(deletion.as_bytes()).unwrap();

        out.write(
            format!(
                "[{BLACK_SQUARE}] RX: '{}'\n",
                String::from_utf8(task::block_on(cipher_b.decrypt(buf.as_ref())).unwrap()).unwrap()
            )
            .as_ref(),
        )
        .unwrap();
        out.write(b"\n").unwrap();
        out.write(prompt_b.as_bytes()).unwrap();
        out.flush().unwrap();
        drop(out)
    });

    loop {}
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::consts::RESOURCES_PATH;
    use crate::Config;

    #[test]
    fn config_is_valid() {
        let path = Path::new(RESOURCES_PATH).join("cfg.toml");
        toml::from_str::<Config>(&*fs::read_to_string(path).unwrap()).unwrap();
    }
}
