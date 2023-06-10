use aead::{consts::U32, generic_array::ArrayLength, Aead, Key, Nonce, OsRng, Result};
use cha::cipher::{KeyInit, KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20poly1305::{AeadCore, ChaChaPoly1305};

use crate::cipher::IOCipher;

pub(super) struct ChaCipher<C, N>
where
    C: KeyIvInit<KeySize = U32, IvSize = N> + StreamCipher + StreamCipherSeek,
    N: ArrayLength<u8>,
{
    cipher: ChaChaPoly1305<C, N>,
}

impl<C, N> ChaCipher<C, N>
where
    C: KeyIvInit<KeySize = U32, IvSize = N> + StreamCipher + StreamCipherSeek,
    N: ArrayLength<u8>,
{
    #[allow(dead_code)]
    fn new(key: Key<C>) -> ChaCipher<C, N>
    where
        Key<C>: Into<ChaCipher<C, N>>,
    {
        key.into()
    }
}

impl<C, N> IOCipher for ChaCipher<C, N>
where
    C: KeyIvInit<KeySize = U32, IvSize = N> + StreamCipher + StreamCipherSeek,
    N: ArrayLength<u8>,
{
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = &ChaChaPoly1305::<C, N>::generate_nonce(&mut OsRng);
        Ok([
            nonce.as_ref(),
            self.cipher.encrypt(nonce, plaintext)?.as_ref(),
        ]
        .concat())
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let spec = Nonce::<ChaChaPoly1305<C, N>>::default().len();
        let (nonce, ciphertext) = {
            let (a, b) = ciphertext.split_at(spec);
            (a.into(), b)
        };
        self.cipher.decrypt(nonce, ciphertext)
    }
}

impl<C, N> From<Key<C>> for ChaCipher<C, N>
where
    C: KeyIvInit<KeySize = U32, IvSize = N> + StreamCipher + StreamCipherSeek,
    N: ArrayLength<u8>,
{
    fn from(a: Key<C>) -> ChaCipher<C, N> {
        ChaCipher {
            cipher: ChaChaPoly1305::<C, N>::new(&a),
        }
    }
}
