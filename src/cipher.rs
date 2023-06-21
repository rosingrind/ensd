mod aes;
mod cha;
mod rng;

use ::aes::{Aes128, Aes192, Aes256};
use ::cha::{cipher::KeyIvInit, ChaCha20, XChaCha20};
use aead::{
    consts::{U12, U13, U14, U15, U16},
    rand_core::{block::BlockRng, CryptoRng, RngCore, SeedableRng},
    Result,
};
use serde::Deserialize;

use crate::cipher::aes::AesCipher;
use crate::cipher::cha::ChaCipher;
use crate::cipher::rng::AppRngCore;
use crate::Encryption;

#[allow(dead_code)]
pub(super) type AppRng = BlockRng<AppRngCore>;

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

impl Default for AesNonce {
    fn default() -> Self {
        AesNonce::U12
    }
}

/// `IOCipher` trait for heterogeneous encryption implementation.
/// Assumes method implementations to [`encrypt`][IOCipher::encrypt] and
/// [`decrypt`][IOCipher::decrypt] data.
pub(super) trait IOCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    fn encrypt_at(&self, nonce: &[u8], associated_data: &[u8], buffer: &mut Vec<u8>) -> Result<()>;

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;

    fn decrypt_at(&self, nonce: &[u8], associated_data: &[u8], buffer: &mut Vec<u8>) -> Result<()>;
}

pub(super) struct CipherHandle {
    cipher: Box<dyn IOCipher + Sync + Send>,
}

#[allow(dead_code)]
impl CipherHandle {
    pub fn new(cfg: &Encryption, rng: impl CryptoRng + RngCore) -> CipherHandle {
        let cipher = match cfg {
            Encryption::AES { cipher, nonce } => get_aes_cipher(cipher, nonce, rng),
            Encryption::ChaCha { cipher } => get_cha_cipher(cipher, rng),
        };

        CipherHandle { cipher }
    }

