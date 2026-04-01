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

use crate::extra::bytes_to_hex;
use core::hash::Hasher;
use std::fmt;
use std::io::{BufRead, Error};

#[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
use digest::DynDigest;
#[cfg(any(feature = "ring_backend", feature = "mix_backend"))]
use ring::digest::{Algorithm, Context, SHA256, SHA384, SHA512, SHA512_256};
use twox_hash::{XxHash32, XxHash3_64, XxHash64};

/*
* Why we set BUFFER_SIZE as 8192
    https://doc.rust-lang.org/std/io/struct.BufReader.html#impl-BufReader%3CR%3E
* Why we set allow(dead_code)
    https://github.com/rust-lang/rust/issues/47133
*/
#[allow(dead_code)]
pub const BUFFER_SIZE: usize = 8192;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SupportedAlgorithm {
    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    MD2,
    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    MD4,
    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    MD5,
    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    SHA1,
    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    SHA224,
    SHA256,
    SHA384,
    SHA512,
    SHA512_256,
    XXHASH32,
    XXHASH64,
    XXHASH3_64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum AlgorithmBackend {
    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    Digest,
    #[cfg(any(feature = "ring_backend", feature = "mix_backend"))]
    Ring,
    Xxhash,
}

impl SupportedAlgorithm {
    pub fn from_input<S: AsRef<str>>(algorithm: S) -> Result<Self, String> {
        let algorithm = algorithm.as_ref().trim();
        let normalized = algorithm.to_ascii_lowercase();

        match normalized.as_str() {
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            "md2" => Ok(Self::MD2),
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            "md4" => Ok(Self::MD4),
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            "md5" => Ok(Self::MD5),
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            "sha1" => Ok(Self::SHA1),
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            "sha224" => Ok(Self::SHA224),
            "sha256" => Ok(Self::SHA256),
            "sha384" => Ok(Self::SHA384),
            "sha512" => Ok(Self::SHA512),
            "sha512_256" | "sha512-256" | "sha512/256" => Ok(Self::SHA512_256),
            "xxhash32" | "xxh32" => Ok(Self::XXHASH32),
            "xxhash64" | "xxh64" => Ok(Self::XXHASH64),
            "xxh3" | "xxh3_64" | "xxh3-64" | "xxh3/64" | "xxhash3" | "xxhash3_64"
            | "xxhash3-64" | "xxhash3/64" => Ok(Self::XXHASH3_64),
            _ => Err(format!("Error: Unsupported algorithm: {}", algorithm)),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::MD2 => "MD2",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::MD4 => "MD4",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::MD5 => "MD5",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::SHA1 => "SHA1",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::SHA224 => "SHA224",
            SupportedAlgorithm::SHA256 => "SHA256",
            SupportedAlgorithm::SHA384 => "SHA384",
            SupportedAlgorithm::SHA512 => "SHA512",
            SupportedAlgorithm::SHA512_256 => "SHA512_256",
            SupportedAlgorithm::XXHASH32 => "XXHASH32",
            SupportedAlgorithm::XXHASH64 => "XXHASH64",
            SupportedAlgorithm::XXHASH3_64 => "XXHASH3_64",
        }
    }

    pub const fn prefixed_hash_name(self) -> &'static str {
        match self {
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::MD2 => "md2",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::MD4 => "md4",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::MD5 => "md5",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::SHA1 => "sha1",
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::SHA224 => "sha224",
            SupportedAlgorithm::SHA256 => "sha256",
            SupportedAlgorithm::SHA384 => "sha384",
            SupportedAlgorithm::SHA512 => "sha512",
            SupportedAlgorithm::SHA512_256 => "sha512/256",
            SupportedAlgorithm::XXHASH32 => "xxhash32",
            SupportedAlgorithm::XXHASH64 => "xxhash64",
            SupportedAlgorithm::XXHASH3_64 => "xxh3_64",
        }
    }

    const fn backend(self) -> AlgorithmBackend {
        match self {
            SupportedAlgorithm::XXHASH32
            | SupportedAlgorithm::XXHASH64
            | SupportedAlgorithm::XXHASH3_64 => AlgorithmBackend::Xxhash,
            #[cfg(any(feature = "ring_backend", feature = "mix_backend"))]
            SupportedAlgorithm::SHA256
            | SupportedAlgorithm::SHA384
            | SupportedAlgorithm::SHA512
            | SupportedAlgorithm::SHA512_256 => AlgorithmBackend::Ring,
            #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
            SupportedAlgorithm::MD2
            | SupportedAlgorithm::MD4
            | SupportedAlgorithm::MD5
            | SupportedAlgorithm::SHA1
            | SupportedAlgorithm::SHA224 => AlgorithmBackend::Digest,
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::SHA256
            | SupportedAlgorithm::SHA384
            | SupportedAlgorithm::SHA512
            | SupportedAlgorithm::SHA512_256 => AlgorithmBackend::Digest,
        }
    }
}

