use aead::rand_core::block::BlockRngCore;
use aead::rand_core::{CryptoRng, SeedableRng};
use sha2::{Digest, Sha256};

const N: usize = 32;
type SeedArray = [u8; N];

/// `AppRngSeed` for generic usage of [`AppRngCore`][AppRngCore] with `String` and `&str` types.
pub struct AppRngSeed(pub SeedArray);

impl Default for AppRngSeed {
    fn default() -> AppRngSeed {
        AppRngSeed([0; N])
    }
}

impl AsMut<[u8]> for AppRngSeed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl From<String> for AppRngSeed {
    fn from(value: String) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(value);
        Self(hasher.finalize().into())
    }
}

impl From<&str> for AppRngSeed {
    fn from(value: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(value);
        Self(hasher.finalize().into())
    }
}

/// `AppRngCore` for generic [`BlockRng`][aead::rand_core::block::BlockRng] instancing.
///
/// Implements [`generate`][AppRngCore::generate] method based on stored [`seed`][AppRngSeed] value.
pub struct AppRngCore {
    seed: AppRngSeed,
}

impl CryptoRng for AppRngCore {}

impl BlockRngCore for AppRngCore {
    type Item = u32;
    type Results = [u32; 16];

    // FIXME: proper `generate` implementation
    fn generate(&mut self, results: &mut Self::Results) {
        let step = self.seed.0.len() / results.len();
        for (r, n) in results
            .iter_mut()
            .zip(self.seed.0.chunks(4).cycle().step_by(step))
        {
            *r = u32::from_ne_bytes([n[1], n[3], n[0], n[2]]);
        }
    }
}

impl SeedableRng for AppRngCore {
    type Seed = AppRngSeed;

    // FIXME: proper `seed` storing
    fn from_seed(seed: Self::Seed) -> Self {
        Self { seed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cipher::{AesNonce, AesSpec, CipherHandle};
    use crate::consts::{TEST_PHRASE, TEST_STRING};
    use crate::Encryption;

    use aead::{
        rand_core::{block::BlockRng, RngCore},
        OsRng,
    };
    use futures::executor::block_on;

    #[test]
    fn seeding_works() {
        let mut buf = [0; 16];
        BlockRng::<AppRngCore>::from_seed(TEST_PHRASE.into()).fill_bytes(&mut buf);
        // byte filling is properly seeded
        assert_ne!(buf, [0; 16]);

        BlockRng::<AppRngCore>::from_seed(AppRngSeed::default()).fill_bytes(&mut buf);
        // blank seed produces blank fills
        assert_eq!(buf, [0; 16]);
    }

    #[test]
    fn seeding_same() {
        let aes_a = CipherHandle::new(
            &Encryption::AES {
                cipher: AesSpec::default(),
                nonce: AesNonce::default(),
            },
            BlockRng::<AppRngCore>::from_seed(TEST_PHRASE.into()),
        );
        let aes_b = CipherHandle::new(
            &Encryption::AES {
                cipher: AesSpec::default(),
                nonce: AesNonce::default(),
            },
            BlockRng::<AppRngCore>::from_seed(TEST_PHRASE.into()),
        );

        let txt_a = block_on(aes_a.encrypt(TEST_STRING.as_ref())).unwrap();
        let txt_b = block_on(aes_b.encrypt(TEST_STRING.as_ref())).unwrap();

        let res_a = block_on(aes_a.decrypt(txt_b.as_ref()));
        let res_b = block_on(aes_b.decrypt(txt_a.as_ref()));
        // ciphers with same seeds may be interchanged
        assert!(res_a.is_ok());
        assert!(res_b.is_ok());
    }

    #[test]
    fn seeding_diff() {
        let aes_a = CipherHandle::new(
            &Encryption::AES {
                cipher: AesSpec::default(),
                nonce: AesNonce::default(),
            },
            OsRng,
        );
        let aes_b = CipherHandle::new(
            &Encryption::AES {
                cipher: AesSpec::default(),
                nonce: AesNonce::default(),
            },
            OsRng,
        );

        let txt_a = block_on(aes_a.encrypt(TEST_STRING.as_ref())).unwrap();
        let txt_b = block_on(aes_b.encrypt(TEST_STRING.as_ref())).unwrap();

        let res_a = block_on(aes_a.decrypt(txt_b.as_ref()));
        let res_b = block_on(aes_b.decrypt(txt_a.as_ref()));
        // ciphers with diff seeds can't be interchanged
        assert!(res_a.is_err());
        assert!(res_b.is_err());
    }
}
