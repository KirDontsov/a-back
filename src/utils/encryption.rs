use aes::Aes256;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use hex;
use rand_core::{OsRng, RngCore};
use std::error::Error;
use std::fmt;

// Custom error type to handle different error types
#[derive(Debug)]
pub struct EncryptionError {
	message: String,
}

impl fmt::Display for EncryptionError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.message)
	}
}

impl Error for EncryptionError {}

pub fn encrypt_data(data: &str, key: &[u8; 32], iv: &[u8; 16]) -> String {
	type Aes256CbcEnc = Encryptor<Aes256>;

	let cipher = Aes256CbcEnc::new_from_slices(key, iv).expect("Invalid key or IV length");
	let plaintext = data.as_bytes();

	// Pad the plaintext to a multiple of block size using PKCS7
	let block_size = 16;
	let padding_len = block_size - (plaintext.len() % block_size);
	let mut padded_plaintext = plaintext.to_vec();
	padded_plaintext.resize(plaintext.len() + padding_len, padding_len as u8);

	let ciphertext =
		cipher.encrypt_padded_vec_mut::<cipher::block_padding::Pkcs7>(&padded_plaintext);
	hex::encode(ciphertext)
}

pub fn decrypt_data(
	encrypted_data: &str,
	key: &[u8; 32],
	iv: &[u8; 16],
) -> Result<String, Box<dyn std::error::Error>> {
	type Aes256CbcDec = Decryptor<Aes256>;

	let ciphertext = hex::decode(encrypted_data)?;
	let cipher = Aes256CbcDec::new_from_slices(key, iv).expect("Invalid key or IV length");

	// Handle the decryption result properly
	match cipher.decrypt_padded_vec_mut::<cipher::block_padding::Pkcs7>(&ciphertext) {
		Ok(mut decrypted) => {
			// Remove PKCS7 padding manually if needed
			if !decrypted.is_empty() {
				let padding_len = decrypted[decrypted.len() - 1] as usize;
				if padding_len <= decrypted.len()
					&& decrypted
						.iter()
						.skip(decrypted.len() - padding_len)
						.all(|&x| x == padding_len as u8)
				{
					decrypted.truncate(decrypted.len() - padding_len);
				}
			}
			Ok(String::from_utf8(decrypted)?)
		}
		Err(e) => Err(Box::new(EncryptionError {
			message: format!("Decryption failed: {:?}", e),
		})),
	}
}

pub fn generate_iv() -> [u8; 16] {
	let mut iv = [0u8; 16];
	OsRng.fill_bytes(&mut iv);
	iv
}