impl fmt::Display for SupportedAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn consume_reader<R: BufRead, F>(reader: &mut R, mut update: F) -> Result<(), Error>
where
    F: FnMut(&[u8]),
{
    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let read_bytes = reader.read(&mut buffer)?;
        if read_bytes == 0 {
            return Ok(());
        }

        update(&buffer[..read_bytes]);
    }
}

#[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
fn hash_with_digest<R: BufRead>(
    reader: &mut R,
    algorithm: SupportedAlgorithm,
) -> Result<String, Error> {
    let mut hasher: Box<dyn DynDigest> = match algorithm {
        SupportedAlgorithm::MD2 => Box::new(md2::Md2::default()),
        SupportedAlgorithm::MD4 => Box::new(md4::Md4::default()),
        SupportedAlgorithm::MD5 => Box::new(md5::Md5::default()),
        SupportedAlgorithm::SHA1 => Box::new(sha1::Sha1::default()),
        SupportedAlgorithm::SHA224 => Box::new(sha2::Sha224::default()),
        #[cfg(feature = "hashes_backend")]
        SupportedAlgorithm::SHA256 => Box::new(sha2::Sha256::default()),
        #[cfg(feature = "hashes_backend")]
        SupportedAlgorithm::SHA384 => Box::new(sha2::Sha384::default()),
        #[cfg(feature = "hashes_backend")]
        SupportedAlgorithm::SHA512 => Box::new(sha2::Sha512::default()),
        #[cfg(feature = "hashes_backend")]
        SupportedAlgorithm::SHA512_256 => Box::new(sha2::Sha512_256::default()),
        _ => unreachable!("non-digest algorithms are handled separately"),
    };

    consume_reader(reader, |chunk| hasher.update(chunk))?;

    let digest = hasher.finalize_reset();
    Ok(bytes_to_hex(digest.as_ref()))
}

#[cfg(any(feature = "ring_backend", feature = "mix_backend"))]
fn ring_algorithm(algorithm: SupportedAlgorithm) -> &'static Algorithm {
    match algorithm {
        SupportedAlgorithm::SHA256 => &SHA256,
        SupportedAlgorithm::SHA384 => &SHA384,
        SupportedAlgorithm::SHA512 => &SHA512,
        SupportedAlgorithm::SHA512_256 => &SHA512_256,
        _ => unreachable!("non-ring algorithms are handled separately"),
    }
}

#[cfg(any(feature = "ring_backend", feature = "mix_backend"))]
fn hash_with_ring<R: BufRead>(
    reader: &mut R,
    algorithm: SupportedAlgorithm,
) -> Result<String, Error> {
    let mut hasher = Context::new(ring_algorithm(algorithm));
    consume_reader(reader, |chunk| hasher.update(chunk))?;

    let digest = hasher.finish();
    Ok(bytes_to_hex(digest.as_ref()))
}

fn hash_with_xxhash<R: BufRead>(
    reader: &mut R,
    algorithm: SupportedAlgorithm,
) -> Result<String, Error> {
    let mut hasher: Box<dyn Hasher> = match algorithm {
        SupportedAlgorithm::XXHASH32 => Box::new(XxHash32::with_seed(0)),
        SupportedAlgorithm::XXHASH64 => Box::new(XxHash64::with_seed(0)),
        SupportedAlgorithm::XXHASH3_64 => Box::new(XxHash3_64::with_seed(0)),
        _ => unreachable!("non-xxhash algorithms are handled separately"),
    };

    consume_reader(reader, |chunk| hasher.write(chunk))?;

    let hash = hasher.finish();
    Ok(match algorithm {
        SupportedAlgorithm::XXHASH32 => format!("{hash:08x}"),
        SupportedAlgorithm::XXHASH64 => format!("{hash:016x}"),
        SupportedAlgorithm::XXHASH3_64 => format!("XXH3_{hash:016x}"),
        _ => unreachable!("non-xxhash algorithms are handled separately"),
    })
}

