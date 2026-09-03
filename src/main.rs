use clap::{Args as ClapArgs, Parser, Subcommand};
use ezcheck::{
    parse_shasum_file, resolve_hash_input, Data, SupportedAlgorithm, Verification,
    VerificationOutcome,
};
use std::io::{self, IsTerminal, Write};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CLI_ABOUT: &str =
    "An easy tool to calculate and check hash.\nMade with love by Heqi Liu, https://github.com/metaphorme";

const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_RESET: &str = "\x1b[0m";

fn colorize(message: String, color: &str, enabled: bool) -> String {
    if enabled {
        format!("{color}{message}{ANSI_RESET}")
    } else {
        message
    }
}

fn render_verification(outcome: &VerificationOutcome, color: bool) -> String {
    match outcome {
        VerificationOutcome::Match { algorithm } => {
            colorize(format!("{} OK", algorithm), ANSI_GREEN, color)
        }
        VerificationOutcome::Failed {
            algorithm,
            current_hash,
        } => format!(
            "{}  Current Hash:{}",
            colorize(format!("{} FAILED", algorithm), ANSI_RED, color),
            current_hash
        ),
    }
}

fn write_checksum_line<W: Write>(writer: &mut W, hash: &str, file_path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    let path_bytes = file_path.as_os_str().as_bytes();
    #[cfg(not(unix))]
    let path_text = file_path.to_string_lossy();
    #[cfg(not(unix))]
    let path_bytes = path_text.as_bytes();

    let escaped = path_bytes
        .iter()
        .any(|byte| matches!(byte, b'\\' | b'\n' | b'\r'));
    let mut line = Vec::with_capacity(hash.len() + path_bytes.len() + 4);

    if escaped {
        line.push(b'\\');
    }
    line.extend_from_slice(hash.as_bytes());
    line.extend_from_slice(b"  ");

    for byte in path_bytes {
        match byte {
            b'\\' => line.extend_from_slice(b"\\\\"),
            b'\n' => line.extend_from_slice(b"\\n"),
            b'\r' => line.extend_from_slice(b"\\r"),
            _ => line.push(*byte),
        }
    }
    line.push(b'\n');

    writer.write_all(&line)
}

#[cfg(all(feature = "hashes_backend", not(feature = "ring_backend")))]
const CLI_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (Hashes Backend)");
#[cfg(all(feature = "ring_backend", not(feature = "hashes_backend")))]
const CLI_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (Ring Backend)");
#[cfg(all(feature = "hashes_backend", feature = "ring_backend"))]
const CLI_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (Mix Backend)");

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
    #[command(alias = "c")]
    Calculate {
        #[arg(
            value_enum,
            ignore_case = true,
            help = "Algorithm to use; defaults to SHA256"
        )]
        algorithm: Option<SupportedAlgorithm>,

        #[command(flatten)]
        input: CalculateInput,
    },

    /// Compare with given hash (alias: m)
    #[command(alias = "m")]
    Compare {
        #[arg(
            value_enum,
            ignore_case = true,
            help = "Algorithm to use; omit to infer it from the expected hash"
        )]
        algorithm: Option<SupportedAlgorithm>,

        #[command(flatten)]
        input: CompareInput,

        #[arg(
            short,
            long,
            help = "Expected hash; accepts either hash or algorithm:hash"
        )]
        check_hash: String,
    },

    /// Check with given shasum file (alias: k)
    #[command(alias = "k")]
    Check {
        #[arg(
            value_enum,
            ignore_case = true,
            help = "Algorithm to use; omit to resolve each check-file record independently"
        )]
        algorithm: Option<SupportedAlgorithm>,

        #[arg(short, long, help = "GNU shasum-compatible file to verify")]
        check_file: PathBuf,
    },
}

#[derive(ClapArgs)]
#[group(required = true, multiple = false)]
struct CalculateInput {
    /// File(s) to calculate hash. Specify "-" to read from standard input.
    #[arg(short, long, num_args = 1..)]
    file: Option<Vec<PathBuf>>,

    /// Direct text input for hash calculation.
    #[arg(short, long)]
    text: Option<String>,
}

#[derive(ClapArgs)]
#[group(required = true, multiple = false)]
struct CompareInput {
    /// File to calculate hash. Specify "-" to read from standard input.
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Direct text input for hash comparing.
    #[arg(short, long)]
    text: Option<String>,
}

#[derive(Clone, Copy)]
enum CommandStatus {
    Success,
    Failure,
}

impl CommandStatus {
    fn exit_code(self) -> ExitCode {
        match self {
            Self::Success => ExitCode::SUCCESS,
            Self::Failure => ExitCode::FAILURE,
        }
    }
}

