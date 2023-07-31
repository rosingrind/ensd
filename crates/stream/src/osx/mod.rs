mod ext;

use err::Result;
use ext::AudioClient;

#[derive(Copy, Clone)]
enum Direction {
    Capture,
    Render,
}

impl From<Direction> for bool {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Capture => true,
            Direction::Render => false,
        }
    }
}

use super::{
    ext::{APISyncGuard, BoxClient, MicChan, OutChan},
    DeviceType, StreamHandle,
};

pub(super) use crate::ext::DeviceBuilder;

#[inline]
async fn mic_loop(
    audio_client: APISyncGuard<AudioClient>,
    chan: APISyncGuard<MicChan>,
) -> Result<()> {
    todo!()
}

#[inline]
async fn out_loop(
    audio_client: APISyncGuard<AudioClient>,
    chan: APISyncGuard<OutChan>,
) -> Result<()> {
    todo!()
}

impl DeviceBuilder for StreamHandle {
    fn new_client(device_type: &DeviceType) -> Result<BoxClient> {
        todo!()
    }
}
