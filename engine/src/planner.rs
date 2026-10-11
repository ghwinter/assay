use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::{Command, Output};

use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckScope {
    pub id: String,
    pub inputs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlannedCheck {
    pub id: String,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Plan {
    pub schema_version: &'static str,
    pub base_revision: String,
    pub changed_paths: Vec<String>,
    pub unmatched_paths: Vec<String>,
    pub full_run_fallback: bool,
    pub checks: Vec<PlannedCheck>,
}

pub struct GitChanges {
    pub base_revision: String,
    pub paths: Vec<String>,
}

pub fn validate_input_scope(scope: &str) -> Result<(), String> {
    let is_directory = scope.ends_with('/');
    let path = if is_directory {
        scope.strip_suffix('/').unwrap_or_default()
    } else {
        scope
    };

    if path.is_empty()
        || path.starts_with('/')
        || path.contains(['\\', ':', '*', '?', '\0'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!(
            "输入范围 {scope:?} 必须是仓库相对路径；目录范围以 / 结尾，不支持 glob"
        ));
    }

    Ok(())
}

pub fn scope_matches(scope: &str, changed_path: &str) -> bool {
    if let Some(directory) = scope.strip_suffix('/') {
        changed_path
            .strip_prefix(directory)
            .is_some_and(|remainder| remainder.starts_with('/'))
    } else {
        scope == changed_path
    }
}

pub fn plan_checks(
    base_revision: String,
    changed_paths: Vec<String>,
    checks: &[CheckScope],
    required_check_ids: &[String],
) -> Plan {
    let changed_paths: Vec<String> = changed_paths
        .into_iter()
        .map(|path| path.replace('\\', "/"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let required: BTreeSet<&str> = required_check_ids.iter().map(String::as_str).collect();
    let mut reasons: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();

    for check in checks {
        if check.inputs.is_empty() {
            reasons
                .entry(check.id.as_str())
                .or_default()
                .insert("no input scope declared; run conservatively".to_string());
        }
        if required.contains(check.id.as_str()) {
            reasons
                .entry(check.id.as_str())
                .or_default()
                .insert("required by the declared change".to_string());
        }
    }

    let mut unmatched_paths = Vec::new();
    for path in &changed_paths {
        let mut matched = false;
        for check in checks {
            for scope in &check.inputs {
                if scope_matches(scope, path) {
                    matched = true;
                    reasons
                        .entry(check.id.as_str())
                        .or_default()
                        .insert(format!("{path} matched input scope {scope}"));
                }
            }
        }
        if !matched {
            unmatched_paths.push(path.clone());
        }
    }

    let full_run_fallback = !unmatched_paths.is_empty();
    if full_run_fallback {
        let paths = unmatched_paths.join(", ");
        for check in checks {
            reasons
                .entry(check.id.as_str())
                .or_default()
                .insert(format!(
                    "unmapped changed path(s) [{paths}]; select all checks conservatively"
                ));
        }
    }

    let planned = checks
        .iter()
        .filter_map(|check| {
            reasons
                .remove(check.id.as_str())
                .map(|reasons| PlannedCheck {
                    id: check.id.clone(),
                    reasons: reasons.into_iter().collect(),
                })
        })
        .collect();

    Plan {
        schema_version: "assay-plan/v1",
        base_revision,
        changed_paths,
        unmatched_paths,
        full_run_fallback,
        checks: planned,
    }
}

pub fn git_changes(root: &Path, base: &str) -> Result<GitChanges, String> {
    if base.trim().is_empty() || base.starts_with('-') {
        return Err("--base 必须是非空 Git revision，且不能以 '-' 开头".to_string());
    }

    let root = root
        .canonicalize()
        .map_err(|error| format!("无法解析目标仓库 {}：{error}", root.display()))?;
    let repository_root = git_output(&root, ["rev-parse", "--show-toplevel"])?;
    let repository_root = String::from_utf8(repository_root.stdout)
        .map_err(|_| "Git 仓库根路径不是 UTF-8".to_string())?;
    let repository_root = Path::new(repository_root.trim())
        .canonicalize()
        .map_err(|error| format!("无法解析 Git 仓库根路径：{error}"))?;
    if root != repository_root {
        return Err(format!(
            "--root 必须是 Git 仓库根目录（目标：{}；仓库根：{}）",
            root.display(),
            repository_root.display()
        ));
    }

    let revision_arg = format!("{base}^{{commit}}");
    let revision_output = git_output(
        &root,
        ["rev-parse", "--verify", "--quiet", revision_arg.as_str()],
    )?;
    let revision = String::from_utf8(revision_output.stdout)
        .map_err(|_| "Git revision 不是 UTF-8".to_string())?
        .trim()
        .to_string();

    let diff = git_output(
        &root,
        [
            "diff",
            "--name-only",
            "-z",
            "--no-renames",
            revision.as_str(),
            "--",
        ],
    )?;
    let untracked = git_output(&root, ["ls-files", "--others", "--exclude-standard", "-z"])?;

    let mut paths = BTreeSet::new();
    paths.extend(decode_git_paths(&diff.stdout)?);
    paths.extend(decode_git_paths(&untracked.stdout)?);

    Ok(GitChanges {
        base_revision: revision,
        paths: paths.into_iter().collect(),
    })
}

fn git_output<const N: usize>(root: &Path, args: [&str; N]) -> Result<Output, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| format!("无法启动 git：{error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if detail.is_empty() {
            format!("git 命令失败，退出码 {:?}", output.status.code())
        } else {
            detail
        });
    }
    Ok(output)
}

fn decode_git_paths(bytes: &[u8]) -> Result<Vec<String>, String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            String::from_utf8(path.to_vec())
                .map(|path| path.replace('\\', "/"))
                .map_err(|_| {
                    "Git 返回了非 UTF-8 路径；当前 Assay CLI 仅支持 UTF-8 路径".to_string()
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{git_changes, plan_checks, scope_matches, validate_input_scope, CheckScope};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn check(id: &str, inputs: &[&str]) -> CheckScope {
        CheckScope {
            id: id.to_string(),
            inputs: inputs.iter().map(|input| (*input).to_string()).collect(),
        }
    }

    #[test]
    fn input_scopes_accept_relative_files_and_directory_prefixes() {
        assert!(validate_input_scope("Cargo.toml").is_ok());
        assert!(validate_input_scope("src/").is_ok());
        assert!(validate_input_scope("../outside").is_err());
        assert!(validate_input_scope("C:/outside").is_err());
        assert!(validate_input_scope("src/*.rs").is_err());
    }

    #[test]
    fn directory_scope_matches_only_descendants() {
        assert!(scope_matches("src/", "src/main.rs"));
        assert!(!scope_matches("src/", "src"));
        assert!(!scope_matches("src/", "src-old/main.rs"));
        assert!(scope_matches("Cargo.toml", "Cargo.toml"));
        assert!(!scope_matches("Cargo.toml", "Cargo.lock"));
    }

    #[test]
    fn plan_selects_matching_and_unscoped_checks() {
        let checks = [
            check("K-SRC", &["src/"]),
            check("K-MANIFEST", &["Cargo.toml"]),
            check("K-ALWAYS", &[]),
        ];
        let plan = plan_checks(
            "abc123".to_string(),
            vec!["src/main.rs".to_string()],
            &checks,
            &[],
        );

        assert!(!plan.full_run_fallback);
        assert_eq!(
            plan.checks
                .iter()
                .map(|check| check.id.as_str())
                .collect::<Vec<_>>(),
            ["K-SRC", "K-ALWAYS"]
        );
    }

    #[test]
    fn plan_falls_back_to_all_checks_for_unmapped_paths() {
        let checks = [
            check("K-SRC", &["src/"]),
            check("K-MANIFEST", &["Cargo.toml"]),
        ];
        let plan = plan_checks(
            "abc123".to_string(),
            vec!["build.rs".to_string()],
            &checks,
            &[],
        );

        assert!(plan.full_run_fallback);
        assert_eq!(plan.unmatched_paths, ["build.rs"]);
        assert_eq!(plan.checks.len(), checks.len());
    }

    #[test]
    fn plan_includes_change_required_checks_even_without_path_match() {
        let checks = [
            check("K-SRC", &["src/"]),
            check("K-PROTOCOL", &["PROTOCOL.md"]),
        ];
        let plan = plan_checks(
            "abc123".to_string(),
            vec!["src/main.rs".to_string()],
            &checks,
            &["K-PROTOCOL".to_string()],
        );

        assert_eq!(plan.checks.len(), 2);
        assert!(plan.checks[1]
            .reasons
            .iter()
            .any(|reason| reason == "required by the declared change"));
    }

    #[test]
    fn git_diff_includes_staged_unstaged_deleted_and_untracked_paths() {
        let root = temporary_directory();
        initialize_repository(&root);
        fs::write(
            root.join("src/main.rs"),
            "fn main() { println!(\"changed\"); }\n",
        )
        .unwrap();
        git(&root, ["add", "src/main.rs"]);
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"changed\"\n").unwrap();
        fs::remove_file(root.join("README.md")).unwrap();
        fs::write(root.join("new.rs"), "fn new_file() {}\n").unwrap();

        let changes = git_changes(&root, "HEAD").unwrap();

        assert_eq!(
            changes.paths,
            ["Cargo.toml", "README.md", "new.rs", "src/main.rs"]
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn temporary_directory() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("assay-planner-{}-{timestamp}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn initialize_repository(root: &Path) {
        git(root, ["init", "-q"]);
        git(root, ["config", "user.name", "Assay Tests"]);
        git(
            root,
            ["config", "user.email", "assay-tests@example.invalid"],
        );
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"fixture\"\n").unwrap();
        fs::write(root.join("README.md"), "fixture\n").unwrap();
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
}
