#[cfg(not(any(feature = "hashes_backend", feature = "ring_backend")))]
compile_error!("You must enable at least one backend feature: 'hashes_backend' or 'ring_backend'.");

mod calculator;
mod error;

pub use calculator::SupportedAlgorithm;
pub use error::EzcheckError;

pub type Result<T> = std::result::Result<T, EzcheckError>;

#[cfg(unix)]
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// 对一个输入执行全部候选算法，并保留它所属的单条校验记录。
pub struct Verification {
    data: Data,
    expected_hash: String,
    algorithms: Vec<SupportedAlgorithm>,
}

impl Verification {
    pub fn new(data: Data, resolved_hash: ResolvedHashInput) -> Self {
        Self {
            data,
            expected_hash: resolved_hash.hash,
            algorithms: resolved_hash.algorithms,
        }
    }

    pub fn compute(&self) -> Result<Vec<VerificationOutcome>> {
        let results = self.data.compute_hashes(&self.algorithms)?;
        Ok(results
            .into_iter()
            .map(|(algorithm, hash)| compare_hash_result(algorithm, &self.expected_hash, hash))
            .collect())
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationOutcome {
    Match {
        algorithm: SupportedAlgorithm,
    },
    Failed {
        algorithm: SupportedAlgorithm,
        current_hash: String,
    },
}

fn compare_hash_result(
    algorithm: SupportedAlgorithm,
    expected_hash: &str,
    current_hash: String,
) -> VerificationOutcome {
    if current_hash.eq_ignore_ascii_case(expected_hash) {
        VerificationOutcome::Match { algorithm }
    } else {
        VerificationOutcome::Failed {
            algorithm,
            current_hash,
        }
    }
}

#[derive(Clone)]
pub enum Data {
    File(PathBuf),
    Stdin,
    Text(String),
}

impl Data {
    pub fn from_path(path: PathBuf) -> Self {
        if path.as_os_str() == "-" {
            Self::Stdin
        } else {
            Self::File(path)
        }
    }

    pub fn calculate(&self, algorithm: SupportedAlgorithm) -> Result<String> {
        let mut hashes = self.compute_hashes(&[algorithm])?;
        Ok(hashes
            .pop()
            .expect("a single requested algorithm always produces one hash")
            .1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedHashInput {
    hash: String,
    algorithms: Vec<SupportedAlgorithm>,
    detected_from_hash: bool,
}

impl ResolvedHashInput {
    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn algorithms(&self) -> &[SupportedAlgorithm] {
        &self.algorithms
    }

    pub fn detected_from_hash(&self) -> bool {
        self.detected_from_hash
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Data::File(path) => return write!(f, "{}", path.display()),
            Data::Stdin => "-",
            Data::Text(text) => text,
        };
        write!(f, "{value}")
    }
}

fn compute_hashes_from_reader<R: BufRead>(
    reader: R,
    algorithms: &[SupportedAlgorithm],
) -> Result<Vec<(SupportedAlgorithm, String)>> {
    calculator::calculate_hashes(reader, algorithms).map_err(EzcheckError::CalculateHash)
}

impl Data {
    fn compute_hashes(
        &self,
        algorithms: &[SupportedAlgorithm],
    ) -> Result<Vec<(SupportedAlgorithm, String)>> {
        match self {
            Data::Stdin => compute_hashes_from_reader(stdin().lock(), algorithms),
            Data::File(path) => {
                let file = File::open(path).map_err(|source| EzcheckError::OpenFile {
                    path: path.clone(),
                    source,
                })?;
                compute_hashes_from_reader(BufReader::new(file), algorithms)
            }
            Data::Text(text) => {
                compute_hashes_from_reader(BufReader::new(text.as_bytes()), algorithms)
            }
        }
    }
}

fn validate_hash_for_algorithm(hash: &str, algorithm: SupportedAlgorithm) -> Result<()> {
    let detected_algorithms = SupportedAlgorithm::detect_from_hash(hash)?;

    if detected_algorithms.contains(&algorithm) {
        Ok(())
    } else {
        Err(EzcheckError::HashAlgorithmMismatch(algorithm))
    }
}

fn parse_hash_input<S: AsRef<str>>(hash_input: S) -> Result<(Option<SupportedAlgorithm>, String)> {
    let hash_input = hash_input.as_ref().trim();
    if hash_input.is_empty() {
        return Err(EzcheckError::InvalidHash(String::new()));
    }

    if let Some((algorithm_name, hash)) = hash_input.split_once(':') {
        let algorithm_name = algorithm_name.trim();
        let hash = hash.trim();

        if algorithm_name.is_empty() || hash.is_empty() {
            return Err(EzcheckError::InvalidHash(hash_input.to_string()));
        }

        let algorithm = SupportedAlgorithm::from_input(algorithm_name)?;
        validate_hash_for_algorithm(hash, algorithm)?;
        Ok((Some(algorithm), hash.to_string()))
    } else {
        Ok((None, hash_input.to_string()))
    }
}

pub fn resolve_hash_input<S: AsRef<str>>(
    hash_input: S,
    algorithm: Option<SupportedAlgorithm>,
) -> Result<ResolvedHashInput> {
    let (prefixed_algorithm, hash) = parse_hash_input(hash_input)?;

    let algorithms = match (algorithm, prefixed_algorithm) {
        (Some(specified_algorithm), Some(prefixed_algorithm))
            if specified_algorithm != prefixed_algorithm =>
        {
            return Err(EzcheckError::ConflictingAlgorithms {
                specified: specified_algorithm,
                prefixed: prefixed_algorithm,
            });
        }
        (Some(specified_algorithm), None) => {
            validate_hash_for_algorithm(&hash, specified_algorithm)?;
            vec![specified_algorithm]
        }
        (Some(specified_algorithm), Some(_)) => vec![specified_algorithm],
        (None, Some(prefixed_algorithm)) => vec![prefixed_algorithm],
        (None, None) => SupportedAlgorithm::detect_from_hash(&hash)?,
    };

    Ok(ResolvedHashInput {
        hash,
        algorithms,
        detected_from_hash: algorithm.is_none() && prefixed_algorithm.is_none(),
    })
}

fn invalid_shasum_line(line: usize) -> EzcheckError {
    EzcheckError::InvalidShasumLine { line }
}

fn unescape_shasum_path(path: &[u8], line_number: usize) -> Result<Vec<u8>> {
    let mut result = Vec::with_capacity(path.len());
    let mut index = 0;

    while index < path.len() {
        if path[index] != b'\\' {
            result.push(path[index]);
            index += 1;
            continue;
        }

        let escaped = *path
            .get(index + 1)
            .ok_or_else(|| invalid_shasum_line(line_number))?;
        match escaped {
            b'\\' => result.push(b'\\'),
            b'n' => result.push(b'\n'),
            b'r' => result.push(b'\r'),
            _ => return Err(invalid_shasum_line(line_number)),
        }
        index += 2;
    }

    Ok(result)
}

#[cfg(unix)]
fn path_from_bytes(path: Vec<u8>, _line_number: usize) -> Result<PathBuf> {
    Ok(PathBuf::from(OsString::from_vec(path)))
}

#[cfg(not(unix))]
fn path_from_bytes(path: Vec<u8>, line_number: usize) -> Result<PathBuf> {
    String::from_utf8(path)
        .map(PathBuf::from)
        .map_err(|_| invalid_shasum_line(line_number))
}

fn parse_shasum_line(line: &[u8], line_number: usize) -> Result<Option<(String, PathBuf)>> {
    let mut line = line;
    if line.ends_with(b"\n") {
        line = &line[..line.len() - 1];
    }
    if line.ends_with(b"\r") {
        line = &line[..line.len() - 1];
    }
    if line.iter().all(|byte| byte.is_ascii_whitespace()) {
        return Ok(None);
    }

    let escaped = line.starts_with(b"\\");
    if escaped {
        line = &line[1..];
    }

    let separator = line
        .iter()
        .position(|byte| *byte == b' ')
        .ok_or_else(|| invalid_shasum_line(line_number))?;
    let mode = line
        .get(separator + 1)
        .copied()
        .ok_or_else(|| invalid_shasum_line(line_number))?;
    if mode != b' ' && mode != b'*' {
        return Err(invalid_shasum_line(line_number));
    }

    let hash = std::str::from_utf8(&line[..separator])
        .map_err(|_| invalid_shasum_line(line_number))?
        .to_string();
    let raw_path = line
        .get(separator + 2..)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| invalid_shasum_line(line_number))?;
    let path = if escaped {
        unescape_shasum_path(raw_path, line_number)?
    } else {
        raw_path.to_vec()
    };

    Ok(Some((hash, path_from_bytes(path, line_number)?)))
}

/// 将校验文件解析为彼此独立的校验记录。
pub fn parse_shasum_file<P: AsRef<Path>>(
    shasum_file_path: P,
    algorithm: Option<SupportedAlgorithm>,
) -> Result<Vec<Verification>> {
    /*
    Example shasum file:
        ee1fb7719c31070f1fbdc8f2d32370c9d1ca6962  image.png
        ee1fb7719c31070f1fbdc8f2d32370c9d1ca6962 *image.png
                                                 ^ In binary mode, neglected.
     */
    let shasum_file_path = shasum_file_path.as_ref();
    let file = File::open(shasum_file_path).map_err(|source| EzcheckError::OpenFile {
        path: shasum_file_path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut line = Vec::new();
    let mut line_number = 0;

    loop {
        line.clear();
        let read =
            reader
                .read_until(b'\n', &mut line)
                .map_err(|source| EzcheckError::ReadShasumFile {
                    path: shasum_file_path.to_path_buf(),
                    source,
                })?;
        if read == 0 {
            break;
        }
        line_number += 1;

        let Some((hash, file_path)) = parse_shasum_line(&line, line_number)? else {
            continue;
        };

        let resolved_hash = resolve_hash_input(&hash, algorithm)?;

        entries.push(Verification::new(Data::from_path(file_path), resolved_hash));
    }

    Ok(entries)
}

#[cfg(test)]
mod test_core {
    use super::{
        parse_shasum_file, parse_shasum_line, resolve_hash_input, Data, SupportedAlgorithm,
        Verification,
    };
    use crate::EzcheckError;
    use crate::VerificationOutcome::{Failed, Match};
    use std::path::PathBuf;

    fn verification(
        data: Data,
        expected_hash: &str,
        algorithm: SupportedAlgorithm,
    ) -> Verification {
        let resolved_hash = resolve_hash_input(expected_hash, Some(algorithm)).unwrap();
        Verification::new(data, resolved_hash)
    }

    #[test]
    fn test_calculate_compute_hash_file() {
        let data = Data::File(PathBuf::from("tests/滕王阁序.txt"));
        assert_eq!(
            data.calculate(SupportedAlgorithm::SHA256).unwrap(),
            "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        );
    }

    #[test]
    fn test_calculate_compute_hash_text() {
        let data = Data::Text(String::from("Veni, vidi, vici"));
        assert_eq!(
            data.calculate(SupportedAlgorithm::SHA256).unwrap(),
            "b1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466"
        );
    }

    #[test]
    fn test_compare_hash_file() {
        let task = verification(
            Data::File(PathBuf::from("tests/滕王阁序.txt")),
            "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95",
            SupportedAlgorithm::SHA256,
        );
        assert_eq!(
            task.compute().unwrap(),
            vec![Match {
                algorithm: SupportedAlgorithm::SHA256,
            }]
        )
    }

    #[test]
    fn test_compare_hash_text() {
        let task = verification(
            Data::Text(String::from("Veni, vidi, vici")),
            "a1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466",
            SupportedAlgorithm::SHA256,
        );
        assert_eq!(
            task.compute().unwrap(),
            vec![Failed {
                algorithm: SupportedAlgorithm::SHA256,
                current_hash: String::from(
                    "b1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466"
                ),
            }]
        )
    }

    #[test]
    fn test_compare_hash_text_is_case_insensitive() {
        let task = verification(
            Data::Text(String::from("Veni, vidi, vici")),
            "B1610284C94BBF9AA78333E57DDCE234A5E845D61E09CE91A7E19FA24737F466",
            SupportedAlgorithm::SHA256,
        );
        assert_eq!(
            task.compute().unwrap(),
            vec![Match {
                algorithm: SupportedAlgorithm::SHA256,
            }]
        )
    }

    #[test]
    fn test_parse_shasum_file_preserves_relative_paths() {
        let entries =
            parse_shasum_file("tests/sha256sum.txt", Some(SupportedAlgorithm::SHA256)).unwrap();

        assert_eq!(entries.len(), 2);
        assert!(matches!(
            entries[0].data(),
            Data::File(path) if path == &PathBuf::from("滕王阁序.txt")
        ));
        assert!(matches!(
            entries[1].data(),
            Data::File(path) if path == &PathBuf::from("image.jpg")
        ));
    }

    #[test]
    fn test_parse_shasum_file_supports_prefixed_hashes() {
        let entries = parse_shasum_file("tests/prefixed-shasum.txt", None).unwrap();

        assert_eq!(entries.len(), 2);
        for entry in &entries {
            assert_eq!(entry.algorithms, vec![SupportedAlgorithm::SHA256]);
        }
        assert!(matches!(
            entries[0].data(),
            Data::File(path) if path == &PathBuf::from("滕王阁序.txt")
        ));
        assert!(matches!(
            entries[1].data(),
            Data::File(path) if path == &PathBuf::from("image.jpg")
        ));
    }

    #[test]
    fn test_resolve_hash_input_supports_prefixed_hashes() {
        let resolved = resolve_hash_input(
            "ShA256:00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95",
            None,
        )
        .unwrap();

        assert_eq!(resolved.algorithms(), &[SupportedAlgorithm::SHA256]);
        assert_eq!(
            resolved.hash(),
            "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        );
        assert!(!resolved.detected_from_hash());
    }

    #[test]
    fn test_resolve_hash_input_rejects_conflicting_algorithms() {
        let error = resolve_hash_input(
            "sha512/256:00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95",
            Some(SupportedAlgorithm::SHA256),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            EzcheckError::ConflictingAlgorithms {
                specified: SupportedAlgorithm::SHA256,
                prefixed: SupportedAlgorithm::SHA512_256,
            }
        ));
    }

    #[test]
    fn test_explicit_algorithm_rejects_an_invalid_hash_with_a_typed_error() {
        let error = resolve_hash_input("not-a-hash", Some(SupportedAlgorithm::SHA256)).unwrap_err();

        assert!(matches!(
            error,
            EzcheckError::InvalidHash(hash) if hash == "not-a-hash"
        ));
    }

    #[test]
    fn test_invalid_shasum_line_reports_its_line_number() {
        let error = parse_shasum_line(b"not-a-shasum-line\n", 7).unwrap_err();

        assert!(matches!(error, EzcheckError::InvalidShasumLine { line: 7 }));
    }

    #[test]
    fn test_supported_algorithm_supports_xxhash_aliases() {
        assert_eq!(
            SupportedAlgorithm::from_input("xxh64").unwrap(),
            SupportedAlgorithm::XXHASH64
        );
    }

    #[test]
    fn test_supported_algorithm_supports_case_insensitive_aliases() {
        assert_eq!(
            SupportedAlgorithm::from_input("sHa512/256").unwrap(),
            SupportedAlgorithm::SHA512_256
        );
    }
}
