use aes_gcm::Aes256Gcm; // AES256
use aes_gcm::aead::{Aead, KeyInit, OsRng, generic_array::GenericArray};
use aes_gcm::Nonce;
use rand_core::RngCore;
use base64::{engine::general_purpose, Engine as _};

// 32 בייט מפתח
const KEY_BYTES: [u8; 32] = [
    1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,
    17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32
];

pub fn encrypt_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = GenericArray::from_slice(&KEY_BYTES);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, password.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(general_purpose::STANDARD.encode(&combined))
}

pub fn decrypt_password(encoded: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data = general_purpose::STANDARD.decode(encoded)?;
    if data.len() < 12 { return Err("Data too short".into()); }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = GenericArray::from_slice(nonce_bytes);

    let key = GenericArray::from_slice(&KEY_BYTES);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    Ok(String::from_utf8(plaintext)?)
}
