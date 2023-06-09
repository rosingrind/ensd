mod aes;
mod cha;

use ::aes::{Aes128, Aes192, Aes256};
use ::cha::{cipher::KeyIvInit, ChaCha20, XChaCha20};
use aead::{
    consts::{U12, U13, U14, U15, U16},
    OsRng, Result,
};
use aes_gcm::KeyInit;
use serde::Deserialize;
use std::sync::Arc;

use crate::cipher::aes::AesCipher;
use crate::cipher::cha::ChaCipher;

#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum AesSpec {
    Aes128,
    Aes192,
    Aes256,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum ChaSpec {
    ChaCha20,
    XChaCha20,
}

impl Default for AesSpec {
    fn default() -> Self {
        AesSpec::Aes256
    }
}

impl Default for ChaSpec {
    fn default() -> Self {
        ChaSpec::ChaCha20
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

#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum ChaNonce {
    U12,
    U24,
}

impl Default for AesNonce {
    fn default() -> Self {
        AesNonce::U12
    }
}

impl Default for ChaNonce {
    fn default() -> Self {
        ChaNonce::U12
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

pub(super) fn get_cha_cipher(cipher: &ChaSpec) -> Arc<dyn IOCipher + Sync + Send> {
    match cipher {
        ChaSpec::ChaCha20 => Arc::new(ChaCipher::<ChaCha20, _>::from(ChaCha20::generate_key(
            OsRng,
        ))),
        ChaSpec::XChaCha20 => Arc::new(ChaCipher::<XChaCha20, _>::from(XChaCha20::generate_key(
            OsRng,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STRING: &str = "alpha test string";

    #[test]
    fn aes_works() {
        let cipher = get_aes_cipher(&AesSpec::default(), &AesNonce::default());
        let res = cipher
            .decrypt(cipher.encrypt(STRING.as_ref()).unwrap().as_ref())
            .unwrap();
        assert_eq!(res.as_slice(), STRING.as_bytes());
    }

    #[test]
    fn cha_works() {
        let cipher = get_cha_cipher(&ChaSpec::default());
        let res = cipher
            .decrypt(cipher.encrypt(STRING.as_ref()).unwrap().as_ref())
            .unwrap();
        assert_eq!(res.as_slice(), STRING.as_bytes());
    }

    #[test]
    fn cipher_diversity() {
        let aes = get_aes_cipher(&AesSpec::default(), &AesNonce::default());
        let cha = get_cha_cipher(&ChaSpec::default());

        let aes_res = aes.encrypt(STRING.as_ref()).unwrap();
        let cha_res = cha.encrypt(STRING.as_ref()).unwrap();
        assert_ne!(aes_res, cha_res);

        let aes_res = aes.decrypt(aes_res.as_ref()).unwrap();
        let cha_res = cha.decrypt(cha_res.as_ref()).unwrap();
        assert_eq!(aes_res, cha_res);
    }
}
