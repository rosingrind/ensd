use aead::{
    consts::U16, generic_array::ArrayLength, Aead, AeadCore, AeadInPlace, Error, Nonce, OsRng,
    Result,
};
use aes::{
    cipher::{BlockCipher, BlockEncrypt, BlockSizeUser, KeyInit},
    Aes128, Aes192, Aes256,
};
use aes_gcm::{AesGcm, Key};
use log::error;

use crate::cipher::IOCipher;

pub(super) struct AesCipher<T, U>
where
    T: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt,
    U: ArrayLength<u8>,
{
    cipher: AesGcm<T, U>,
}

impl<T, U> AesCipher<T, U>
where
    T: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt + KeyInit,
    U: ArrayLength<u8>,
{
    #[allow(dead_code)]
    fn new(key: Key<T>) -> AesCipher<T, U>
    where
        Key<T>: Into<AesCipher<T, U>>,
    {
        key.into()
    }
}

impl<T, U> IOCipher for AesCipher<T, U>
where
    T: BlockCipher + BlockSizeUser<BlockSize = U16> + BlockEncrypt,
    U: ArrayLength<u8>,
{
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = &AesGcm::<T, U>::generate_nonce(&mut OsRng);
        Ok([
            nonce.as_ref(),
            self.cipher.encrypt(nonce, plaintext)?.as_ref(),
        ]
        .concat())
    }

    fn encrypt_at(&self, nonce: &[u8], associated_data: &[u8], buffer: &mut Vec<u8>) -> Result<()> {
        let spec = Nonce::<AesGcm<T, U>>::default().len();
        if nonce.len() == spec {
            self.cipher
                .encrypt_in_place(nonce.into(), associated_data, buffer)
        } else {
            error!(
                "'encrypt_at' error: nonce size '{}' is incompatible with '{}'",
                nonce.len(),
                spec
            );
            Err(Error)
        }
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let spec = Nonce::<AesGcm<T, U>>::default().len();
        if spec > ciphertext.len() {
            error!(
                "'decrypt' error: nonce size '{}' is bigger than 'ciphertext.len()':'{}'",
                spec,
                ciphertext.len()
            );
            return Err(Error);
        }
        let (nonce, ciphertext) = {
            let (a, b) = ciphertext.split_at(spec);
            (a.into(), b)
        };
        self.cipher.decrypt(nonce, ciphertext)
    }

    fn decrypt_at(&self, nonce: &[u8], associated_data: &[u8], buffer: &mut Vec<u8>) -> Result<()> {
        let spec = Nonce::<AesGcm<T, U>>::default().len();
        if nonce.len() == spec {
            self.cipher
                .decrypt_in_place(nonce.into(), associated_data, buffer)
        } else {
            error!(
                "'decrypt_at' error: nonce size '{}' is incompatible with '{}'",
                nonce.len(),
                spec
            );
            Err(Error)
        }
    }
}

impl<T: ArrayLength<u8>> From<Key<Aes128>> for AesCipher<Aes128, T> {
    fn from(a: Key<Aes128>) -> AesCipher<Aes128, T> {
        AesCipher {
            cipher: AesGcm::<Aes128, T>::new(&a),
        }
    }
}

impl<T: ArrayLength<u8>> From<Key<Aes192>> for AesCipher<Aes192, T> {
    fn from(a: Key<Aes192>) -> AesCipher<Aes192, T> {
        AesCipher {
            cipher: AesGcm::<Aes192, T>::new(&a),
        }
    }
}

impl<T: ArrayLength<u8>> From<Key<Aes256>> for AesCipher<Aes256, T> {
    fn from(a: Key<Aes256>) -> AesCipher<Aes256, T> {
        AesCipher {
            cipher: AesGcm::<Aes256, T>::new(&a),
        }
    }
}
