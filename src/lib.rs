#[cfg(all(feature = "hashes_backend", feature = "ring_backend"))]
compile_error!("Feature `hashes_backend` and feature `ring_backend` cannot be enabled at the same time.");
#[cfg(not(any(feature = "hashes_backend", feature = "ring_backend")))]
compile_error!("You must enable at least one of the features: 'hashes_backend' or 'ring_backend'.");

pub mod calculator;
pub mod extra;
use std::fmt;
use std::fs::File;
use std::io::{stdin, Read, BufRead, BufReader};

pub struct Calculate {
    data: Data,
    algorithm: calculator::SupportedAlgorithm,
}

impl Calculate {
    pub fn new(data: Data, algorithm: calculator::SupportedAlgorithm) -> Calculate {
        Self {
            data,
            algorithm,
        }
    }
}

impl Calculate {
    pub fn compute(&self) -> Result<String, String> {
        self.data.compute_hash(self.algorithm)
    }
}

pub struct Compare {
    pub data: Data,
    compare: String,
    algorithm: calculator::SupportedAlgorithm,
}

impl Compare {
    pub fn new(data: Data, compare: String, algorithm: calculator::SupportedAlgorithm) -> Compare {
        Self {
            data,
            compare,
            algorithm,
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
        match (self, other) {
            (IfMatch::Match(_), IfMatch::Match(_)) => true,
            (IfMatch::Failed(_), IfMatch::Failed(_)) => true,
            _ => false,
        }
    }
}

impl Compare {
    pub fn compute(&self) -> Result<IfMatch, String> {
        let hash_result = match self.data.compute_hash(self.algorithm) {
            Ok(hash_result) => hash_result,
            Err(error) => return Err(error),
        };

        if hash_result == self.compare {
            Ok(IfMatch::Match(format!("{:8} OK", self.algorithm)))
        } else {
            Ok(IfMatch::Failed(format!("{:8} FAILED  Current Hash:  {}", self.algorithm, hash_result)))
        }
    }
}

pub enum Data {
    ReadFile(String),  // Input data from a file
    Text(String),     // Input data from user input
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let algorithm = match self {
            Data::ReadFile(file_name) => file_name,
            Data::Text(text) => text,
        };
        write!(f, "{}", algorithm)
    }
}

pub trait ComputeHash {
    fn compute_hash(&self, algorithm: calculator::SupportedAlgorithm) -> Result<String, String>;
}

impl ComputeHash for Data {
    fn compute_hash(&self, algorithm: calculator::SupportedAlgorithm) -> Result<String, String> {
        match self {
            Data::ReadFile(path) => {
                let reader: BufReader<Box<dyn Read>>;
                if path == "-" {
                    // Read from standard input
                    reader = BufReader::new(Box::new(stdin()) as Box<dyn Read>);
                } else {
                    // Try to open the file
                    let file = File::open(path).map_err(|e| format!("Error: Error opening file: {}", e))?;
                    reader = BufReader::new(Box::new(file) as Box<dyn Read>);
                }
                match calculator::hash_calculator(reader, algorithm) {
                    Ok(hash) => Ok(hash),
                    Err(e) => Err(format!("Error: Error calculating hash: {}", e)),
                }
            }
            Data::Text(text) => {
                let reader = BufReader::new(text.as_bytes());
                match calculator::hash_calculator(reader, algorithm) {
                    Ok(hash) => Ok(hash),
                    Err(e) => Err(format!("Error: Error calculating hash: {}", e)),
                }
            }
        }
    }
}

#[cfg(feature = "hashes_backend")]
pub fn match_algorithm<S: AsRef<str>>(algorithm: S) -> Result<calculator::SupportedAlgorithm, String> {
    let algorithm = algorithm.as_ref();

    match algorithm {
        "MD2" | "Md2" | "mD2" | "md2" => Ok(calculator::SupportedAlgorithm::MD2),
        "MD4" | "Md4" | "mD4" | "md4" => Ok(calculator::SupportedAlgorithm::MD4),
        "MD5" | "Md5" | "mD5" | "md5" => Ok(calculator::SupportedAlgorithm::MD5),
        "sha1" | "Sha1" | "sHa1" | "shA1" | "SHa1" | "sHA1" | "ShA1" | "SHA1" => Ok(calculator::SupportedAlgorithm::SHA1),
        "sha224" | "Sha224" | "sHa224" | "shA224" | "SHa224" | "sHA224" | "ShA224" | "SHA224" => Ok(calculator::SupportedAlgorithm::SHA224),
        "sha256" | "Sha256" | "sHa256" | "shA256" | "SHa256" | "sHA256" | "ShA256" | "SHA256" => Ok(calculator::SupportedAlgorithm::SHA256),
        "sha384" | "Sha384" | "sHa384" | "shA384" | "SHa384" | "sHA384" | "ShA384" | "SHA384" => Ok(calculator::SupportedAlgorithm::SHA384),
        "sha512" | "Sha512" | "sHa512" | "shA512" | "SHa512" | "sHA512" | "ShA512" | "SHA512" => Ok(calculator::SupportedAlgorithm::SHA512),
        "sha512_256" | "Sha512_256" | "sHa512_256" | "shA512_256" | "SHa512_256" | "sHA512_256" | "ShA512_256" | "SHA512_256" | "sha512-256" | "Sha512-256" | "sHa512-256" | "shA512-256" | "SHa512-256" | "sHA512-256" | "ShA512-256" | "SHA512-256" => Ok(calculator::SupportedAlgorithm::SHA512_256),
        _ => Err(format!("Error: Unsupported algorithm: {}", algorithm)),
    }
}

