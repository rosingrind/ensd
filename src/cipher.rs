mod aes;

use ::aes::{Aes128, Aes192, Aes256};
use aead::{
    consts::{U12, U13, U14, U15, U16},
    OsRng, Result,
};
use aes_gcm::KeyInit;
use serde::Deserialize;
use std::sync::Arc;

use crate::cipher::aes::AesCipher;

#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum AesSpec {
    Aes128,
    Aes192,
    Aes256,
}

impl Default for AesSpec {
    fn default() -> Self {
        AesSpec::Aes256
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum AesNonce {
    U12,
    U13,
    U14,
    U15,
    U16,
}

impl Default for AesNonce {
    fn default() -> Self {
        AesNonce::U12
    }
}
pub(super) trait IOCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

pub(super) fn get_aes_cipher(
    cipher: &AesSpec,
    nonce: &AesNonce,
) -> Arc<dyn IOCipher + Sync + Send> {
    match nonce {
        AesNonce::U12 => match cipher {
            AesSpec::Aes128 => Arc::new(AesCipher::<_, U12>::from(Aes128::generate_key(OsRng))),
            AesSpec::Aes192 => Arc::new(AesCipher::<_, U12>::from(Aes192::generate_key(OsRng))),
            AesSpec::Aes256 => Arc::new(AesCipher::<_, U12>::from(Aes256::generate_key(OsRng))),
        },
        AesNonce::U13 => match cipher {
            AesSpec::Aes128 => Arc::new(AesCipher::<_, U13>::from(Aes128::generate_key(OsRng))),
            AesSpec::Aes192 => Arc::new(AesCipher::<_, U13>::from(Aes192::generate_key(OsRng))),
            AesSpec::Aes256 => Arc::new(AesCipher::<_, U13>::from(Aes256::generate_key(OsRng))),
        },
        AesNonce::U14 => match cipher {
            AesSpec::Aes128 => Arc::new(AesCipher::<_, U14>::from(Aes128::generate_key(OsRng))),
            AesSpec::Aes192 => Arc::new(AesCipher::<_, U14>::from(Aes192::generate_key(OsRng))),
            AesSpec::Aes256 => Arc::new(AesCipher::<_, U14>::from(Aes256::generate_key(OsRng))),
        },
        AesNonce::U15 => match cipher {
            AesSpec::Aes128 => Arc::new(AesCipher::<_, U15>::from(Aes128::generate_key(OsRng))),
            AesSpec::Aes192 => Arc::new(AesCipher::<_, U15>::from(Aes192::generate_key(OsRng))),
            AesSpec::Aes256 => Arc::new(AesCipher::<_, U15>::from(Aes256::generate_key(OsRng))),
        },
        AesNonce::U16 => match cipher {
            AesSpec::Aes128 => Arc::new(AesCipher::<_, U16>::from(Aes128::generate_key(OsRng))),
            AesSpec::Aes192 => Arc::new(AesCipher::<_, U16>::from(Aes192::generate_key(OsRng))),
            AesSpec::Aes256 => Arc::new(AesCipher::<_, U16>::from(Aes256::generate_key(OsRng))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_works() {
        let cipher = get_aes_cipher(&AesSpec::default(), &AesNonce::default());
        let string = "alpha test string";
        let res = cipher
            .decrypt(cipher.encrypt(string.as_ref()).unwrap().as_ref())
            .unwrap();
        assert_eq!(res.as_slice(), string.as_bytes());
    }
}
