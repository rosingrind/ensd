use crate::ext::{APISyncGuard, IOAudio, MicChan, OutChan};
use async_trait::async_trait;

pub(super) struct AudioClient(wasapi::AudioClient);

impl AudioClient {
    pub(super) fn new(client: wasapi::AudioClient) -> Self {
        AudioClient(client)
    }

    pub(super) fn get(&self) -> &wasapi::AudioClient {
        &self.0
    }
}

unsafe impl Sync for AudioClient {}

unsafe impl Send for AudioClient {}

pub(super) struct WasapiClient(APISyncGuard<AudioClient>);

impl WasapiClient {
    pub(super) fn new(client: APISyncGuard<AudioClient>) -> Self {
        WasapiClient(client)
    }

    pub(super) fn get(&self) -> APISyncGuard<AudioClient> {
        self.0.clone()
    }
}

#[async_trait]
impl IOAudio for WasapiClient {
    async fn play_rec(&self, chan: &MicChan) -> err::Result<()> {
        super::mic_loop(self.get(), chan).await
    }

    async fn play_out(&self, chan: &OutChan) -> err::Result<()> {
        super::out_loop(self.get(), chan).await
    }
}
