mod cipher;
mod consts;
mod stream;

use serde::Deserialize;
use std::net::Ipv4Addr;

use crate::cipher::{AesNonce, AesSpec, ChaSpec};

#[derive(Debug, Deserialize)]
struct Config {
    encryption: Encryption,
    msg_a: UDP,
    snd_a: UDP,
    msg_b: UDP,
    snd_b: UDP,
}

#[derive(Debug, Deserialize)]
struct UDP {
    ip: Ipv4Addr,
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
        let path = Path::new(RESOURCES_PATH).join("data.toml");
        toml::from_str::<Config>(&*fs::read_to_string(path).unwrap()).unwrap();
    }
}
