#[cfg(all(feature = "hashes_backend", feature = "ring_backend"))]
compile_error!("Feature `hashes_backend` and feature `ring_backend` cannot be enabled at the same time.");
#[cfg(not(any(feature = "hashes_backend", feature = "ring_backend")))]
compile_error!("You must enable at least one of the features: 'hashes_backend' or 'ring_backend'.");

use std::process;
use clap::{Parser, Subcommand};
use ezcheck::{ Calculate, Data, Compare, IfMatch, match_algorithm, phase_shasum_file };
use ezcheck::calculator::SupportedAlgorithm;
use ezcheck::extra;

#[cfg(feature = "hashes_backend")]
#[derive(Parser)]
#[command(name = "ezcheck")]
#[command(version = "0.1.1 (Hashes Backend)")]
#[command(about = "An easy tool to calculate and check hash.\nMade with love by Heqi Liu, https://github.com/metaphorme")]
struct Cli {
    #[command(subcommand)]
    args: Args,
}

#[cfg(feature = "ring_backend")]
#[derive(Parser)]
#[command(name = "ezcheck")]
#[command(version = "0.1.1 (Ring Backend)")]
#[command(about = "An easy tool to calculate and check hash.\nMade with love by Heqi Liu, https://github.com/metaphorme")]
struct Cli {
    #[command(subcommand)]
    args: Args,
}

#[cfg(feature = "hashes_backend")]
#[derive(Subcommand)]
enum Args {
    /// Calculate hash for a file or text.
    Calculate {
        /// Optional algorithm to use for calculate hash.
        /// Supported algorithms:
        ///  * MD2(Unsafe)
        ///  * MD4(Unsafe)
        ///  * MD5(Unsafe)
        ///  * SHA1(Unsafe)
        ///  * SHA224
        ///  * SHA256(default)
        ///  * SHA384
        ///  * SHA512
        ///  * SHA512/256
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// File to calculate hash, specify filename with -f/--file or directly provide the filename. Specify "-" to read from standard input.
        #[arg(short, long)]
        file: Option<String>,

        /// Direct text input for hash calculation.
        #[arg(short, long)]
        text: Option<String>,
    },

    /// Compare with given hash.
    Compare {
        /// Optional algorithm to use for calculate hash.
        /// Leave blank to automatically detect the hash algorithm.
        /// Supported algorithms:
        ///  * MD2(Unsafe)
        ///  * MD4(Unsafe)
        ///  * MD5(Unsafe)
        ///  * SHA1(Unsafe)
        ///  * SHA224
        ///  * SHA256
        ///  * SHA384
        ///  * SHA512
        ///  * SHA512/256
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// File to calculate hash, specify filename with -f/--file or directly provide the filename. Specify "-" to read from standard input.
        #[arg(short, long)]
        file: Option<String>,

        /// Direct text input for hash comparing.
        #[arg(short, long)]
        text: Option<String>,

        /// Hash to compare with.
        #[arg(short, long)]
        check_hash: Option<String>,
    },

    /// Check with given shasum file.
    Check {
        /// Optional algorithm to use for calculate hash.
        /// Leave blank to automatically detect the hash algorithm.
        /// Supported algorithms:
        ///  * MD2(Unsafe)
        ///  * MD4(Unsafe)
        ///  * MD5(Unsafe)
        ///  * SHA1(Unsafe)
        ///  * SHA224
        ///  * SHA256
        ///  * SHA384
        ///  * SHA512
        ///  * SHA512/256
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// shasum file to check with.
        #[arg(short, long)]
        check_file: Option<String>,
    }
}

#[cfg(feature = "ring_backend")]
#[derive(Subcommand)]
enum Args {
    /// Calculate hash for a file or text.
    Calculate {
        /// Optional algorithm to use for calculate hash
        /// Supported algorithms:
        ///  * SHA256(default)
        ///  * SHA384
        ///  * SHA512
        ///  * SHA512/256
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// File to calculate hash, specify filename with -f/--file or directly provide the filename. Specify "-" to read from standard input.
        #[arg(short, long)]
        file: Option<String>,

        /// Direct text input for hash calculation.
        #[arg(short, long)]
        text: Option<String>,
    },

    /// Compare with given hash.
    Compare {
        /// Optional algorithm to use for calculate hash.
        /// Leave blank to automatically detect the hash algorithm.
        /// Supported algorithms:
        ///  * SHA256
        ///  * SHA384
        ///  * SHA512
        ///  * SHA512/256
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// File to calculate hash, specify filename with -f/--file or directly provide the filename. Specify "-" to read from standard input.
        #[arg(short, long)]
        file: Option<String>,

        /// Direct text input for hash comparing.
        #[arg(short, long)]
        text: Option<String>,

        /// Hash to compare with.
        #[arg(short, long)]
        check_hash: Option<String>,
    },

    /// Check with given shasum file.
    Check {
        /// Optional algorithm to use for calculate hash.
        /// Leave blank to automatically detect the hash algorithm.
        /// Supported algorithms:
        ///  * SHA256
        ///  * SHA384
        ///  * SHA512
        ///  * SHA512/256
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// shasum file to check with.
        #[arg(short, long)]
        check_file: Option<String>,
    }
}

