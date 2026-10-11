use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_input_path() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "assay-histogram-{}-{nonce}.bin",
        std::process::id()
    ))
}

#[test]
fn cli_counts_a_file_and_emits_json() {
    let path = temporary_input_path();
    fs::write(&path, [0, 1, 1, 2, 255, 255, 255]).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_histogram"))
        .arg(&path)
        .args([
            "--workers",
            "3",
            "--chunk-bytes",
            "2",
            "--queue-capacity",
            "1",
        ])
        .output()
        .unwrap();
    let _ = fs::remove_file(path);

    assert!(result.status.success());
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains("\"bytes_read\":7"));
    assert!(output.contains("\"chunks\":4"));
    assert!(output.contains("\"workers\":3"));
    assert!(output.contains("\"histogram\":[1,2,1"));
}

#[test]
fn cli_reports_invalid_configuration_and_missing_files() {
    let invalid = Command::new(env!("CARGO_BIN_EXE_histogram"))
        .args(["missing.bin", "--workers", "0"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8(invalid.stderr)
        .unwrap()
        .contains("worker count must be positive"));

    let missing = Command::new(env!("CARGO_BIN_EXE_histogram"))
        .arg(temporary_input_path())
        .output()
        .unwrap();
    assert!(!missing.status.success());
    let error = String::from_utf8(missing.stderr).unwrap().to_lowercase();
    assert!(error.contains("no such file") || error.contains("cannot find the file"));
}
