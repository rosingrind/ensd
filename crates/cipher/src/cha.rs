use aead::{consts::U32, generic_array::ArrayLength, Aead, AeadInPlace, Key, Nonce, OsRng};
use cha::cipher::{KeyInit, KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20poly1305::{AeadCore, ChaChaPoly1305};
use err::{Error, Result};
use log::{error, trace};

use crate::IOCipher;

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
        trace!("encrypting {} bytes of plaintext", plaintext.len());

        let nonce = &ChaChaPoly1305::<C, N>::generate_nonce(&mut OsRng);
        Ok([
            nonce.as_ref(),
            self.cipher.encrypt(nonce, plaintext)?.as_ref(),
        ]
        .concat())
    }

    fn encrypt_at(&self, nonce: &[u8], associated_data: &[u8], buffer: &mut Vec<u8>) -> Result<()> {
        trace!("encrypting {} bytes at buffer", buffer.len());

        let spec = Nonce::<ChaChaPoly1305<C, N>>::default().len();
        // FIXME: duplicate code fragment
        if nonce.len() == spec {
            self.cipher
                .encrypt_in_place(nonce.into(), associated_data, buffer)
                .map_err(Error::from)
        } else {
            error!(
                "'encrypt_at' error: nonce size '{}' is incompatible with '{}'",
                nonce.len(),
                spec
            );
            Err(aead::Error.into())
        }
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        trace!("decrypting {} bytes of ciphertext", ciphertext.len());

        let spec = Nonce::<ChaChaPoly1305<C, N>>::default().len();
        if spec > ciphertext.len() {
            error!(
                "'decrypt' error: nonce size '{}' is bigger than 'ciphertext.len()':'{}'",
                spec,
                ciphertext.len()
            );
            return Err(aead::Error.into());
        }
        let (nonce, ciphertext) = {
            let (a, b) = ciphertext.split_at(spec);
            (a.into(), b)
        };
        self.cipher.decrypt(nonce, ciphertext).map_err(Error::from)
    }

    fn decrypt_at(&self, nonce: &[u8], associated_data: &[u8], buffer: &mut Vec<u8>) -> Result<()> {
        trace!("decrypting {} bytes at buffer", buffer.len());

        let spec = Nonce::<ChaChaPoly1305<C, N>>::default().len();
        // FIXME: duplicate code fragment
        if nonce.len() == spec {
            self.cipher
                .decrypt_in_place(nonce.into(), associated_data, buffer)
                .map_err(Error::from)
        } else {
            error!(
                "'decrypt_at' error: nonce size '{}' is incompatible with '{}'",
                nonce.len(),
                spec
            );
            Err(aead::Error.into())
        }
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
