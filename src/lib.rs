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

pub mod calculator;
pub mod extra;

use std::fmt;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::path::Path;

pub struct Calculate {
    data: Data,
    algorithm: calculator::SupportedAlgorithm,
}

impl Calculate {
    pub fn new(data: Data, algorithm: calculator::SupportedAlgorithm) -> Calculate {
        Self { data, algorithm }
    }

    pub fn compute(&self) -> Result<String, String> {
        self.data.compute_hash(self.algorithm)
    }
}

pub struct Compare {
    pub data: Data,
    compare: String,
    algorithm: calculator::SupportedAlgorithm,
}

const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_RESET: &str = "\x1b[0m";

fn colorize(message: String, color: &str) -> String {
    format!("{color}{message}{ANSI_RESET}")
}

impl Compare {
    pub fn new(data: Data, compare: String, algorithm: calculator::SupportedAlgorithm) -> Compare {
        Self {
            data,
            compare,
            algorithm,
        }
    }

    pub fn expected_hash(&self) -> &str {
        &self.compare
    }

    pub fn compute(&self) -> Result<IfMatch, String> {
        let hash_result = self.data.compute_hash(self.algorithm)?;

        if hash_result.eq_ignore_ascii_case(&self.compare) {
            Ok(IfMatch::Match(colorize(
                format!("{} OK", self.algorithm),
                ANSI_GREEN,
            )))
        } else {
            Ok(IfMatch::Failed(format!(
                "{}  Current Hash:{}",
                colorize(format!("{} FAILED", self.algorithm), ANSI_RED),
                hash_result
            )))
        }
    }
}

#[derive(Debug)]
pub enum IfMatch {
    Match(String),
    Failed(String),
}

impl PartialEq for IfMatch {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (IfMatch::Match(_), IfMatch::Match(_)) | (IfMatch::Failed(_), IfMatch::Failed(_))
        )
    }
}

impl Eq for IfMatch {}

pub enum Data {
    ReadFile(String),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedHashInput {
    pub hash: String,
    pub algorithms: Vec<calculator::SupportedAlgorithm>,
    pub detected_from_hash: bool,
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Data::ReadFile(file_name) => file_name,
            Data::Text(text) => text,
        };
        write!(f, "{}", value)
    }
}

pub trait ComputeHash {
    fn compute_hash(&self, algorithm: calculator::SupportedAlgorithm) -> Result<String, String>;
}

fn compute_hash_from_reader<R: BufRead>(
    reader: R,
    algorithm: calculator::SupportedAlgorithm,
) -> Result<String, String> {
    calculator::hash_calculator(reader, algorithm)
        .map_err(|error| format!("Error: Error calculating hash: {}", error))
}

impl ComputeHash for Data {
    fn compute_hash(&self, algorithm: calculator::SupportedAlgorithm) -> Result<String, String> {
        match self {
            Data::ReadFile(path) if path == "-" => {
                compute_hash_from_reader(stdin().lock(), algorithm)
            }
            Data::ReadFile(path) => {
                let file = File::open(path)
                    .map_err(|error| format!("Error: Cannot open file {}: {}", path, error))?;
                compute_hash_from_reader(BufReader::new(file), algorithm)
            }
            Data::Text(text) => {
                compute_hash_from_reader(BufReader::new(text.as_bytes()), algorithm)
            }
        }
    }
}

fn validate_hash_for_algorithm(
    hash: &str,
    algorithm: calculator::SupportedAlgorithm,
) -> Result<(), String> {
    let detected_algorithms =
        extra::detect_hash_algorithm(hash).map_err(|error| format!("{} {}", error, hash))?;

    if detected_algorithms.contains(&algorithm) {
        Ok(())
    } else {
        Err(format!(
            "Error: Hash does not match algorithm {}.",
            algorithm
        ))
    }
}

