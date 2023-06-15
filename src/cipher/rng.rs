use aead::rand_core::block::BlockRngCore;
use aead::rand_core::{CryptoRng, SeedableRng};

type Seed = [u8; 32];

pub struct AppRngCore {
    seed: Seed,
}

impl CryptoRng for AppRngCore {}

impl BlockRngCore for AppRngCore {
    type Item = u32;
    type Results = [u32; 16];

    // FIXME: proper `generate` implementation
    fn generate(&mut self, results: &mut Self::Results) {
        let step = self.seed.len() / results.len();
        for (r, n) in results
            .iter_mut()
            .zip(self.seed.chunks(4).cycle().step_by(step))
        {
            *r = u32::from_ne_bytes([n[1], n[3], n[0], n[2]]);
        }
    }
}

impl SeedableRng for AppRngCore {
    type Seed = Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        AppRngCore { seed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cipher::{AesNonce, AesSpec, CipherHandle};
    use crate::consts::{TEST_SEED, TEST_STRING};
    use crate::Encryption;

    use aead::{
        rand_core::{block::BlockRng, RngCore},
        OsRng,
    };
    use futures::executor::block_on;

    #[test]
    fn seeding_works() {
        let mut buf = [0; 16];
        BlockRng::<AppRngCore>::from_seed(TEST_SEED).fill_bytes(&mut buf);
        // byte filling is properly seeded
        assert_ne!(buf, [0; 16]);

        BlockRng::<AppRngCore>::from_seed([0; 32]).fill_bytes(&mut buf);
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
            BlockRng::<AppRngCore>::from_seed(TEST_SEED),
        );
        let aes_b = CipherHandle::new(
            &Encryption::AES {
                cipher: AesSpec::default(),
                nonce: AesNonce::default(),
            },
            BlockRng::<AppRngCore>::from_seed(TEST_SEED),
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
