mod cipher;
mod consts;
mod stream;

use serde::Deserialize;
use std::net::Ipv4Addr;
use std::{fs, path::Path};

use crate::cipher::{AesNonce, AesSpec, ChaSpec};

#[derive(Debug, Deserialize)]
struct Data {
    encryption: Encryption,
    udp_a: UDP,
    udp_b: UDP,
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

const DATA_PATH: &str = "./res/data";

fn main() {
    let path = Path::new(DATA_PATH).with_extension("toml");
    toml::from_str::<Data>(&*fs::read_to_string(path).unwrap()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_is_valid() {
        let path = Path::new(DATA_PATH).with_extension("toml");
        toml::from_str::<Data>(&*fs::read_to_string(path).unwrap()).unwrap();
    }
}
