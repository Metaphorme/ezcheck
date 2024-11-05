use std::process;
use clap::{Parser, Subcommand};
use ezcheck::core::{ Calculate, Data, Compare, Compute, match_algorithm, phase_shasum_file };
use ezcheck::calculator::calculator::SupportedAlgorithm;
use ezcheck::extra::extra;

#[derive(Parser)]
#[command(name = "ezcheck")]
#[command(version = "0.1.0")]
#[command(about = "An easy tool to calculate and check hash.\nMade with love by Heqi Liu, https://github.com/metaphorme")]
struct Cli {
    #[command(subcommand)]
    args: Args,
}

#[derive(Subcommand)]
enum Args {
    /// Calculate hash for a file or text.
    Calculate {
        /// Optional algorithm to use for calculate hash
        /// Supported algorithms:
        ///  * MD2(Unsafe)
        ///  * MD4(Unsafe)
        ///  * MD5(Unsafe)
        ///  * SHA1(Unsafe)
        ///  * SHA224
        ///  * SHA256(default)
        ///  * SHA384
        ///  * SHA512
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// File to calculate hash, specify filename with -f/--file or directly provide the filename.
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
        #[arg(verbatim_doc_comment)]
        algorithm: Option<String>,

        /// File to calculate hash, specify filename with -f/--file or directly provide the filename.
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
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("{}", e),
                }
            }
        },

        Args::Check { algorithm, check_file, } => {
            match check_file {
                Some(f) => {
                    let tasks =  match phase_shasum_file(
                        f,
                        detect_algorithm(algorithm),
                    ) {
                        Ok(tasks) => tasks,
                        Err(e) => {
                            eprintln!("{}", e);
                            process::exit(1);
                        }
                    };
                    for task in tasks {
                        let result = task.compute();
                        match result {
                            Ok(result) => println!("{}: {}", task.data ,result),
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                }
                None => eprintln!("Must provide a check file.\nRun `ezcheck check --help` for more information."),
            }
        }
    }
}