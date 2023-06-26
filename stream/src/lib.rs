use async_std::channel;
use log::{error, info, trace, warn};
use std::collections::VecDeque;
use std::error;
use wasapi::{Direction, SampleType, ShareMode, WaveFormat};

type BoxError = Box<dyn error::Error>;
type AudioRes<T> = Result<T, BoxError>;

const S_RATE: usize = 48000;
const CHUNK_SIZE: usize = 4096;
const TIMEOUT: u32 = 1000;

pub async fn out_stream(chan: channel::Receiver<Vec<u8>>) -> AudioRes<()> {
    wasapi::initialize_sta()?;
    let device = wasapi::get_default_device(&Direction::Render)?;
    let mut audio_client = device.get_iaudioclient()?;
    let desired_format = WaveFormat::new(32, 32, &SampleType::Float, S_RATE, 2);

    let block_align = desired_format.get_blockalign();
    info!("out: {:?}", desired_format);

    let (def_time, min_time) = audio_client.get_periods()?;
    info!(
        "default audio-client period {}, min period {}",
        def_time, min_time
    );

    audio_client.initialize_client(
        &desired_format,
        min_time,
        &Direction::Render,
        &ShareMode::Shared,
        true,
    )?;

    let h_event = audio_client.set_get_eventhandle()?;

    let mut buffer_frame_count = audio_client.get_bufferframecount()?;
    let capacity = 100 * block_align as usize * (1024 + 2 * buffer_frame_count as usize);

    let render_client = audio_client.get_audiorenderclient()?;
    let mut sample_queue = VecDeque::with_capacity(capacity);
    audio_client.start_stream()?;

    let len = block_align as usize * buffer_frame_count as usize;
    loop {
        buffer_frame_count = audio_client.get_available_space_in_frames()?;
        trace!("buffer write space == '{}' available", buffer_frame_count);
        while sample_queue.len() < len {
            trace!("feeding more samples to buffer");
            match chan.try_recv() {
                Ok(buf) => {
                    trace!("appending data received from rx channel");
                    sample_queue.append(&mut VecDeque::from(buf));
                }
                Err(channel::TryRecvError::Empty) => {
                    warn!("no data received - feeding zeroes to buffer");
                    sample_queue.append(&mut VecDeque::from(vec![0; len - sample_queue.len()]));
                }
                Err(_) => {
                    error!("breaking procedure as channel is closed");
                    break;
                }
            }
        }

        render_client.write_to_device_from_deque(
            buffer_frame_count as usize,
            block_align as usize,
            &mut sample_queue,
            None,
        )?;
        if let e @ Err(_) = h_event.wait_for_event(TIMEOUT) {
            audio_client.stop_stream()?;
            return e;
        }
        trace!(
            "put {:04} frames into out - cycle ended",
            buffer_frame_count
        );
    }
}

pub async fn mic_stream(chan: channel::Sender<Vec<u8>>) -> AudioRes<()> {
    wasapi::initialize_sta()?;
    let device = wasapi::get_default_device(&Direction::Capture)?;
    let mut audio_client = device.get_iaudioclient()?;
    let desired_format = WaveFormat::new(32, 32, &SampleType::Float, S_RATE, 2);

    let block_align = desired_format.get_blockalign();
    info!("mic: {:?}", desired_format);

    let (def_time, min_time) = audio_client.get_periods()?;
    info!(
        "default audio-client period {}, min period {}",
        def_time, min_time
    );

    audio_client.initialize_client(
        &desired_format,
        min_time,
        &Direction::Capture,
        &ShareMode::Shared,
        true,
    )?;

    let h_event = audio_client.set_get_eventhandle()?;

    let buffer_frame_count = audio_client.get_bufferframecount()?;
    let capacity = 100 * block_align as usize * (1024 + 2 * buffer_frame_count as usize);

    let render_client = audio_client.get_audiocaptureclient()?;
    let mut sample_queue = VecDeque::with_capacity(capacity);
    audio_client.start_stream()?;

    let len = block_align as usize * CHUNK_SIZE;
    loop {
        while sample_queue.len() > len {
            trace!("pushing captured samples from buffer to tx channel");
            let chunk = sample_queue.drain(..len).collect::<Vec<_>>();
            chan.try_send(chunk)?;
        }

        let len = sample_queue.len();
        render_client.read_from_device_to_deque(block_align as usize, &mut sample_queue)?;
        if let e @ Err(_) = h_event.wait_for_event(TIMEOUT) {
            audio_client.stop_stream()?;
            return e;
        }
        trace!(
            "got {:04} frames from mic - cycle ended",
            sample_queue.len() - len
        );
    }
}
