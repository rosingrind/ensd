use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use err::Result;

use crate::DeviceType;

pub(crate) trait DeviceBuilder {
    fn new_client(device_type: &DeviceType) -> Result<BoxClient>;
}
pub(crate) type BoxClient = Box<dyn IOAudio + Sync + Send>;
pub(crate) type APISyncGuard<T> = Arc<Mutex<T>>;

pub(super) type MicChan = async_std::channel::Sender<Vec<u8>>;
pub(super) type OutChan = async_std::channel::Receiver<Vec<u8>>;

#[async_trait]
pub(crate) trait IOAudio {
    async fn play_rec(&self, chan: APISyncGuard<MicChan>) -> Result<()>;

    async fn play_out(&self, chan: APISyncGuard<OutChan>) -> Result<()>;
}
