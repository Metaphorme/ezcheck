use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_DIR_ID: AtomicU64 = AtomicU64::new(0);

fn ezcheck_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ezcheck")
}

fn expected_usage(subcommand: &str) -> String {
    format!(
        "Usage: ezcheck{} {subcommand}",
        std::env::consts::EXE_SUFFIX
    )
}

fn unique_temp_dir() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let sequence = NEXT_TEMP_DIR_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "ezcheck-cli-test-{}-{timestamp}-{sequence}",
        std::process::id(),
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(target_os = "macos")]
fn unsupported_non_utf8_path(error: &std::io::Error) -> bool {
    // macOS 文件系统拒绝非法 UTF-8 路径时返回 EILSEQ（errno 92）。
    error.raw_os_error() == Some(92)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn unsupported_non_utf8_path(_error: &std::io::Error) -> bool {
    false
}

#[cfg(unix)]
fn write_non_utf8_fixture(path: &std::path::Path, contents: &[u8]) -> bool {
    match fs::write(path, contents) {
        Ok(()) => true,
        Err(error) if unsupported_non_utf8_path(&error) => {
            eprintln!("skipped: filesystem cannot create non-UTF-8 fixture path ({error})");
            false
        }
        Err(error) => panic!("cannot write non-UTF-8 fixture {path:?}: {error}"),
    }
}

#[test]
fn calculate_text_prints_hash_only() {
    let output = Command::new(ezcheck_bin())
        .args(["calculate", "sha256", "-t", "Hello"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969\n"
    );
}

#[test]
fn calculate_help_uses_the_algorithm_catalog() {
    let output = Command::new(ezcheck_bin())
        .args(["calculate", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.to_ascii_lowercase().contains("possible values:"));
    assert!(stdout.contains("sha512_256"));

    #[cfg(feature = "hashes_backend")]
    assert!(stdout.contains("md2"));
    #[cfg(not(feature = "hashes_backend"))]
    assert!(!stdout.contains("md2"));
}

#[test]
fn calculate_returns_non_zero_when_any_file_cannot_be_read() {
    let dir = unique_temp_dir();
    let missing_file = dir.join("missing.txt");

    let output = Command::new(ezcheck_bin())
        .args(["calculate", "sha256", "-f"])
        .arg(&missing_file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Cannot open file"));
}

#[test]
fn calculate_rejects_a_missing_input_as_a_usage_error() {
    let output = Command::new(ezcheck_bin())
        .args(["calculate", "sha256"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected = expected_usage("calculate");
    assert!(stderr.contains(&expected), "unexpected stderr: {stderr}");
}

#[test]
fn compare_rejects_multiple_inputs_as_a_usage_error() {
    let output = Command::new(ezcheck_bin())
        .args([
            "compare",
            "sha256",
            "-f",
            "payload.txt",
            "-t",
            "Hello",
            "-c",
            "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected = expected_usage("compare");
    assert!(stderr.contains(&expected), "unexpected stderr: {stderr}");
}

#[test]
fn compare_requires_a_hash_as_a_usage_error() {
    let output = Command::new(ezcheck_bin())
        .args(["compare", "sha256", "-t", "Hello"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--check-hash"));
}

#[test]
fn check_requires_a_check_file_as_a_usage_error() {
    let output = Command::new(ezcheck_bin())
        .args(["check", "sha256"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--check-file"));
}

#[test]
fn check_rejects_an_empty_check_file() {
    let dir = unique_temp_dir();
    let check_path = dir.join("empty.txt");
    fs::write(&check_path, b"\n \t\r\n").unwrap();

    let output = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No checksum entries"));
}

#[test]
fn compare_returns_zero_when_hash_matches() {
    let output = Command::new(ezcheck_bin())
        .args([
            "compare",
            "sha256",
            "-t",
            "Hello",
            "-c",
            "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "SHA256 OK\n");
}

#[test]
fn compare_returns_non_zero_when_hash_does_not_match() {
    let output = Command::new(ezcheck_bin())
        .args([
            "compare",
            "sha256",
            "-t",
            "Hello",
            "-c",
            "085f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "SHA256 FAILED  Current Hash:185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969\n"
    );
}

#[test]
fn compare_rejects_invalid_hash_for_explicit_algorithm() {
    let output = Command::new(ezcheck_bin())
        .args(["compare", "sha256", "-t", "Hello", "-c", "not-a-hash"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid hash"));
}

#[test]
fn compare_auto_detects_hash_from_stdin_without_rereading_input() {
    let mut child = Command::new(ezcheck_bin())
        .args([
            "compare",
            "-f",
            "-",
            "-c",
            "7e75b18b88d2cb8be95b05ec611e54e2460408a2dcf858f945686446c9d07aac",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(b"Hello").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("SHA512_256 OK"));
}

#[test]
fn check_returns_zero_when_all_entries_match() {
    let dir = unique_temp_dir();
    let file_path = dir.join("payload.txt");
    let check_path = dir.join("sha256sum.txt");

    fs::write(&file_path, b"Hello").unwrap();
    fs::write(
        &check_path,
        "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969  payload.txt\n",
    )
    .unwrap();

    let output = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "payload.txt: SHA256 OK\n"
    );
}

#[test]
fn check_supports_file_names_with_spaces() {
    let dir = unique_temp_dir();
    let file_path = dir.join("payload with spaces.txt");
    let check_path = dir.join("sha256sum.txt");

    fs::write(&file_path, b"Hello").unwrap();
    fs::write(
        &check_path,
        "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969  payload with spaces.txt\n",
    )
    .unwrap();

    let output = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "payload with spaces.txt: SHA256 OK\n"
    );
}

#[cfg(unix)]
#[test]
fn check_supports_non_utf8_file_names() {
    let dir = unique_temp_dir();
    let file_name = std::ffi::OsString::from_vec(b"payload-\xff.bin".to_vec());
    let file_path = dir.join(&file_name);
    let check_path = dir.join("sha256sum.txt");
    let mut check_line =
        b"185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969  ".to_vec();
    check_line.extend_from_slice(file_name.as_os_str().as_bytes());
    check_line.push(b'\n');

    if !write_non_utf8_fixture(&file_path, b"Hello") {
        return;
    }
    fs::write(&check_path, check_line).unwrap();

    let output = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("SHA256 OK"));
}

#[cfg(unix)]
#[test]
fn check_supports_escaped_shasum_file_names() {
    let dir = unique_temp_dir();
    let file_path = dir.join("payload\\name.txt");
    let check_path = dir.join("sha256sum.txt");

    fs::write(&file_path, b"Hello").unwrap();
    fs::write(
        &check_path,
        "\\185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969  payload\\\\name.txt\n",
    )
    .unwrap();

    let output = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("SHA256 OK"));
}

#[test]
fn check_returns_non_zero_when_any_entry_does_not_match() {
    let dir = unique_temp_dir();
    let file_path = dir.join("payload.txt");
    let check_path = dir.join("sha256sum.txt");

    fs::write(&file_path, b"Hello").unwrap();
    fs::write(
        &check_path,
        "085f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969  payload.txt\n",
    )
    .unwrap();

    let output = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "payload.txt: SHA256 FAILED  Current Hash:185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969\n"
    );
}

#[test]
fn check_verifies_prefixed_entries_as_distinct_records() {
    let dir = unique_temp_dir();
    let file_path = dir.join("payload.txt");
    let check_path = dir.join("mixed-algorithms.txt");
    let sha256 = "185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969";

    fs::write(&file_path, b"Hello").unwrap();
    fs::write(
        &check_path,
        format!("sha256:{sha256}  payload.txt\nsha512/256:{sha256}  payload.txt\n"),
    )
    .unwrap();

    let output = Command::new(ezcheck_bin())
        .args(["check", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("SHA512_256 FAILED"));
}

#[test]
fn generated_check_file_in_another_directory_round_trips() {
    let dir = unique_temp_dir();
    let data_dir = dir.join("data");
    let manifest_dir = dir.join("manifests");
    let file_path = data_dir.join("payload.txt");
    let check_path = manifest_dir.join("sha256sum.txt");
    fs::create_dir_all(&data_dir).unwrap();
    fs::create_dir_all(&manifest_dir).unwrap();
    fs::write(&file_path, b"Hello").unwrap();

    let generated = Command::new(ezcheck_bin())
        .args(["calculate", "sha256", "-f", "data/payload.txt"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(generated.status.success());
    fs::write(&check_path, generated.stdout).unwrap();

    let checked = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(checked.status.success());
    assert_eq!(
        String::from_utf8_lossy(&checked.stdout),
        "data/payload.txt: SHA256 OK\n"
    );
}

#[cfg(unix)]
#[test]
fn generated_check_file_preserves_special_file_name_bytes() {
    let dir = unique_temp_dir();
    let file_name = std::ffi::OsString::from_vec(b"payload\\line\ncarriage\r-\xff.bin".to_vec());
    let file_path = dir.join(&file_name);
    let check_path = dir.join("sha256sum.txt");
    if !write_non_utf8_fixture(&file_path, b"Hello") {
        return;
    }

    let generated = Command::new(ezcheck_bin())
        .args(["calculate", "sha256", "-f"])
        .arg(&file_name)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(generated.status.success());
    assert!(generated.stdout.starts_with(b"\\"));
    assert!(generated
        .stdout
        .ends_with(b"  payload\\\\line\\ncarriage\\r-\xff.bin\n"));
    fs::write(&check_path, generated.stdout).unwrap();

    let checked = Command::new(ezcheck_bin())
        .args(["check", "sha256", "-c"])
        .arg(&check_path)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(checked.status.success());
}
