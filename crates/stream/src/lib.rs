mod ext;

#[cfg(target_os = "macos")]
mod osx;
#[cfg(target_os = "windows")]
mod win;

use async_std::sync::{Arc, Mutex};
use err::{Error, Result};
use ext::{BoxClient, MicChan, OutChan};
use howler::Result as HowlerResult;

use crate::ext::APISyncGuard;
#[cfg(target_os = "macos")]
use osx::DeviceBuilder;
#[cfg(target_os = "windows")]
use win::DeviceBuilder;

const S_RATE: usize = 32000;
const CHUNK_SIZE: usize = 4096;
const TIMEOUT: u32 = 1000;

pub enum DeviceType {
    Mic(MicChan),
    Out(OutChan),
}

enum ClientType {
    Mic(BoxClient, APISyncGuard<MicChan>),
    Out(BoxClient, APISyncGuard<OutChan>),
}

impl ClientType {
    async fn play(&self) -> Result<()> {
        match self {
            ClientType::Mic(audio_client, chan) => audio_client.play_rec(chan.clone()).await,
            ClientType::Out(audio_client, chan) => audio_client.play_out(chan.clone()).await,
        }
    }
}

pub struct StreamHandle {
    client: ClientType,
}

impl StreamHandle {
    pub fn new(device_type: DeviceType) -> HowlerResult<Self> {
        let client = StreamHandle::new_client(&device_type)?;

        Ok(StreamHandle {
            client: match device_type {
                DeviceType::Mic(chan) => ClientType::Mic(client, Arc::new(Mutex::new(chan))),
                DeviceType::Out(chan) => ClientType::Out(client, Arc::new(Mutex::new(chan))),
            },
        })
    }

    pub async fn play(&self) -> HowlerResult<()> {
        self.client.play().await.map_err(Error::into)
    }
}