fn parse_hash_input<S: AsRef<str>>(
    hash_input: S,
) -> Result<(Option<calculator::SupportedAlgorithm>, String), String> {
    let hash_input = hash_input.as_ref().trim();
    if hash_input.is_empty() {
        return Err(String::from("Error: Invalid hash."));
    }

    if let Some((algorithm_name, hash)) = hash_input.split_once(':') {
        let algorithm_name = algorithm_name.trim();
        let hash = hash.trim();

        if algorithm_name.is_empty() || hash.is_empty() {
            return Err(String::from("Error: Invalid hash."));
        }

        let algorithm = calculator::SupportedAlgorithm::from_input(algorithm_name)?;
        validate_hash_for_algorithm(hash, algorithm)?;
        Ok((Some(algorithm), hash.to_string()))
    } else {
        Ok((None, hash_input.to_string()))
    }
}

pub fn resolve_hash_input<S: AsRef<str>>(
    hash_input: S,
    algorithm: Option<calculator::SupportedAlgorithm>,
) -> Result<ResolvedHashInput, String> {
    let (prefixed_algorithm, hash) = parse_hash_input(hash_input)?;

    let algorithms = match (algorithm, prefixed_algorithm) {
        (Some(specified_algorithm), Some(prefixed_algorithm))
            if specified_algorithm != prefixed_algorithm =>
        {
            return Err(format!(
                "Error: Conflicting algorithms: specified {}, hash prefix specifies {}.",
                specified_algorithm, prefixed_algorithm
            ));
        }
        (Some(specified_algorithm), _) => vec![specified_algorithm],
        (None, Some(prefixed_algorithm)) => vec![prefixed_algorithm],
        (None, None) => {
            extra::detect_hash_algorithm(&hash).map_err(|error| format!("{} {}", error, hash))?
        }
    };

    Ok(ResolvedHashInput {
        hash,
        algorithms,
        detected_from_hash: algorithm.is_none() && prefixed_algorithm.is_none(),
    })
}

pub fn match_algorithm<S: AsRef<str>>(
    algorithm: S,
) -> Result<calculator::SupportedAlgorithm, String> {
    calculator::SupportedAlgorithm::from_input(algorithm)
}

fn resolve_shasum_entry_path(base_dir: &Path, file_path: &str) -> String {
    if file_path == "-" {
        return file_path.to_string();
    }

    let file_path = Path::new(file_path);
    if file_path.is_absolute() || base_dir == Path::new(".") {
        file_path.to_string_lossy().into_owned()
    } else {
        base_dir.join(file_path).to_string_lossy().into_owned()
    }
}

pub fn phase_shasum_file<S: AsRef<str>>(
    shasum_file_path: S,
    algorithm: Option<calculator::SupportedAlgorithm>,
) -> Result<Vec<Compare>, String> {
    /*
    Example shasum file:
        ee1fb7719c31070f1fbdc8f2d32370c9d1ca6962  image.png
        ee1fb7719c31070f1fbdc8f2d32370c9d1ca6962 *image.png
                                                 ^ In binary mode, neglected.
     */
    let shasum_file_path = shasum_file_path.as_ref();
    let file = File::open(shasum_file_path)
        .map_err(|error| format!("Error: Cannot open file {}: {}", shasum_file_path, error))?;
    let reader = BufReader::new(file);
    let base_dir = Path::new(shasum_file_path)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));

    let mut compare_tasks = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|error| {
            format!(
                "Error: Cannot read shasum file {}: {}",
                shasum_file_path, error
            )
        })?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        if parts.len() != 2 {
            return Err("Error: Not a valid shasum file.".to_string());
        }

        let resolved_hash = resolve_hash_input(parts[0], algorithm)?;
        let file_path = parts[1].strip_prefix('*').unwrap_or(parts[1]);
        let file_path = resolve_shasum_entry_path(base_dir, file_path);

        for algorithm in resolved_hash.algorithms {
            compare_tasks.push(Compare::new(
                Data::ReadFile(file_path.clone()),
                resolved_hash.hash.clone(),
                algorithm,
            ));
        }
    }

    Ok(compare_tasks)
}