fn detect_algorithm(input: Option<String>) -> Option<SupportedAlgorithm> {
    match input {
        Some(ref b) => Option::from(match_algorithm(b).unwrap_or_else(|e| {
            eprintln!("{}", e);
            process::exit(1);
        })),
        None => None,  // User doesn't input algorithm
    }
}

fn main() {
    let args = Cli::parse();

    match args.args {
        Args::Calculate { algorithm, file, text } => {
            // --file option and -- text option are mutually exclusive
            if file.is_some() && text.is_some() {
                eprintln!("Error: Both file and text options cannot be used together.");
                process::exit(1);
            }

            if file.is_none() && text.is_none() {
                eprintln!("Error: At least one of file or text options must be provided.\nRun `ezcheck calculate --help` for more information.");
                process::exit(1);
            }

            let algorithm = detect_algorithm(algorithm);

            let algorithm = match algorithm {
                Some(a) => a,
                _ => {
                    println!("No algorithm specified. Using SHA256 as the default.");
                    SupportedAlgorithm::SHA256
                }
            };

            if file.is_some() {  // File mode
                let path = file.unwrap();
                let task = Calculate::new(
                    Data::ReadFile(String::from(&path)),
                    algorithm,
                );
                let result = task.compute();
                match result {
                    Ok(result) => println!("{}  {}", result, &path),
                    Err(e) => eprintln!("{}", e),
                }
            } else {  // Text mode
                let text = text.unwrap();
                let task = Calculate::new(
                    Data::Text(text),
                    algorithm,
                );
                let result = task.compute();
                match result {
                    Ok(result) => println!("{}:  {}", algorithm, result),
                    Err(e) => eprintln!("{}", e),
                }
            }
        },

        Args::Compare { algorithm, file, text, check_hash} => {
            // --file option and -- text option are mutually exclusive
            if file.is_some() && text.is_some() {
                eprintln!("Error: Both file and text options cannot be used together.");
                process::exit(1);
            }

            if file.is_none() && text.is_none() {
                eprintln!("Error: At least one of file or text options must be provided.\nRun `ezcheck compare --help` for more information.");
                process::exit(1);
            }

            let hash = match check_hash {
                Some(h) => h,
                _ => {
                    eprintln!("Error: Must provide hash.");
                    process::exit(1);
                }
            };

            let algorithm = match detect_algorithm(algorithm) {
                Some(a) => {  // User inputs valid algorithm
                    vec!(a)
                },
                _ => match extra::detect_hash_algorithm(&hash) {  // User doesn't input algorithm
                    Ok(a) => {
                        if a.len() == 1 {
                            println!("INFO: Detect Hash Algorithm: {}", a[0]);

                        } else {
                            let algorithm_names: Vec<String> = a.iter()
                                .map(|alg| alg.to_string())
                                .collect();

                            println!("INFO: Hash Algorithm could be {}", algorithm_names.join(", "));
                        }
                        a
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                }
            };

            for alg in &algorithm {
                let task = if let Some(ref file_path) = file {
                    Compare::new(
                        Data::ReadFile(file_path.clone()),
                        hash.clone(),
                        *alg,
                    )
                } else if let Some(ref text) = text {
                    Compare::new(
                        Data::Text(text.clone()),
                        hash.clone(),
                        *alg,
                    )
                } else {  // Cannot be here!
                    Compare::new(
                        Data::Text("".to_string()),
                        "".to_string(),
                        *alg,
                    )
                };

                let result = task.compute();
                match result {
                    Ok(IfMatch::Match(message)) => {
                        println!("{}", message);
                        break;
                    },
                    Ok(IfMatch::Failed(message)) => {
                        println!("{}", message);
                    },
                    Err(e) => eprintln!("{}", e),
                }
            }
        },

        Args::Check { algorithm, check_file, } => {
            match check_file {
                Some(f) => {
                    match phase_shasum_file (
                        f,
                        detect_algorithm(algorithm),
                    ) {
                        Ok(tasks) => {
                            let mut file_name = String::new();
                            let mut file_matched = false;
                            for task in tasks {
                                if file_name == task.data.to_string() && file_matched == true {
                                    continue;
                                } else {
                                    let result = task.compute();
                                    match result {
                                        Ok(IfMatch::Match(message))=> {
                                            file_name = task.data.to_string();
                                            file_matched = true;
                                            println!("{}: {}", task.data, message);
                                        },
                                        Ok(IfMatch::Failed(message)) => {
                                            file_name = task.data.to_string();
                                            file_matched = false;
                                            println!("{}: {}", task.data, message);
                                        }
                                        Err(e) => eprintln!("{}: {}", task.data, e),
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("{}", e);
                            process::exit(1);
                        }
                    };
                }
                None => eprintln!("Must provide a check file.\nRun `ezcheck check --help` for more information."),
            }
        }
    }
}