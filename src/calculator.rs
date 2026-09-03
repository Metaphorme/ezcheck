use crate::EzcheckError;
use clap::ValueEnum;
use core::hash::Hasher;
use std::fmt::{self, Write};
use std::io::{BufRead, Error};

#[cfg(feature = "hashes_backend")]
use digest::DynDigest;
#[cfg(feature = "ring_backend")]
use ring::digest::{Algorithm, Context, SHA256, SHA384, SHA512, SHA512_256};
use twox_hash::{XxHash32, XxHash3_64, XxHash64};

// 与标准库 BufReader 的默认容量保持一致。
const BUFFER_SIZE: usize = 8192;

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(hex_string, "{byte:02x}").expect("writing to a String cannot fail");
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum SupportedAlgorithm {
    #[cfg(feature = "hashes_backend")]
    #[value(name = "md2", help = "Legacy algorithm; unsafe for security use")]
    MD2,
    #[cfg(feature = "hashes_backend")]
    #[value(name = "md4", help = "Legacy algorithm; unsafe for security use")]
    MD4,
    #[cfg(feature = "hashes_backend")]
    #[value(name = "md5", help = "Legacy algorithm; unsafe for security use")]
    MD5,
    #[cfg(feature = "hashes_backend")]
    #[value(name = "sha1", help = "Legacy algorithm; unsafe for security use")]
    SHA1,
    #[cfg(feature = "hashes_backend")]
    #[value(name = "sha224")]
    SHA224,
    #[value(name = "sha256")]
    SHA256,
    #[value(name = "sha384")]
    SHA384,
    #[value(name = "sha512")]
    SHA512,
    #[value(name = "sha512_256", alias("sha512-256"), alias("sha512/256"))]
    SHA512_256,
    #[value(name = "xxhash32", alias("xxh32"))]
    XXHASH32,
    #[value(name = "xxhash64", alias("xxh64"))]
    XXHASH64,
    #[value(
        name = "xxhash3_64",
        alias("xxh3"),
        alias("xxh3_64"),
        alias("xxh3-64"),
        alias("xxh3/64"),
        alias("xxhash3"),
        alias("xxhash3-64"),
        alias("xxhash3/64")
    )]
    XXHASH3_64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum AlgorithmBackend {
    #[cfg(feature = "hashes_backend")]
    Digest,
    #[cfg(feature = "ring_backend")]
    Ring,
    Xxhash,
}

impl SupportedAlgorithm {
    pub fn from_input<S: AsRef<str>>(algorithm: S) -> Result<Self, EzcheckError> {
        let algorithm = algorithm.as_ref().trim();
        <Self as ValueEnum>::from_str(algorithm, true)
            .map_err(|_| EzcheckError::UnsupportedAlgorithm(algorithm.to_string()))
    }

