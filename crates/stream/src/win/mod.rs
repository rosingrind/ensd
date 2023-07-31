mod ext;

use async_std::{
    channel,
    sync::{Arc, Mutex},
    task,
};
use err::{Error, Result};
use ext::{AudioClient, WasapiClient};
use log::{error, info, trace, warn};
use std::collections::VecDeque;
use wasapi::{Direction, SampleType, ShareMode, WaveFormat};

use super::{
    ext::{APISyncGuard, BoxClient, MicChan, OutChan},
    DeviceType, StreamHandle, CHUNK_SIZE, S_RATE, TIMEOUT,
};

pub(super) use crate::ext::DeviceBuilder;

#[inline]
async fn mic_loop(
    audio_client: APISyncGuard<AudioClient>,
    chan: APISyncGuard<MicChan>,
) -> Result<()> {
    wasapi::initialize_sta()?;
    let api = audio_client.lock().await;
    let audio_client = api.get();

    let block_align = audio_client.get_mixformat()?.get_blockalign();
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
            task::block_on(chan.lock()).try_send(chunk)?;
        }

        let len = sample_queue.len();
        render_client.read_from_device_to_deque(block_align as usize, &mut sample_queue)?;
        if let e @ Err(_) = h_event.wait_for_event(TIMEOUT) {
            audio_client.stop_stream()?;
            break e;
        }
        trace!(
            "got {:04} frames from mic - cycle ended",
            sample_queue.len() - len
        );
    }
    .map_err(Error::from)?;
    Ok(wasapi::deinitialize())
}

#[inline]
async fn out_loop(
    audio_client: APISyncGuard<AudioClient>,
    chan: APISyncGuard<OutChan>,
) -> Result<()> {
    wasapi::initialize_sta()?;
    let api = audio_client.lock().await;
    let audio_client = api.get();

    let block_align = audio_client.get_mixformat()?.get_blockalign();
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
            match task::block_on(chan.lock()).try_recv() {
                Ok(buf) => {
                    trace!("appending data received from rx channel");
                    sample_queue.append(&mut VecDeque::from(buf));
                }
                Err(channel::TryRecvError::Empty) => {
                    warn!("no data received - feeding zeroes to buffer");
                    sample_queue.append(&mut VecDeque::from(vec![0; len - sample_queue.len()]));
                }
                e => {
                    error!("breaking procedure as channel is closed");
                    e?;
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
            break e;
        }
        trace!(
            "put {:04} frames into out - cycle ended",
            buffer_frame_count
        );
    }
    .map_err(Error::from)?;
    Ok(wasapi::deinitialize())
}

impl DeviceBuilder for StreamHandle {
    fn new_client(device_type: &DeviceType) -> Result<BoxClient> {
        wasapi::initialize_sta()?;
        let direction = match device_type {
            DeviceType::Mic(_) => Direction::Capture,
            DeviceType::Out(_) => Direction::Render,
        };
        let device = wasapi::get_default_device(&direction)?;
        let mut audio_client = device.get_iaudioclient()?;
        let desired_format = WaveFormat::new(32, 32, &SampleType::Float, S_RATE, 2, None);
        info!("desired audio format: {:?}", desired_format);

        let (def_time, min_time) = audio_client.get_periods()?;
        info!(
            "default audio-client period {}, min period {}",
            def_time, min_time
        );

        audio_client.initialize_client(
            &desired_format,
            min_time,
            &direction,
            &ShareMode::Shared,
            true,
        )?;

        let api = Arc::new(Mutex::new(AudioClient::new(audio_client)));
        let audio_client = WasapiClient::new(api);

        Ok(Box::new(audio_client))
    }
}
