use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use sha2::{Digest, Sha256};

fn derive_key(passphrase: &str) -> [u8; 32] {
    Sha256::digest(passphrase.as_bytes()).into()
}

/// Encrypts an API key using AES-256-GCM.
/// Requires `AI_KEY_MASTER` env var. Returns error if not configured.
pub fn encrypt_api_key(plaintext: &str) -> Result<String, String> {
    let passphrase = std::env::var("AI_KEY_MASTER")
        .map_err(|_| "AI_KEY_MASTER not configured — API key will be stored in plaintext".to_string())?;

    let key_bytes = derive_key(&passphrase);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(B64.encode(&combined))
}

/// Decrypts an API key. Falls back to the stored value if decryption fails
/// (backward compat with legacy plaintext keys or when AI_KEY_MASTER is not set).
pub fn decrypt_api_key(stored: &str) -> String {
    let Ok(passphrase) = std::env::var("AI_KEY_MASTER") else {
        return stored.to_string();
    };

    let Ok(combined) = B64.decode(stored) else {
        return stored.to_string();
    };

    if combined.len() < 28 {
        // 12-byte nonce + 16-byte GCM tag minimum
        return stored.to_string();
    }

    let (nonce_bytes, ct) = combined.split_at(12);
    let key_bytes = derive_key(&passphrase);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ct)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_else(|| stored.to_string())
}
