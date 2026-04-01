use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn ezcheck_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ezcheck")
}

fn unique_temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "ezcheck-cli-test-{}-{}",
        std::process::id(),
        suffix
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
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
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "\u{1b}[32mSHA256 OK\u{1b}[0m\n"
    );
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
        "\u{1b}[31mSHA256 FAILED\u{1b}[0m  Current Hash:185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969\n"
    );
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
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}: \u{1b}[32mSHA256 OK\u{1b}[0m\n",
            file_path.display()
        )
    );
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
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}: \u{1b}[31mSHA256 FAILED\u{1b}[0m  Current Hash:185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969\n",
            file_path.display()
        )
    );
}
