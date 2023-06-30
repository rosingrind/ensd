mod ext;
mod win;

use err::{Error, Result};
use ext::{BoxClient, MicChan, OutChan};
use howler::Result as HowlerResult;
#[cfg(windows)]
use win::DeviceBuilder;

const S_RATE: usize = 32000;
const CHUNK_SIZE: usize = 4096;
const TIMEOUT: u32 = 1000;

pub enum DeviceType {
    Mic(MicChan),
    Out(OutChan),
}

enum ClientType {
    Mic(BoxClient, MicChan),
    Out(BoxClient, OutChan),
}

impl ClientType {
    async fn play(&self) -> Result<()> {
        match self {
            ClientType::Mic(audio_client, chan) => audio_client.play_rec(chan).await,
            ClientType::Out(audio_client, chan) => audio_client.play_out(chan).await,
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
                DeviceType::Mic(chan) => ClientType::Mic(client, chan),
                DeviceType::Out(chan) => ClientType::Out(client, chan),
            },
        })
    }

    pub async fn play(&self) -> HowlerResult<()> {
        self.client.play().await.map_err(Error::into)
    }
}