#[cfg(feature = "ring_backend")]
pub fn match_algorithm<S: AsRef<str>>(algorithm: S) -> Result<calculator::SupportedAlgorithm, String> {
    let algorithm = algorithm.as_ref();

    match algorithm {
        "sha256" | "Sha256" | "sHa256" | "shA256" | "SHa256" | "sHA256" | "ShA256" | "SHA256" => Ok(calculator::SupportedAlgorithm::SHA256),
        "sha384" | "Sha384" | "sHa384" | "shA384" | "SHa384" | "sHA384" | "ShA384" | "SHA384" => Ok(calculator::SupportedAlgorithm::SHA384),
        "sha512" | "Sha512" | "sHa512" | "shA512" | "SHa512" | "sHA512" | "ShA512" | "SHA512" => Ok(calculator::SupportedAlgorithm::SHA512),
        "sha512_256" | "Sha512_256" | "sHa512_256" | "shA512_256" | "SHa512_256" | "sHA512_256" | "ShA512_256" | "SHA512_256" | "sha512-256" | "Sha512-256" | "sHa512-256" | "shA512-256" | "SHa512-256" | "sHA512-256" | "ShA512-256" | "SHA512-256" => Ok(calculator::SupportedAlgorithm::SHA512_256),
        _ => Err(format!("Error: Unsupported algorithm: {}", algorithm)),
    }
}

pub fn phase_shasum_file<S: AsRef<str>>
(shasum_file_path: S, algorithm: Option<calculator::SupportedAlgorithm>)
    -> Result<Vec<Compare>, String> {
    /*
    Example shasum file:
        ee1fb7719c31070f1fbdc8f2d32370c9d1ca6962  image.png
        ee1fb7719c31070f1fbdc8f2d32370c9d1ca6962 *image.png
                                                 ^ In binary mode, neglected.
     */
    let shasum_file_path = shasum_file_path.as_ref();
    let mut detect_algorithm = true;
    if algorithm.is_some() {
        detect_algorithm = false;
    }

    let file = match File::open(shasum_file_path) {
        Ok(file) => file,
        Err(error) => return Err(format!("Error: Error opening file: {}", error)),
    };
    let reader = BufReader::new(file);

    let mut compare_tasks = vec![];

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();  // Split

        if parts.len() == 2 {
            let hash = parts[0];
            let mut file_path = parts[1].to_string();

            let algorithms = match detect_algorithm {
                true => {
                    match extra::detect_hash_algorithm(hash) {
                        Ok(result) => result,
                        Err(e) => return Err(format!("{} {}", e, hash)),
                    }
                }
                false => vec![algorithm.unwrap()]
            };

            if file_path.starts_with("*") {  // Neglect * starts with filename
                file_path.remove(0);
            }

            for algorithm in algorithms {
                compare_tasks.push(Compare::new(
                    Data::ReadFile(file_path.clone()),
                    hash.to_string(),
                    algorithm,
                ));
            }

        } else if parts.is_empty() {  // Blank line
            continue;
        } else {
            return Err("Error: Not a valid shasum file.".to_string());
        }
    }
    Ok(compare_tasks)
}

#[cfg(test)]
mod test_core {
    use super::{Calculate, Compare, Data};
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
        assert_eq!(
            task.compute().unwrap(),
            Match("".to_string())
        )
    }

    #[test]
    fn test_compare_hash_text() {
        let task = Compare::new(
            Data::Text(String::from("Veni, vidi, vici")),
            String::from("a1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466"),
            calculator::SupportedAlgorithm::SHA256,
        );
        assert_eq!(
            task.compute().unwrap(),
            Failed(String::from(""))
        )
    }

    // This test is only available in tests dir.
    // use crate::core::phase_shasum_file;
    // #[test]
    // fn test_phase_shasum_file() {
    //     let mut tasks = phase_shasum_file("tests/sha256sum.txt", Option::from(calculator::SupportedAlgorithm::SHA256)).unwrap();
    //     for task in tasks {
    //         assert_eq!(
    //             task.compute().unwrap(),
    //             "SHA256 OK"
    //         )
    //     }
    // }
}