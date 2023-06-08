use aes_gcm::{
    aead::{
        consts::U16, generic_array::ArrayLength, Aead, AeadCore, KeyInit, Nonce, OsRng, Result,
    },
    aes::{
        cipher::{BlockCipher, BlockEncrypt, BlockSizeUser},
        Aes128, Aes192, Aes256,
    },
    AesGcm, Key,
};

use crate::cipher::IOCipher;

pub(super) struct Crypto<T, U>
where
    // FIXME: sync/extend trait list
    T: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt,
    U: ArrayLength<u8>,
{
    nonce: Nonce<AesGcm<T, U>>,
    cipher: AesGcm<T, U>,
}

impl<T, U> Crypto<T, U>
where
    T: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt + KeyInit,
    U: ArrayLength<u8>,
{
    #[allow(dead_code)]
    fn new(key: Key<T>) -> Crypto<T, U>
    where
        Key<T>: Into<Crypto<T, U>>,
    {
        key.into()
    }
}

impl<T, U> IOCipher for Crypto<T, U>
where
    T: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt,
    U: ArrayLength<u8>,
{
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        self.cipher.encrypt(&self.nonce, plaintext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.cipher.decrypt(&self.nonce, ciphertext)
    }
}

impl<T: ArrayLength<u8>> From<Key<Aes128>> for Crypto<Aes128, T> {
    fn from(a: Key<Aes128>) -> Crypto<Aes128, T> {
        Crypto {
            cipher: AesGcm::<Aes128, T>::new(&a),
            nonce: AesGcm::<Aes128, T>::generate_nonce(&mut OsRng),
        }
    }
}

impl<T: ArrayLength<u8>> From<Key<Aes192>> for Crypto<Aes192, T> {
    fn from(a: Key<Aes192>) -> Crypto<Aes192, T> {
        Crypto {
            cipher: AesGcm::<Aes192, T>::new(&a),
            nonce: AesGcm::<Aes192, T>::generate_nonce(&mut OsRng),
        }
    }
}

impl<T: ArrayLength<u8>> From<Key<Aes256>> for Crypto<Aes256, T> {
    fn from(a: Key<Aes256>) -> Crypto<Aes256, T> {
        Crypto {
            cipher: AesGcm::<Aes256, T>::new(&a),
            nonce: AesGcm::<Aes128, T>::generate_nonce(&mut OsRng),
        }
    }
}
