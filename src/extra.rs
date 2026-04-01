#[cfg(not(any(
    feature = "hashes_backend",
    feature = "ring_backend",
    feature = "mix_backend"
)))]
compile_error!("You must enable at least one of the features: 'hashes_backend', 'ring_backend' or 'mix_backend'.");
#[cfg(any(
    all(feature = "hashes_backend", feature = "ring_backend"),
    all(feature = "hashes_backend", feature = "mix_backend"),
    all(feature = "ring_backend", feature = "mix_backend"),
    all(
        feature = "hashes_backend",
        feature = "ring_backend",
        feature = "mix_backend"
    )
))]
compile_error!(
    "Only one of the features `hashes_backend`, `ring_backend`, or `mix_backend` can be enabled at a time."
);

use crate::calculator::SupportedAlgorithm;
use std::fmt::Write;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(hex_string, "{byte:02x}").unwrap();
    }
    hex_string
}

fn is_ascii_hex(input: &str) -> bool {
    input.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_xxh3_64(input: &str) -> bool {
    input
        .strip_prefix("XXH3_")
        .or_else(|| input.strip_prefix("xxh3_"))
        .is_some_and(is_ascii_hex)
}

pub fn detect_hash_algorithm<S: AsRef<str>>(hash: S) -> Result<Vec<SupportedAlgorithm>, String> {
    let hash = hash.as_ref();

    match hash.len() {
        8 if is_ascii_hex(hash) => Ok(vec![SupportedAlgorithm::XXHASH32]),
        16 if is_ascii_hex(hash) => Ok(vec![SupportedAlgorithm::XXHASH64]),
        21 if is_xxh3_64(hash) => Ok(vec![SupportedAlgorithm::XXHASH3_64]),
        #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
        32 if is_ascii_hex(hash) => Ok(vec![
            SupportedAlgorithm::MD5,
            SupportedAlgorithm::MD4,
            SupportedAlgorithm::MD2,
        ]),
        #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
        40 if is_ascii_hex(hash) => Ok(vec![SupportedAlgorithm::SHA1]),
        #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
        56 if is_ascii_hex(hash) => Ok(vec![SupportedAlgorithm::SHA224]),
        64 if is_ascii_hex(hash) => Ok(vec![
            SupportedAlgorithm::SHA256,
            SupportedAlgorithm::SHA512_256,
        ]),
        96 if is_ascii_hex(hash) => Ok(vec![SupportedAlgorithm::SHA384]),
        128 if is_ascii_hex(hash) => Ok(vec![SupportedAlgorithm::SHA512]),
        _ => Err(String::from("Error: Invalid hash.")),
    }
}

#[cfg(test)]
mod test_extra {
    use super::*;
    use crate::calculator::SupportedAlgorithm;

    #[test]
    fn test_detect_hash_algorithm() {
        assert_eq!(
            detect_hash_algorithm(
                "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
            )
            .unwrap()[0],
            SupportedAlgorithm::SHA256
        )
    }

    #[test]
    fn test_detect_hash_algorithm_rejects_invalid_hex() {
        assert!(detect_hash_algorithm(
            "zz691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        )
        .is_err());
    }

    #[test]
    fn test_detect_hash_algorithm_accepts_lowercase_xxh3_prefix() {
        assert_eq!(
            detect_hash_algorithm("xxh3_802c0db623389036").unwrap(),
            vec![SupportedAlgorithm::XXHASH3_64]
        );
    }

    #[test]
    fn test_detect_hash_algorithm_xxhash64() {
        assert_eq!(
            detect_hash_algorithm("4a34911ba20e6c30").unwrap(),
            vec![SupportedAlgorithm::XXHASH64]
        );
    }
}
