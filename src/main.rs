mod cipher;
mod stream;

use serde::Deserialize;
use std::net::Ipv4Addr;
use std::{fs, path::Path};

use crate::cipher::{CipherSpec, NonceSpec};

#[derive(Debug, Deserialize)]
struct Data {
    #[serde(default)]
    cipher: CipherSpec,
    #[serde(default)]
    nonce: NonceSpec,
    udp_a: UDP,
    udp_b: UDP,
}

#[derive(Debug, Deserialize)]
struct UDP {
    ip: Ipv4Addr,
    port: u16,
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
    fn config_valid() {
        let path = Path::new(DATA_PATH).with_extension("toml");
        toml::from_str::<Data>(&*fs::read_to_string(path).unwrap()).unwrap();
    }
}
