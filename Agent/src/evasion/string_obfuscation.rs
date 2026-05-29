use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct StringObfuscator;

impl StringObfuscator {
    pub fn new() -> Self {
        Self
    }

    /// XOR each byte with the given key.
    pub fn xor_encode(input: &[u8], key: u8) -> Vec<u8> {
        input.iter().map(|&b| b ^ key).collect()
    }

    /// Decode XOR-encoded bytes (symmetric operation).
    pub fn xor_decode(encoded: &[u8], key: u8) -> Vec<u8> {
        Self::xor_encode(encoded, key)
    }

    /// Generate a pseudo-random non-zero key from process entropy.
    pub fn random_key() -> u8 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let seed = nanos ^ (std::process::id() as u64).wrapping_mul(2654435761);
        let key = (seed & 0xFF) as u8;
        if key == 0 { 0x42 } else { key }
    }

    /// Obfuscate a string, returning (encoded_bytes, key).
    pub fn obfuscate_string(input: &str) -> (Vec<u8>, u8) {
        let key = Self::random_key();
        let encoded = Self::xor_encode(input.as_bytes(), key);
        (encoded, key)
    }

    /// Deobfuscate bytes back to a String.
    pub fn deobfuscate_string(encoded: &[u8], key: u8) -> String {
        let decoded = Self::xor_decode(encoded, key);
        String::from_utf8_lossy(&decoded).into_owned()
    }

    /// Multi-byte XOR for stronger obfuscation.
    pub fn xor_encode_multi(input: &[u8], key: &[u8]) -> Vec<u8> {
        if key.is_empty() {
            return input.to_vec();
        }
        input
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ key[i % key.len()])
            .collect()
    }

    /// Multi-byte XOR decode (symmetric).
    pub fn xor_decode_multi(encoded: &[u8], key: &[u8]) -> Vec<u8> {
        Self::xor_encode_multi(encoded, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_obfuscator_new() {
        let _s = StringObfuscator::new();
    }

    #[test]
    fn test_string_obfuscator_default() {
        let _s = StringObfuscator::default();
    }

    #[test]
    fn test_xor_roundtrip() {
        let input = b"Hello, world!";
        let key = 0xAB;
        let encoded = StringObfuscator::xor_encode(input, key);
        let decoded = StringObfuscator::xor_decode(&encoded, key);
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_random_key_nonzero() {
        for _ in 0..100 {
            assert_ne!(StringObfuscator::random_key(), 0);
        }
    }

    #[test]
    fn test_obfuscate_deobfuscate() {
        let original = "CreateThread";
        let (encoded, key) = StringObfuscator::obfuscate_string(original);
        assert_ne!(encoded, original.as_bytes());
        let recovered = StringObfuscator::deobfuscate_string(&encoded, key);
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_multi_key_roundtrip() {
        let input = b"VirtualAlloc";
        let key = b"secret";
        let encoded = StringObfuscator::xor_encode_multi(input, key);
        let decoded = StringObfuscator::xor_decode_multi(&encoded, key);
        assert_eq!(decoded, input);
    }
}
