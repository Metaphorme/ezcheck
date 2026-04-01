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

use clap::{Parser, Subcommand};
use ezcheck::calculator::SupportedAlgorithm;
use ezcheck::{
    match_algorithm, phase_shasum_file, resolve_hash_input, Calculate, Compare, Data, IfMatch,
};
use std::process;

const CLI_ABOUT: &str =
    "An easy tool to calculate and check hash.\nMade with love by Heqi Liu, https://github.com/metaphorme";

#[cfg(feature = "hashes_backend")]
const CLI_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (Hashes Backend)");
#[cfg(feature = "ring_backend")]
const CLI_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (Ring Backend)");
#[cfg(feature = "mix_backend")]
const CLI_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (Mix Backend)");

#[cfg(feature = "mix_backend")]
const CALCULATE_HELP_TEMPLATE: &str = "Calculate hash for a file or text (alias: c)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Supported algorithms:
      * MD2(Unsafe)
      * MD4(Unsafe)
      * MD5(Unsafe)
      * SHA1(Unsafe)
      * SHA224
      * SHA256(default)
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";
#[cfg(feature = "hashes_backend")]
const CALCULATE_HELP_TEMPLATE: &str = "Calculate hash for a file or text (alias: c)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Supported algorithms:
      * MD2(Unsafe)
      * MD4(Unsafe)
      * MD5(Unsafe)
      * SHA1(Unsafe)
      * SHA224
      * SHA256(default)
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";
#[cfg(feature = "ring_backend")]
const CALCULATE_HELP_TEMPLATE: &str = "Calculate hash for a file or text (alias: c)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Supported algorithms:
      * SHA256(default)
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";

#[cfg(feature = "mix_backend")]
const COMPARE_HELP_TEMPLATE: &str = "Compare with given hash (alias: m)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Leave blank to automatically detect the hash algorithm.
    The value passed to -c/--check-hash may also use algorithm:hash.
    Supported algorithms:
      * MD2(Unsafe)
      * MD4(Unsafe)
      * MD5(Unsafe)
      * SHA1(Unsafe)
      * SHA224
      * SHA256
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";
#[cfg(feature = "hashes_backend")]
const COMPARE_HELP_TEMPLATE: &str = "Compare with given hash (alias: m)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Leave blank to automatically detect the hash algorithm.
    The value passed to -c/--check-hash may also use algorithm:hash.
    Supported algorithms:
      * MD2(Unsafe)
      * MD4(Unsafe)
      * MD5(Unsafe)
      * SHA1(Unsafe)
      * SHA224
      * SHA256
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";
#[cfg(feature = "ring_backend")]
const COMPARE_HELP_TEMPLATE: &str = "Compare with given hash (alias: m)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Leave blank to automatically detect the hash algorithm.
    The value passed to -c/--check-hash may also use algorithm:hash.
    Supported algorithms:
      * SHA256
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";

#[cfg(feature = "mix_backend")]
const CHECK_HELP_TEMPLATE: &str = "Check with given shasum file (alias: k)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Leave blank to automatically detect the hash algorithm.
    The hash column in the check file may also use algorithm:hash.
    Supported algorithms:
      * MD2(Unsafe)
      * MD4(Unsafe)
      * MD5(Unsafe)
      * SHA1(Unsafe)
      * SHA224
      * SHA256
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";
#[cfg(feature = "hashes_backend")]
const CHECK_HELP_TEMPLATE: &str = "Check with given shasum file (alias: k)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Leave blank to automatically detect the hash algorithm.
    The hash column in the check file may also use algorithm:hash.
    Supported algorithms:
      * MD2(Unsafe)
      * MD4(Unsafe)
      * MD5(Unsafe)
      * SHA1(Unsafe)
      * SHA224
      * SHA256
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";
#[cfg(feature = "ring_backend")]
const CHECK_HELP_TEMPLATE: &str = "Check with given shasum file (alias: k)

Usage: {usage}

Arguments:
  [ALGORITHM]
    Optional algorithm to use for calculate hash.
    Leave blank to automatically detect the hash algorithm.
    The hash column in the check file may also use algorithm:hash.
    Supported algorithms:
      * SHA256
      * SHA384
      * SHA512
      * SHA512_256
      * XXHASH32
      * XXHASH64
      * XXHASH3_64

Options:
{options}";

#[derive(Parser)]
#[command(name = "ezcheck")]
#[command(version = CLI_VERSION)]
#[command(about = CLI_ABOUT)]
struct Cli {
    #[command(subcommand)]
    args: Args,
}

#[derive(Subcommand)]
enum Args {
    /// Calculate hash for a file or text (alias: c)
    #[command(alias = "c", help_template = CALCULATE_HELP_TEMPLATE)]
    Calculate {
        algorithm: Option<String>,

        /// File(s) to calculate hash, specify filename with -f/--file or directly provide the filename. Specify "-" to read from standard input.
        #[arg(short, long, num_args = 1..)]
        file: Option<Vec<String>>,

        /// Direct text input for hash calculation.
        #[arg(short, long)]
        text: Option<String>,
    },

    /// Compare with given hash (alias: m)
    #[command(alias = "m", help_template = COMPARE_HELP_TEMPLATE)]
    Compare {
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

    /// Check with given shasum file (alias: k)
    #[command(alias = "k", help_template = CHECK_HELP_TEMPLATE)]
    Check {
        algorithm: Option<String>,

        /// shasum file to check with.
        #[arg(short, long)]
        check_file: Option<String>,
    },
}

fn detect_algorithm(input: Option<String>) -> Option<SupportedAlgorithm> {
    input.map(|value| {
        match_algorithm(&value).unwrap_or_else(|error| {
            eprintln!("{}", error);
            process::exit(1);
        })
    })
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1);
}