fn report_error(error: impl std::fmt::Display) -> CommandStatus {
    eprintln!("{error}");
    CommandStatus::Failure
}

fn calculate(algorithm: Option<SupportedAlgorithm>, input: CalculateInput) -> CommandStatus {
    let algorithm = match algorithm {
        Some(algorithm) => algorithm,
        None => {
            println!("No algorithm specified. Using SHA256 as the default.");
            SupportedAlgorithm::SHA256
        }
    };

    let mut succeeded = true;

    match input {
        CalculateInput {
            file: Some(files),
            text: None,
        } => {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();

            for file_path in files {
                let data = Data::from_path(file_path.clone());
                match data.calculate(algorithm) {
                    Ok(result) => {
                        if let Err(error) = write_checksum_line(&mut stdout, &result, &file_path) {
                            eprintln!("Error: Cannot write output: {error}");
                            succeeded = false;
                            break;
                        }
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        succeeded = false;
                    }
                }
            }
        }
        CalculateInput {
            file: None,
            text: Some(text),
        } => {
            let data = Data::Text(text);
            match data.calculate(algorithm) {
                Ok(result) => println!("{result}"),
                Err(error) => {
                    eprintln!("{error}");
                    succeeded = false;
                }
            }
        }
        _ => unreachable!("Clap guarantees exactly one input source"),
    }

    if succeeded {
        CommandStatus::Success
    } else {
        CommandStatus::Failure
    }
}

fn compare(
    algorithm: Option<SupportedAlgorithm>,
    input: CompareInput,
    check_hash: String,
) -> CommandStatus {
    let resolved_hash = match resolve_hash_input(check_hash, algorithm) {
        Ok(resolved_hash) => resolved_hash,
        Err(error) => return report_error(error),
    };

    if resolved_hash.detected_from_hash() {
        if resolved_hash.algorithms().len() == 1 {
            println!(
                "INFO: Detect Hash Algorithm: {}",
                resolved_hash.algorithms()[0]
            );
        } else {
            let algorithm_names: Vec<String> = resolved_hash
                .algorithms()
                .iter()
                .map(|algorithm| algorithm.to_string())
                .collect();
            println!(
                "INFO: Hash Algorithm could be {}",
                algorithm_names.join(", ")
            );
        }
    }

    let data = match input {
        CompareInput {
            file: Some(file_path),
            text: None,
        } => Data::from_path(file_path),
        CompareInput {
            file: None,
            text: Some(text),
        } => Data::Text(text),
        _ => unreachable!("Clap guarantees exactly one input source"),
    };
    let task = Verification::new(data, resolved_hash);
    let mut matched = false;
    let color = io::stdout().is_terminal();

    match task.compute() {
        Ok(results) => {
            for result in results {
                println!("{}", render_verification(&result, color));
                if matches!(result, VerificationOutcome::Match { .. }) {
                    matched = true;
                    break;
                }
            }
        }
        Err(error) => return report_error(error),
    }

    if matched {
        CommandStatus::Success
    } else {
        CommandStatus::Failure
    }
}

fn check(algorithm: Option<SupportedAlgorithm>, check_file: PathBuf) -> CommandStatus {
    match parse_shasum_file(&check_file, algorithm) {
        Ok(entries) => {
            if entries.is_empty() {
                return report_error(format!(
                    "Error: No checksum entries found in {}.",
                    check_file.display()
                ));
            }

            let mut has_unmatched_task = false;
            let color = io::stdout().is_terminal();

            for entry in entries {
                let data = entry.data().to_string();
                let mut matched = false;

                match entry.compute() {
                    Ok(results) => {
                        for result in results {
                            println!("{}: {}", data, render_verification(&result, color));
                            if matches!(result, VerificationOutcome::Match { .. }) {
                                matched = true;
                                break;
                            }
                        }
                    }
                    Err(error) => eprintln!("{}: {}", data, error),
                }

                if !matched {
                    has_unmatched_task = true;
                }
            }

            if has_unmatched_task {
                CommandStatus::Failure
            } else {
                CommandStatus::Success
            }
        }
        Err(error) => report_error(error),
    }
}

fn main() -> ExitCode {
    let args = Cli::parse();

    let status = match args.args {
        Args::Calculate { algorithm, input } => calculate(algorithm, input),

        Args::Compare {
            algorithm,
            input,
            check_hash,
        } => compare(algorithm, input, check_hash),

        Args::Check {
            algorithm,
            check_file,
        } => check(algorithm, check_file),
    };

    status.exit_code()
}