    pub fn detect_from_hash<S: AsRef<str>>(hash: S) -> Result<Vec<Self>, EzcheckError> {
        let hash = hash.as_ref();

        match hash.len() {
            8 if is_ascii_hex(hash) => Ok(vec![Self::XXHASH32]),
            16 if is_ascii_hex(hash) => Ok(vec![Self::XXHASH64]),
            21 if is_xxh3_64(hash) => Ok(vec![Self::XXHASH3_64]),
            #[cfg(feature = "hashes_backend")]
            32 if is_ascii_hex(hash) => Ok(vec![Self::MD5, Self::MD4, Self::MD2]),
            #[cfg(feature = "hashes_backend")]
            40 if is_ascii_hex(hash) => Ok(vec![Self::SHA1]),
            #[cfg(feature = "hashes_backend")]
            56 if is_ascii_hex(hash) => Ok(vec![Self::SHA224]),
            64 if is_ascii_hex(hash) => Ok(vec![Self::SHA256, Self::SHA512_256]),
            96 if is_ascii_hex(hash) => Ok(vec![Self::SHA384]),
            128 if is_ascii_hex(hash) => Ok(vec![Self::SHA512]),
            _ => Err(EzcheckError::InvalidHash(hash.to_string())),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::MD2 => "MD2",
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::MD4 => "MD4",
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::MD5 => "MD5",
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::SHA1 => "SHA1",
            #[cfg(feature = "hashes_backend")]
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
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::MD2 => "md2",
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::MD4 => "md4",
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::MD5 => "md5",
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::SHA1 => "sha1",
            #[cfg(feature = "hashes_backend")]
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
            #[cfg(feature = "ring_backend")]
            SupportedAlgorithm::SHA256
            | SupportedAlgorithm::SHA384
            | SupportedAlgorithm::SHA512
            | SupportedAlgorithm::SHA512_256 => AlgorithmBackend::Ring,
            #[cfg(feature = "hashes_backend")]
            SupportedAlgorithm::MD2
            | SupportedAlgorithm::MD4
            | SupportedAlgorithm::MD5
            | SupportedAlgorithm::SHA1
            | SupportedAlgorithm::SHA224 => AlgorithmBackend::Digest,
            #[cfg(all(feature = "hashes_backend", not(feature = "ring_backend")))]
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

#[cfg(feature = "ring_backend")]
fn ring_algorithm(algorithm: SupportedAlgorithm) -> &'static Algorithm {
    match algorithm {
        SupportedAlgorithm::SHA256 => &SHA256,
        SupportedAlgorithm::SHA384 => &SHA384,
        SupportedAlgorithm::SHA512 => &SHA512,
        SupportedAlgorithm::SHA512_256 => &SHA512_256,
        _ => unreachable!("non-ring algorithms are handled separately"),
    }
}

enum HashContext {
    #[cfg(feature = "hashes_backend")]
    Digest(Box<dyn DynDigest>),
    #[cfg(feature = "ring_backend")]
    Ring(Box<Context>),
    Xxhash(Box<dyn Hasher>),
}

impl HashContext {
    fn for_algorithm(algorithm: SupportedAlgorithm) -> Self {
        match algorithm.backend() {
            #[cfg(feature = "hashes_backend")]
            AlgorithmBackend::Digest => {
                let hasher: Box<dyn DynDigest> = match algorithm {
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
                Self::Digest(hasher)
            }
            #[cfg(feature = "ring_backend")]
            AlgorithmBackend::Ring => Self::Ring(Box::new(Context::new(ring_algorithm(algorithm)))),
            AlgorithmBackend::Xxhash => {
                let hasher: Box<dyn Hasher> = match algorithm {
                    SupportedAlgorithm::XXHASH32 => Box::new(XxHash32::with_seed(0)),
                    SupportedAlgorithm::XXHASH64 => Box::new(XxHash64::with_seed(0)),
                    SupportedAlgorithm::XXHASH3_64 => Box::new(XxHash3_64::with_seed(0)),
                    _ => unreachable!("non-xxhash algorithms are handled separately"),
                };
                Self::Xxhash(hasher)
            }
        }
    }

    fn update(&mut self, chunk: &[u8]) {
        match self {
            #[cfg(feature = "hashes_backend")]
            Self::Digest(hasher) => hasher.update(chunk),
            #[cfg(feature = "ring_backend")]
            Self::Ring(hasher) => hasher.update(chunk),
            Self::Xxhash(hasher) => hasher.write(chunk),
        }
    }

    fn finish(self, algorithm: SupportedAlgorithm) -> String {
        match self {
            #[cfg(feature = "hashes_backend")]
            Self::Digest(hasher) => bytes_to_hex(hasher.finalize().as_ref()),
            #[cfg(feature = "ring_backend")]
            Self::Ring(hasher) => bytes_to_hex((*hasher).finish().as_ref()),
            Self::Xxhash(hasher) => {
                let hash = hasher.finish();
                match algorithm {
                    SupportedAlgorithm::XXHASH32 => format!("{hash:08x}"),
                    SupportedAlgorithm::XXHASH64 => format!("{hash:016x}"),
                    SupportedAlgorithm::XXHASH3_64 => format!("XXH3_{hash:016x}"),
                    _ => unreachable!("non-xxhash algorithms are handled separately"),
                }
            }
        }
    }
}

struct ActiveHasher {
    algorithm: SupportedAlgorithm,
    context: HashContext,
}

impl ActiveHasher {
    fn new(algorithm: SupportedAlgorithm) -> Self {
        Self {
            algorithm,
            context: HashContext::for_algorithm(algorithm),
        }
    }

    fn finish(self) -> (SupportedAlgorithm, String) {
        (self.algorithm, self.context.finish(self.algorithm))
    }
}

pub(crate) fn calculate_hashes<R: BufRead>(
    mut reader: R,
    algorithms: &[SupportedAlgorithm],
) -> Result<Vec<(SupportedAlgorithm, String)>, Error> {
    let mut hashers: Vec<ActiveHasher> =
        algorithms.iter().copied().map(ActiveHasher::new).collect();

    consume_reader(&mut reader, |chunk| {
        for hasher in &mut hashers {
            hasher.context.update(chunk);
        }
    })?;

    Ok(hashers.into_iter().map(ActiveHasher::finish).collect())
}

#[cfg(test)]
fn hash_calculator<R: BufRead>(reader: R, algorithm: SupportedAlgorithm) -> Result<String, Error> {
    let mut hashes = calculate_hashes(reader, &[algorithm])?;
    Ok(hashes
        .pop()
        .expect("a single requested algorithm always produces one hash")
        .1)
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

    #[test]
    fn test_detect_hash_algorithm() {
        assert_eq!(
            SupportedAlgorithm::detect_from_hash(
                "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
            )
            .unwrap(),
            vec![SupportedAlgorithm::SHA256, SupportedAlgorithm::SHA512_256,]
        );
    }

    #[test]
    fn test_detect_hash_algorithm_rejects_invalid_hex() {
        assert!(SupportedAlgorithm::detect_from_hash(
            "zz691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        )
        .is_err());
    }

    #[test]
    fn test_detect_hash_algorithm_accepts_lowercase_xxh3_prefix() {
        assert_eq!(
            SupportedAlgorithm::detect_from_hash("xxh3_802c0db623389036").unwrap(),
            vec![SupportedAlgorithm::XXHASH3_64]
        );
    }

    #[test]
    fn test_detect_hash_algorithm_xxhash64() {
        assert_eq!(
            SupportedAlgorithm::detect_from_hash("4a34911ba20e6c30").unwrap(),
            vec![SupportedAlgorithm::XXHASH64]
        );
    }

    #[cfg(feature = "hashes_backend")]
    #[test]
    fn test_md2() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::MD2).unwrap(),
            "3354cef96052efb872e8c0391a5cfb34"
        );
    }

    #[cfg(feature = "hashes_backend")]
    #[test]
    fn test_md4() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::MD4).unwrap(),
            "5c79b96c023c5a269ad205d33bce0f60"
        );
    }

    #[cfg(feature = "hashes_backend")]
    #[test]
    fn test_md5() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::MD5).unwrap(),
            "af1e16b12fec10c5ad09fb6478005b6c"
        );
    }

    #[cfg(feature = "hashes_backend")]
    #[test]
    fn test_sha1() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            hash_calculator(reader, SupportedAlgorithm::SHA1).unwrap(),
            "5df99149d56d7f82a9751ac4c36ada25d07f5e49"
        );
    }

    #[cfg(feature = "hashes_backend")]
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
