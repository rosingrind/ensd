use crate::ext::{APISyncGuard, IOAudio, MicChan, OutChan};
use async_trait::async_trait;

pub(super) struct AudioClient(coreaudio::audio_unit::AudioUnit);

impl AudioClient {
    pub(super) fn new(client: coreaudio::audio_unit::AudioUnit) -> Self {
        AudioClient(client)
    }

    pub(super) fn get(&self) -> &coreaudio::audio_unit::AudioUnit {
        &self.0
    }

    pub(super) fn get_mut(&mut self) -> &mut coreaudio::audio_unit::AudioUnit {
        &mut self.0
    }
}

unsafe impl Sync for AudioClient {}

unsafe impl Send for AudioClient {}

pub(super) struct CAudioClient(APISyncGuard<AudioClient>);

impl CAudioClient {
    pub(super) fn new(client: APISyncGuard<AudioClient>) -> Self {
        CAudioClient(client)
    }

    pub(super) fn get(&self) -> APISyncGuard<AudioClient> {
        self.0.clone()
    }
}

#[async_trait]
impl IOAudio for CAudioClient {
    async fn play_rec(&self, chan: APISyncGuard<MicChan>) -> err::Result<()> {
        super::mic_loop(self.get(), chan).await
    }

    async fn play_out(&self, chan: APISyncGuard<OutChan>) -> err::Result<()> {
        super::out_loop(self.get(), chan).await
    }
}
