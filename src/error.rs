use crate::SupportedAlgorithm;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// ezcheck 可由调用方可靠区分的错误类型。
#[derive(Debug)]
pub enum EzcheckError {
    UnsupportedAlgorithm(String),
    InvalidHash(String),
    HashAlgorithmMismatch(SupportedAlgorithm),
    ConflictingAlgorithms {
        specified: SupportedAlgorithm,
        prefixed: SupportedAlgorithm,
    },
    OpenFile {
        path: PathBuf,
        source: io::Error,
    },
    ReadShasumFile {
        path: PathBuf,
        source: io::Error,
    },
    InvalidShasumLine {
        line: usize,
    },
    CalculateHash(io::Error),
}

impl fmt::Display for EzcheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedAlgorithm(algorithm) => {
                write!(formatter, "Error: Unsupported algorithm: {algorithm}")
            }
            Self::InvalidHash(hash) if hash.is_empty() => {
                formatter.write_str("Error: Invalid hash.")
            }
            Self::InvalidHash(hash) => write!(formatter, "Error: Invalid hash: {hash}."),
            Self::HashAlgorithmMismatch(algorithm) => {
                write!(formatter, "Error: Hash does not match algorithm {algorithm}.")
            }
            Self::ConflictingAlgorithms {
                specified,
                prefixed,
            } => write!(
                formatter,
                "Error: Conflicting algorithms: specified {specified}, hash prefix specifies {prefixed}."
            ),
            Self::OpenFile { path, source } => {
                write!(formatter, "Error: Cannot open file {}: {source}", path.display())
            }
            Self::ReadShasumFile { path, source } => write!(
                formatter,
                "Error: Cannot read shasum file {}: {source}",
                path.display()
            ),
            Self::InvalidShasumLine { line } => {
                write!(formatter, "Error: Not a valid shasum file at line {line}.")
            }
            Self::CalculateHash(source) => {
                write!(formatter, "Error: Error calculating hash: {source}")
            }
        }
    }
}

impl Error for EzcheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::OpenFile { source, .. }
            | Self::ReadShasumFile { source, .. }
            | Self::CalculateHash(source) => Some(source),
            _ => None,
        }
    }
}
