use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn assay(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_assay"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_and_version_report_cli_contract() {
    let help = assay(&["--help"]);
    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("plan"));
    assert!(help.contains("run"));

    let version = assay(&["--version"]);
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));

    let report = assay(&["version"]);
    assert!(report.status.success());
    let report = String::from_utf8_lossy(&report.stdout);
    assert!(report.contains("Manifest protocols: v0.6, v0.7"));
    assert!(report.contains("Evidence schema: assay-evidence/v1"));
}

#[test]
fn unknown_arguments_fail_with_usage_error() {
    let output = assay(&["plan", "--not-a-real-flag"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn plan_json_and_run_share_the_git_input_selection() {
    let root = temporary_directory();
    initialize_project(&root);
    fs::write(
        root.join("src/main.rs"),
        "fn main() { println!(\"changed\"); }\n",
    )
    .unwrap();

    let root_text = root.to_string_lossy().to_string();
    let plan = assay(&[
        "plan", "--root", &root_text, "--base", "HEAD", "--format", "json",
    ]);
    assert!(
        plan.status.success(),
        "{}",
        String::from_utf8_lossy(&plan.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(plan["schema_version"], "assay-plan/v1");
    assert_eq!(plan["checks"].as_array().unwrap().len(), 1);
    assert_eq!(plan["checks"][0]["id"], "K-SRC");

    let run = assay(&["run", "--root", &root_text, "--base", "HEAD"]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(root.join("assay/evidence/K-SRC.jsonl").exists());
    assert!(!root.join("assay/evidence/K-CARGO.jsonl").exists());

    fs::remove_dir_all(root).unwrap();
}

fn temporary_directory() -> std::path::PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("assay-cli-{}-{timestamp}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}

fn initialize_project(root: &Path) {
    git(root, ["init", "-q"]);
    git(root, ["config", "user.name", "Assay Tests"]);
    git(
        root,
        ["config", "user.email", "assay-tests@example.invalid"],
    );
    fs::create_dir_all(root.join("assay")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"assay-fixture\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
    )
    .unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        root.join("assay/manifest.toml"),
        "[assay]\nprotocol = \"v0.7\"\n\n[[check]]\nid = \"K-SRC\"\ncommand = \"echo src\"\nstage = \"pre-commit\"\ninputs = [\"src/\"]\n\n[[check]]\nid = \"K-CARGO\"\ncommand = \"echo cargo\"\nstage = \"pre-commit\"\ninputs = [\"Cargo.toml\"]\n",
    )
    .unwrap();
    git(root, ["add", "."]);
    git(root, ["commit", "-q", "-m", "baseline"]);
}

fn git<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