    pub async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        self.cipher.encrypt(plaintext)
    }

    pub async fn encrypt_at(
        &self,
        nonce: &[u8],
        associated_data: &[u8],
        buffer: &mut Vec<u8>,
    ) -> Result<()> {
        self.cipher.encrypt_at(nonce, associated_data, buffer)
    }

    pub async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.cipher.decrypt(ciphertext)
    }

    pub async fn decrypt_at(
        &self,
        nonce: &[u8],
        associated_data: &[u8],
        buffer: &mut Vec<u8>,
    ) -> Result<()> {
        self.cipher.decrypt_at(nonce, associated_data, buffer)
    }
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
    rng: impl CryptoRng + RngCore,
) -> Box<dyn IOCipher + Sync + Send> {
    use aes_gcm::KeyInit;

    let gen_rng = BlockRng::<AppRngCore>::from_rng(rng).unwrap();
    match nonce {
        AesNonce::U12 => match cipher {
            AesSpec::Aes128 => Box::new(AesCipher::<_, U12>::from(Aes128::generate_key(gen_rng))),
            AesSpec::Aes192 => Box::new(AesCipher::<_, U12>::from(Aes192::generate_key(gen_rng))),
            AesSpec::Aes256 => Box::new(AesCipher::<_, U12>::from(Aes256::generate_key(gen_rng))),
        },
        AesNonce::U13 => match cipher {
            AesSpec::Aes128 => Box::new(AesCipher::<_, U13>::from(Aes128::generate_key(gen_rng))),
            AesSpec::Aes192 => Box::new(AesCipher::<_, U13>::from(Aes192::generate_key(gen_rng))),
            AesSpec::Aes256 => Box::new(AesCipher::<_, U13>::from(Aes256::generate_key(gen_rng))),
        },
        AesNonce::U14 => match cipher {
            AesSpec::Aes128 => Box::new(AesCipher::<_, U14>::from(Aes128::generate_key(gen_rng))),
            AesSpec::Aes192 => Box::new(AesCipher::<_, U14>::from(Aes192::generate_key(gen_rng))),
            AesSpec::Aes256 => Box::new(AesCipher::<_, U14>::from(Aes256::generate_key(gen_rng))),
        },
        AesNonce::U15 => match cipher {
            AesSpec::Aes128 => Box::new(AesCipher::<_, U15>::from(Aes128::generate_key(gen_rng))),
            AesSpec::Aes192 => Box::new(AesCipher::<_, U15>::from(Aes192::generate_key(gen_rng))),
            AesSpec::Aes256 => Box::new(AesCipher::<_, U15>::from(Aes256::generate_key(gen_rng))),
        },
        AesNonce::U16 => match cipher {
            AesSpec::Aes128 => Box::new(AesCipher::<_, U16>::from(Aes128::generate_key(gen_rng))),
            AesSpec::Aes192 => Box::new(AesCipher::<_, U16>::from(Aes192::generate_key(gen_rng))),
            AesSpec::Aes256 => Box::new(AesCipher::<_, U16>::from(Aes256::generate_key(gen_rng))),
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
pub(super) fn get_cha_cipher(
    cipher: &ChaSpec,
    rng: impl CryptoRng + RngCore,
) -> Box<dyn IOCipher + Sync + Send> {
    let gen_rng = BlockRng::<AppRngCore>::from_rng(rng).unwrap();
    match cipher {
        ChaSpec::ChaCha20 => Box::new(ChaCipher::<ChaCha20, _>::from(ChaCha20::generate_key(
            gen_rng,
        ))),
        ChaSpec::XChaCha20 => Box::new(ChaCipher::<XChaCha20, _>::from(XChaCha20::generate_key(
            gen_rng,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_STRING;

    use aead::{consts::U24, Nonce, OsRng};
    use aes_gcm::AesGcm;
    use chacha20poly1305::ChaChaPoly1305;

    #[async_std::test]
    async fn aes_works() {
        let cipher = CipherHandle::new(
            &Encryption::AES {
                cipher: AesSpec::default(),
                nonce: AesNonce::default(),
            },
            OsRng,
        );
        let res = cipher
            .decrypt(cipher.encrypt(TEST_STRING.as_ref()).await.unwrap().as_ref())
            .await
            .unwrap();
        assert_eq!(res.as_slice(), TEST_STRING.as_bytes());

        let res = cipher
            .encrypt_at(
                Nonce::<AesGcm<Aes256, U16>>::default().as_ref(),
                &[],
                &mut vec![0u8; 24],
            )
            .await;
        assert!(res.is_err());
    }

    #[async_std::test]
    async fn cha_works() {
        let cipher = CipherHandle::new(
            &Encryption::ChaCha {
                cipher: ChaSpec::default(),
            },
            OsRng,
        );
        let res = cipher
            .decrypt(cipher.encrypt(TEST_STRING.as_ref()).await.unwrap().as_ref())
            .await
            .unwrap();
        assert_eq!(res.as_slice(), TEST_STRING.as_bytes());

        let res = cipher
            .encrypt_at(
                Nonce::<ChaChaPoly1305<XChaCha20, U24>>::default().as_ref(),
                &[],
                &mut vec![0u8; 24],
            )
            .await;
        assert!(res.is_err());
    }

    #[async_std::test]
    async fn cipher_integrity() {
        let aes = CipherHandle::new(
            &Encryption::AES {
                cipher: AesSpec::default(),
                nonce: AesNonce::default(),
            },
            OsRng,
        );
        let cha = CipherHandle::new(
            &Encryption::ChaCha {
                cipher: ChaSpec::default(),
            },
            OsRng,
        );

        let aes_res = aes.encrypt(TEST_STRING.as_ref()).await.unwrap();
        let cha_res = cha.encrypt(TEST_STRING.as_ref()).await.unwrap();
        // ciphers produce different output
        assert_ne!(aes_res, cha_res);

        let cha_aes = cha.decrypt(aes_res.as_ref()).await;
        let aes_cha = aes.decrypt(cha_res.as_ref()).await;
        // different ciphers can't decrypt each other
        assert!(cha_aes.is_err());
        assert!(aes_cha.is_err());

        let aes_res = aes.decrypt(aes_res.as_ref()).await.unwrap();
        let cha_res = cha.decrypt(cha_res.as_ref()).await.unwrap();
        // ciphers are operating on the same data
        assert_eq!(aes_res, cha_res);
    }
}