#[cfg(test)]
mod test_core {
    use super::{match_algorithm, phase_shasum_file, resolve_hash_input, Calculate, Compare, Data};
    use crate::calculator;
    use crate::IfMatch::{Failed, Match};

    #[test]
    fn test_calculate_compute_hash_file() {
        let task = Calculate::new(
            Data::ReadFile(String::from("tests/滕王阁序.txt")),
            calculator::SupportedAlgorithm::SHA256,
        );
        assert_eq!(
            task.compute().unwrap(),
            "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        );
    }

    #[test]
    fn test_calculate_compute_hash_text() {
        let task = Calculate::new(
            Data::Text(String::from("Veni, vidi, vici")),
            calculator::SupportedAlgorithm::SHA256,
        );
        assert_eq!(
            task.compute().unwrap(),
            "b1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466"
        );
    }

    #[test]
    fn test_compare_hash_file() {
        let task = Compare::new(
            Data::ReadFile(String::from("tests/滕王阁序.txt")),
            String::from("00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"),
            calculator::SupportedAlgorithm::SHA256,
        );
        assert_eq!(task.compute().unwrap(), Match("".to_string()))
    }

    #[test]
    fn test_compare_hash_text() {
        let task = Compare::new(
            Data::Text(String::from("Veni, vidi, vici")),
            String::from("a1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466"),
            calculator::SupportedAlgorithm::SHA256,
        );
        assert_eq!(task.compute().unwrap(), Failed(String::from("")))
    }

    #[test]
    fn test_compare_hash_text_is_case_insensitive() {
        let task = Compare::new(
            Data::Text(String::from("Veni, vidi, vici")),
            String::from("B1610284C94BBF9AA78333E57DDCE234A5E845D61E09CE91A7E19FA24737F466"),
            calculator::SupportedAlgorithm::SHA256,
        );
        assert_eq!(task.compute().unwrap(), Match(String::new()))
    }

    #[test]
    fn test_phase_shasum_file_resolves_relative_paths() {
        let tasks = phase_shasum_file(
            "tests/sha256sum.txt",
            Some(calculator::SupportedAlgorithm::SHA256),
        )
        .unwrap();

        assert_eq!(tasks.len(), 2);
        for task in tasks {
            assert_eq!(task.compute().unwrap(), Match(String::new()));
        }
    }

    #[test]
    fn test_phase_shasum_file_supports_prefixed_hashes() {
        let tasks = phase_shasum_file("tests/prefixed-shasum.txt", None).unwrap();

        assert_eq!(tasks.len(), 2);
        for task in tasks {
            assert_eq!(task.compute().unwrap(), Match(String::new()));
        }
    }

    #[test]
    fn test_resolve_hash_input_supports_prefixed_hashes() {
        let resolved = resolve_hash_input(
            "ShA256:00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95",
            None,
        )
        .unwrap();

        assert_eq!(
            resolved.algorithms,
            vec![calculator::SupportedAlgorithm::SHA256]
        );
        assert_eq!(
            resolved.hash,
            "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        );
        assert!(!resolved.detected_from_hash);
    }

    #[test]
    fn test_resolve_hash_input_rejects_conflicting_algorithms() {
        assert!(resolve_hash_input(
            "sha512/256:00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95",
            Some(calculator::SupportedAlgorithm::SHA256),
        )
        .is_err());
    }

    #[test]
    fn test_match_algorithm_supports_xxhash() {
        assert_eq!(
            match_algorithm("xxh64").unwrap(),
            calculator::SupportedAlgorithm::XXHASH64
        );
    }

    #[test]
    fn test_match_algorithm_supports_case_insensitive_aliases() {
        assert_eq!(
            match_algorithm("sHa512/256").unwrap(),
            calculator::SupportedAlgorithm::SHA512_256
        );
    }
}
