mod crypto;

use aes_gcm::{
    aead::{
        consts::{U12, U13, U14, U15, U16},
        OsRng, Result,
    },
    aes::{Aes128, Aes192, Aes256},
    KeyInit,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::cipher::crypto::Crypto;

#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum CipherSpec {
    Aes128,
    Aes192,
    Aes256,
}

impl Default for CipherSpec {
    fn default() -> Self {
        CipherSpec::Aes256
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum NonceSpec {
    U12,
    U13,
    U14,
    U15,
    U16,
}

impl Default for NonceSpec {
    fn default() -> Self {
        NonceSpec::U12
    }
}

pub(super) trait IOCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

pub(super) fn get_cipher(
    cipher: &CipherSpec,
    nonce: &NonceSpec,
) -> Arc<dyn IOCipher + Sync + Send> {
    match nonce {
        NonceSpec::U12 => match cipher {
            CipherSpec::Aes128 => Arc::new(Crypto::<_, U12>::from(Aes128::generate_key(OsRng))),
            CipherSpec::Aes192 => Arc::new(Crypto::<_, U12>::from(Aes192::generate_key(OsRng))),
            CipherSpec::Aes256 => Arc::new(Crypto::<_, U12>::from(Aes256::generate_key(OsRng))),
        },
        NonceSpec::U13 => match cipher {
            CipherSpec::Aes128 => Arc::new(Crypto::<_, U13>::from(Aes128::generate_key(OsRng))),
            CipherSpec::Aes192 => Arc::new(Crypto::<_, U13>::from(Aes192::generate_key(OsRng))),
            CipherSpec::Aes256 => Arc::new(Crypto::<_, U13>::from(Aes256::generate_key(OsRng))),
        },
        NonceSpec::U14 => match cipher {
            CipherSpec::Aes128 => Arc::new(Crypto::<_, U14>::from(Aes128::generate_key(OsRng))),
            CipherSpec::Aes192 => Arc::new(Crypto::<_, U14>::from(Aes192::generate_key(OsRng))),
            CipherSpec::Aes256 => Arc::new(Crypto::<_, U14>::from(Aes256::generate_key(OsRng))),
        },
        NonceSpec::U15 => match cipher {
            CipherSpec::Aes128 => Arc::new(Crypto::<_, U15>::from(Aes128::generate_key(OsRng))),
            CipherSpec::Aes192 => Arc::new(Crypto::<_, U15>::from(Aes192::generate_key(OsRng))),
            CipherSpec::Aes256 => Arc::new(Crypto::<_, U15>::from(Aes256::generate_key(OsRng))),
        },
        NonceSpec::U16 => match cipher {
            CipherSpec::Aes128 => Arc::new(Crypto::<_, U16>::from(Aes128::generate_key(OsRng))),
            CipherSpec::Aes192 => Arc::new(Crypto::<_, U16>::from(Aes192::generate_key(OsRng))),
            CipherSpec::Aes256 => Arc::new(Crypto::<_, U16>::from(Aes256::generate_key(OsRng))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cipher_works() {
        let cipher = get_cipher(&CipherSpec::default(), &NonceSpec::default());
        let string = "alpha test string";
        let res = cipher
            .decrypt(cipher.encrypt(string.as_ref()).unwrap().as_ref())
            .unwrap();
        assert_eq!(res.as_slice(), string.as_bytes());
    }
}