pub fn hash_calculator<R: BufRead>(
    mut reader: R,
    algorithm: SupportedAlgorithm,
) -> Result<String, Error> {
    match algorithm.backend() {
        AlgorithmBackend::Xxhash => hash_with_xxhash(&mut reader, algorithm),
        #[cfg(any(feature = "ring_backend", feature = "mix_backend"))]
        AlgorithmBackend::Ring => hash_with_ring(&mut reader, algorithm),
        #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
        AlgorithmBackend::Digest => hash_with_digest(&mut reader, algorithm),
    }
}

#[cfg(test)]
mod test_calculator {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    const TEST_WORD: &[u8; 16] = b"Veni, vidi, vici";

    #[test]
    fn test_xxhash32() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::XXHASH32).unwrap(),
            "0163d3a2"
        );
    }

    #[test]
    fn test_xxhash64() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::XXHASH64).unwrap(),
            "4a34911ba20e6c30"
        );
    }

    #[test]
    fn test_xxhash3_64() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::XXHASH3_64).unwrap(),
            "XXH3_802c0db623389036"
        );
    }

    #[test]
    fn test_supported_algorithm_from_input_accepts_case_insensitive_aliases() {
        assert_eq!(
            SupportedAlgorithm::from_input("sHa512/256").unwrap(),
            SupportedAlgorithm::SHA512_256
        );
    }

    #[test]
    fn test_supported_algorithm_prefixed_hash_name_is_canonical() {
        assert_eq!(SupportedAlgorithm::SHA256.prefixed_hash_name(), "sha256");
        assert_eq!(
            SupportedAlgorithm::SHA512_256.prefixed_hash_name(),
            "sha512/256"
        );
        assert_eq!(
            SupportedAlgorithm::XXHASH3_64.prefixed_hash_name(),
            "xxh3_64"
        );
    }

    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    #[test]
    fn test_md2() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::MD2).unwrap(),
            "3354cef96052efb872e8c0391a5cfb34"
        );
    }

    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    #[test]
    fn test_md4() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::MD4).unwrap(),
            "5c79b96c023c5a269ad205d33bce0f60"
        );
    }

    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    #[test]
    fn test_md5() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::MD5).unwrap(),
            "af1e16b12fec10c5ad09fb6478005b6c"
        );
    }

    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    #[test]
    fn test_sha1() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA1).unwrap(),
            "5df99149d56d7f82a9751ac4c36ada25d07f5e49"
        );
    }

    #[cfg(any(feature = "hashes_backend", feature = "mix_backend"))]
    #[test]
    fn test_sha224() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA224).unwrap(),
            "9111df25d5715bc4ab42d6777f48d1bd592f7f991fbbc356ae370167"
        );
    }

    #[test]
    fn test_sha256() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA256).unwrap(),
            "b1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466"
        );
    }

    #[test]
    fn test_sha384() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA384).unwrap(),
            "aed14590fa99f83c701236d63c50085faa8e57c7196846411dc595c42751e5e17d6bc10b767541d76eecdda086c5d4fc"
        );
    }

    #[test]
    fn test_sha512() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA512).unwrap(),
            "6cf905a2c09fa2d9090f2712e2ae6d0fc8188cc845a1dc9dff4b3bd33e9d4fa43991cbb7cc3cf5d5aa8e32098796eb01e3f03c25c6ea863226e617ad6e5abec2"
        );
    }

    #[test]
    fn test_sha512_256() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA512_256).unwrap(),
            "aea4f1ce7ac12b2374482816aa44d33935fb445d8e8892aeb501c82a97f76d8d"
        );
    }

    #[test]
    fn test_read_file() {
        let test_file = "tests/滕王阁序.txt";
        let file = File::open(test_file).unwrap();
        let reader = BufReader::new(file);

        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA256).unwrap(),
            "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        );
    }
}
