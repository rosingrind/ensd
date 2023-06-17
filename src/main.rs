mod cipher;
mod consts;
mod stream;

use serde::Deserialize;
use std::net::IpAddr;

use crate::cipher::{AesNonce, AesSpec, ChaSpec};

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

fn main() {
    todo!()
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
