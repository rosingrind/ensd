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

/// `AesSpec` for `.toml` config parsing.
/// Offers [`Aes128`][Aes128], [`Aes192`][Aes192] and [`Aes256`][Aes256] block ciphers
/// with [`Aes128`][AesSpec::default] being default choice.
#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum AesSpec {
    Aes128,
    Aes192,
    Aes256,
}

/// `ChaSpec` for `.toml` config parsing.
/// Offers [`ChaCha20`][ChaCha20] and [`XChaCha20`][XChaCha20] block ciphers with
/// [`ChaCha20`][ChaSpec::default] being default choice.
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

/// `AesNonce` for `.toml` config parsing.
/// Offers limited values accepted by [`TagSize`][aes_gcm::TagSize] for available
/// [`AesSpec`][AesSpec] ciphers.
///
/// See [`get_aes_cipher`][get_aes_cipher] for details about recommended `AesNonce` values.
#[derive(Debug, Deserialize, Copy, Clone)]
pub(super) enum AesNonce {
    U12,
    U13,
    U14,
    U15,
    U16,
}

/// `ChaNonce` for `.toml` config parsing.
/// Offers limited values accepted by [`chacha20`][::cha] package for available
/// [`ChaSpec`][ChaSpec] ciphers.
///
/// See [`get_cha_cipher`][get_cha_cipher] for details about recommended `ChaNonce` values.
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

/// `IOCipher` trait for heterogeneous encryption implementation.
/// Assumes method implementations to [`encrypt`][IOCipher::encrypt] and
/// [`decrypt`][IOCipher::decrypt] data.
pub(super) trait IOCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

/// A thread-safe `AES` cipher constructor.
/// Returns [`Arc`][Arc] wrapped trait object interfaced with abstract [`IOCipher`][IOCipher]
/// trait.
///
/// `AES` cipher builder is meant to be configurable for various protocol implementation needs,
/// but note that reference `AES` implementation treats [`U12`][AesNonce::default] nonce size.
///
/// Current cipher implementation allows [`Aes128`][Aes128], [`Aes192`][Aes192] and
/// [`Aes256`][Aes256] block ciphers.
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

/// A thread-safe `ChaCha + Poly1305` cipher constructor.
/// Returns [`Arc`][Arc] wrapped trait object interfaced with abstract [`IOCipher`][IOCipher]
/// trait.
///
/// `ChaCha` cipher builder meant to be configurable for various protocol implementation needs,
/// but note that reference `ChaCha` and `XChaCha` implementations treat
/// [`U12`][ChaNonce::default] and [`U24`][ChaNonce::U24] nonce sizes.
///
/// Current cipher implementation allows [`ChaCha20`][ChaCha20] and [`XChaCha20`][XChaCha20]
/// block ciphers.
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
    fn cipher_integrity() {
        let aes = get_aes_cipher(&AesSpec::default(), &AesNonce::default());
        let cha = get_cha_cipher(&ChaSpec::default());

        let aes_res = aes.encrypt(STRING.as_ref()).unwrap();
        let cha_res = cha.encrypt(STRING.as_ref()).unwrap();
        // ciphers are different
        assert_ne!(aes_res, cha_res);

        let aes_res = aes.decrypt(aes_res.as_ref()).unwrap();
        let cha_res = cha.decrypt(cha_res.as_ref()).unwrap();
        // ciphers operating on the same data
        assert_eq!(aes_res, cha_res);
    }
}