fn validate_input_source(file_present: bool, text_present: bool, help_command: &str) {
    if file_present && text_present {
        exit_with_error("Error: Both file and text options cannot be used together.");
    }

    if !file_present && !text_present {
        exit_with_error(&format!(
            "Error: At least one of file or text options must be provided.\nRun `{}` for more information.",
            help_command
        ));
    }
}

fn calculate(algorithm: Option<String>, file: Option<Vec<String>>, text: Option<String>) {
    validate_input_source(file.is_some(), text.is_some(), "ezcheck calculate --help");

    let algorithm = match detect_algorithm(algorithm) {
        Some(algorithm) => algorithm,
        None => {
            println!("No algorithm specified. Using SHA256 as the default.");
            SupportedAlgorithm::SHA256
        }
    };

    if let Some(files) = file {
        for file_path in files {
            let task = Calculate::new(Data::ReadFile(file_path.clone()), algorithm);
            match task.compute() {
                Ok(result) => println!("{}  {}", result, file_path),
                Err(error) => eprintln!("{}", error),
            }
        }
    } else if let Some(text) = text {
        let task = Calculate::new(Data::Text(text), algorithm);
        match task.compute() {
            Ok(result) => println!("{}", result),
            Err(error) => eprintln!("{}", error),
        }
    } else {
        unreachable!("input validation guarantees that either file or text is present");
    }
}

fn compare(
    algorithm: Option<String>,
    file: Option<String>,
    text: Option<String>,
    check_hash: Option<String>,
) {
    validate_input_source(file.is_some(), text.is_some(), "ezcheck compare --help");

    let hash = match check_hash {
        Some(hash) => hash,
        None => exit_with_error("Error: Must provide hash."),
    };

    let resolved_hash = match resolve_hash_input(hash, detect_algorithm(algorithm)) {
        Ok(resolved_hash) => resolved_hash,
        Err(error) => exit_with_error(&error),
    };

    if resolved_hash.detected_from_hash {
        if resolved_hash.algorithms.len() == 1 {
            println!(
                "INFO: Detect Hash Algorithm: {}",
                resolved_hash.algorithms[0]
            );
        } else {
            let algorithm_names: Vec<String> = resolved_hash
                .algorithms
                .iter()
                .map(|algorithm| algorithm.to_string())
                .collect();
            println!(
                "INFO: Hash Algorithm could be {}",
                algorithm_names.join(", ")
            );
        }
    }

    let mut matched = false;

    for algorithm in resolved_hash.algorithms {
        let task = match (&file, &text) {
            (Some(file_path), None) => Compare::new(
                Data::ReadFile(file_path.clone()),
                resolved_hash.hash.clone(),
                algorithm,
            ),
            (None, Some(text)) => Compare::new(
                Data::Text(text.clone()),
                resolved_hash.hash.clone(),
                algorithm,
            ),
            _ => unreachable!("input validation guarantees exactly one input source"),
        };

        match task.compute() {
            Ok(IfMatch::Match(message)) => {
                println!("{}", message);
                matched = true;
                break;
            }
            Ok(IfMatch::Failed(message)) => {
                println!("{}", message);
            }
            Err(error) => eprintln!("{}", error),
        }
    }

    if !matched {
        process::exit(1);
    }
}

fn check(algorithm: Option<String>, check_file: Option<String>) {
    let check_file = match check_file {
        Some(check_file) => check_file,
        None => exit_with_error(
            "Must provide a check file.\nRun `ezcheck check --help` for more information.",
        ),
    };

    match phase_shasum_file(check_file, detect_algorithm(algorithm)) {
        Ok(tasks) => {
            let mut current_task = None;
            let mut current_task_matched = false;
            let mut has_unmatched_task = false;

            for task in tasks {
                let task_key = (task.data.to_string(), task.expected_hash().to_string());

                if current_task.as_ref() != Some(&task_key) {
                    if current_task.is_some() && !current_task_matched {
                        has_unmatched_task = true;
                    }

                    current_task = Some(task_key.clone());
                    current_task_matched = false;
                }

                if current_task_matched {
                    continue;
                }

                match task.compute() {
                    Ok(IfMatch::Match(message)) => {
                        current_task_matched = true;
                        println!("{}: {}", task.data, message);
                    }
                    Ok(IfMatch::Failed(message)) => {
                        println!("{}: {}", task.data, message);
                    }
                    Err(error) => eprintln!("{}: {}", task.data, error),
                }
            }

            if current_task.is_some() && !current_task_matched {
                has_unmatched_task = true;
            }

            if has_unmatched_task {
                process::exit(1);
            }
        }
        Err(error) => exit_with_error(&error),
    }
}

fn main() {
    let args = Cli::parse();

    match args.args {
        Args::Calculate {
            algorithm,
            file,
            text,
        } => {
            calculate(algorithm, file, text);
        }

        Args::Compare {
            algorithm,
            file,
            text,
            check_hash,
        } => {
            compare(algorithm, file, text, check_hash);
        }

        Args::Check {
            algorithm,
            check_file,
        } => {
            check(algorithm, check_file);
        }
    }
}
