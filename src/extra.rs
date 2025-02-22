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

// Bytes to Hex
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(hex_string, "{:02x}", byte).unwrap();
    }
    hex_string
}

// // Hex to Bytes
// use std::io;
// pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, io::Error> {
//     if hex.len() % 2 != 0 {
//         return Err(io::Error::new(io::ErrorKind::InvalidData, "Error: Invalid hex."));
//     }
//
//     (0..hex.len())
//         .step_by(2)
//         .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
//         .collect()
// }

// Detect hash algorithm
#[cfg(any(feature = "mix_backend"))]
pub fn detect_hash_algorithm<S: AsRef<str>>(hash: S) -> Result<Vec<SupportedAlgorithm>, String> {
    match hash.as_ref().len() {
        40 => Ok(vec![SupportedAlgorithm::SHA1]),
        56 => Ok(vec![SupportedAlgorithm::SHA224]),
        64 => Ok(vec![
            SupportedAlgorithm::SHA256,
            SupportedAlgorithm::SHA512_256,
        ]),
        96 => Ok(vec![SupportedAlgorithm::SHA384]),
        128 => Ok(vec![SupportedAlgorithm::SHA512]),
        8 => Ok(vec![SupportedAlgorithm::XXHASH32]),
        16 => Ok(vec![SupportedAlgorithm::XXHASH64]),
        21 => Ok(vec![SupportedAlgorithm::XXHASH3_64]),
        32 => Ok(vec![
            SupportedAlgorithm::MD5,
            SupportedAlgorithm::MD4,
            SupportedAlgorithm::MD2,
        ]),
        _ => Err(String::from("Error: Invalid hash.")),
    }
}

#[cfg(any(feature = "hashes_backend"))]
pub fn detect_hash_algorithm<S: AsRef<str>>(hash: S) -> Result<Vec<SupportedAlgorithm>, String> {
    match hash.as_ref().len() {
        40 => Ok(vec![SupportedAlgorithm::SHA1]),
        56 => Ok(vec![SupportedAlgorithm::SHA224]),
        64 => Ok(vec![
            SupportedAlgorithm::SHA256,
            SupportedAlgorithm::SHA512_256,
        ]),
        96 => Ok(vec![SupportedAlgorithm::SHA384]),
        128 => Ok(vec![SupportedAlgorithm::SHA512]),
        32 => Ok(vec![
            SupportedAlgorithm::MD5,
            SupportedAlgorithm::MD4,
            SupportedAlgorithm::MD2,
        ]),
        _ => Err(String::from("Error: Invalid hash.")),
    }
}

#[cfg(feature = "ring_backend")]
pub fn detect_hash_algorithm<S: AsRef<str>>(hash: S) -> Result<Vec<SupportedAlgorithm>, String> {
    match hash.as_ref().len() {
        64 => Ok(vec![
            SupportedAlgorithm::SHA256,
            SupportedAlgorithm::SHA512_256,
        ]),
        96 => Ok(vec![SupportedAlgorithm::SHA384]),
        128 => Ok(vec![SupportedAlgorithm::SHA512]),
        _ => Err(String::from("Error: Invalid hash.")),
    }
}

#[cfg(test)]
mod test_extra {
    use super::*;
    use crate::calculator::SupportedAlgorithm;

    // #[test]
    // fn test_transform_of_bytes_and_hex() {
    //     let hex = "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95";
    //     assert_eq!(
    //         extra::bytes_to_hex(&hex_to_bytes(hex).unwrap()),
    //         hex
    //     )
    // }

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
}
