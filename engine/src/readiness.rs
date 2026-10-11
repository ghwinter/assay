//! readiness — 只读导览：**现在缺什么、下一步跑哪条命令。**
//!
//! 三件事不许混（本模块存在的理由）：
//!   * `check` 判**清单自身**是否自洽 —— 门禁，红即阻断
//!   * `run`   跑检查、写证据        —— 执行
//!   * 本模块  判**工作还能不能继续**  —— 导览，只读，不跑任何检查
//!
//! 做：读清单 + 证据 + 棘轮 + 目录形态 + 环境可解析性 → 输出人读摘要与 `--format json` 的动作表。
//! 不做：跑任何检查命令；判断分歧里谁对；解释检查输出的语义；认识任何工具。
//!
//! **与环境的唯一接触点**是"声明的命令能否被解析到"：只看 PATH 与仓库相对路径，
//! **不执行**被声明的命令。这条边界是有摩擦买来的——文档里写着 `python tools/gate.py`，
//! 而本机上 `python` 是 Microsoft Store 的占位符：命令存在、锚点不存在。
//! 引擎不需要知道 `python` 是什么，只需要知道"首词解析不到东西"。

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use super::{assay_dir, fnv1a, load_manifest, load_ratchet, map_content, Claim, Manifest, Ratchet};
use crate::cli::OutputFormat;

// ── 报告模型 ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct Finding {
    id: String,
    /// `ok` | `warn` | `error`
    level: String,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<String>,
}

impl Finding {
    fn ok(id: &str, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            level: "ok".into(),
            detail: detail.into(),
            fix: None,
        }
    }
    fn warn(id: &str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            level: "warn".into(),
            detail: detail.into(),
            fix: Some(fix.into()),
        }
    }
    fn error(id: &str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            level: "error".into(),
            detail: detail.into(),
            fix: Some(fix.into()),
        }
    }
    fn is_blocking(&self) -> bool {
        self.level == "error"
    }
}

#[derive(Serialize)]
struct Action {
    order: usize,
    /// 动作类别：`env` | `birth` | `bind` | `discharge` | `stale` | `close` | `drift` | `done`
    kind: String,
    target: String,
    why: String,
    command: String,
}

#[derive(Serialize)]
struct DoctorReport {
    schema_version: &'static str,
    root: String,
    findings: Vec<Finding>,
    blocking: usize,
    warnings: usize,
}

#[derive(Serialize)]
struct NextReport {
    schema_version: &'static str,
    root: String,
    counts: Counts,
    actions: Vec<Action>,
    /// 证据陈旧/缺失的检查（`next` 的核心输出）
    stale_or_missing: Vec<String>,
    note: &'static str,
}

#[derive(Serialize, Default)]
struct Counts {
    claims: usize,
    checks: usize,
    changes: usize,
    certified: usize,
    unproven: usize,
}

// ── 环境可解析性（只查 PATH，不执行） ────────────────────────────────────────

/// 把一条声明命令的**首词**解析成可执行文件。返回 `None` 表示解析不到。
///
/// 含路径分隔符 ⇒ 按仓库相对/绝对路径查文件；否则按 PATH + PATHEXT 查。
/// 这里刻意不执行任何东西：引擎不吃被声明命令的输出，也不该为了"探活"而跑它。
fn resolve_command_in(token: &str, root: &Path) -> Option<PathBuf> {
    let t = token.trim().trim_matches('"').trim_matches('\'');
    if t.is_empty() {
        return None;
    }
    let p = Path::new(t);
    let looks_like_path = t.contains('/') || t.contains('\\');
    if looks_like_path {
        let cand = if p.is_absolute() {
            p.to_path_buf()
        } else {
            root.join(p)
        };
        return cand.is_file().then_some(cand);
    }

    let path_var = env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_lowercase())
            .collect()
    } else {
        vec![String::new()]
    };
    for dir in env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        for ext in &exts {
            let mut name = t.to_string();
            if !ext.is_empty() {
                name.push_str(ext);
            }
            let cand = dir.join(&name);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// 一条命令的首词。
pub(crate) fn first_token(command: &str) -> &str {
    command.split_whitespace().next().unwrap_or("")
}

// ── 证据视图（只读最新投影，不重建结构） ─────────────────────────────────────

#[derive(Default, Clone)]
struct EvidenceView {
    verdict: String,
    started_at: String,
    engine_version: String,
    /// 证据信封的版本状态。三态必须分开，因为处置不同：
    ///   `V1` 当前信封；`Legacy` 无 `schema_version` 的 v0 记录（**已知**旧格式，有迁移路径）；
    ///   `Unknown` 有版本号但不认识（**拒绝解释**，不许猜）。
    envelope: Envelope,
    /// 证据**自身**记录的检查定义（协议 §2.4 的 `checker` 段）
    cmd_fp: String,
    strength: u8,
    scope: String,
    claims: Vec<String>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Envelope {
    #[default]
    Legacy,
    V1,
    Unknown,
}

const KNOWN_SCHEMA: &str = "assay-evidence/v1";

/// 逐条失效判据——**这是"再验证规则"的最小可用形态**（ch07 证据包第 10 项）。
///
/// 刻意**不**用整份清单的指纹当失效判据：那是把"改了一个注释"和"改了检查命令"
/// 判成同一件事，结果是全部证据天天报陈旧，人开始无视它。
/// 正确的粒度是**这条检查自己的定义**：命令 / 档 / 范围 / 绑定的声明。
#[derive(Debug, PartialEq)]
enum Freshness {
    Current,
    DefinitionChanged(String),
    LastVerdict(String),
    /// v0 记录：没有信封，因此**没有**可比的检查定义字段。
    LegacyRecord,
    UnknownSchema,
    Missing,
}

fn freshness(c: &super::Check, ev: Option<&EvidenceView>) -> Freshness {
    let Some(ev) = ev else {
        return Freshness::Missing;
    };
    match ev.envelope {
        Envelope::Legacy => return Freshness::LegacyRecord,
        Envelope::Unknown => return Freshness::UnknownSchema,
        Envelope::V1 => {}
    }
    let now = fnv1a(&c.command);
    if ev.cmd_fp != now {
        return Freshness::DefinitionChanged("检查命令已改".into());
    }
    if ev.strength != c.strength {
        return Freshness::DefinitionChanged(format!(
            "强度档已改 {} → {}",
            ev.strength, c.strength
        ));
    }
    if ev.scope != super::scope_of(c.scope.as_deref(), c.bound) {
        return Freshness::DefinitionChanged("覆盖范围已改".into());
    }
    let mut a = ev.claims.clone();
    let mut b = c.claims.clone();
    a.sort();
    b.sort();
    if a != b {
        return Freshness::DefinitionChanged("绑定的声明已改".into());
    }
    if ev.verdict != "pass" {
        return Freshness::LastVerdict(ev.verdict.clone());
    }
    Freshness::Current
}

/// 读 `<root>/assay/evidence/<check-id>.json`（最新投影）。
/// 解析失败一律当"无证据"处理——**坏掉的证据不能点头**。
fn read_evidence(root: &Path, check_id: &str) -> Option<EvidenceView> {
    let p = assay_dir(root)
        .join("evidence")
        .join(format!("{check_id}.json"));
    let raw = fs::read_to_string(&p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let envelope = match v.get("schema_version").and_then(|s| s.as_str()) {
        Some(s) if s == KNOWN_SCHEMA => Envelope::V1,
        Some(_) => Envelope::Unknown,
        None => Envelope::Legacy,
    };
    Some(EvidenceView {
        verdict: v
            .get("result")
            .and_then(|r| r.pointer("/verdict"))
            .or_else(|| v.pointer("/verdict"))
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string(),
        started_at: v
            .pointer("/result/started_at")
            .or_else(|| v.pointer("/started"))
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        engine_version: v
            .pointer("/checker/engine_version")
            .or_else(|| v.pointer("/engine_version"))
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        envelope,
        cmd_fp: v
            .pointer("/checker/command_fingerprint")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        strength: v
            .pointer("/checker/strength")
            .and_then(|s| s.as_u64())
            .unwrap_or(0) as u8,
        scope: v
            .pointer("/checker/scope")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        claims: v
            .pointer("/lineage/claim_ids")
            .and_then(|s| s.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// 一条证据是否"当前且通过"。三态严格区分：缺证据 ≠ 定义已改 ≠ 通过。
fn evidence_is_current_pass(c: &super::Check, ev: Option<&EvidenceView>) -> bool {
    freshness(c, ev) == Freshness::Current
}

// ── doctor ──────────────────────────────────────────────────────────────────

fn git_head(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_toplevel(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn manifest_text(root: &Path) -> Option<String> {
    fs::read_to_string(assay_dir(root).join("manifest.toml")).ok()
}

fn doctor_findings(root: &Path) -> (Vec<Finding>, Option<Manifest>) {
    let mut f = Vec::new();

    // 1. 仓库根
    match git_toplevel(root) {
        None => f.push(Finding::error(
            "git-repository",
            "不是 git 仓库（或 git 不可执行）",
            "在仓库根运行，或先 git init",
        )),
        Some(top) => {
            let want = fs::canonicalize(root).ok();
            let got = fs::canonicalize(&top).ok();
            if want.is_some() && want == got {
                f.push(Finding::ok("git-repository", format!("仓库根 {top}")));
            } else {
                f.push(Finding::warn(
                    "git-repository",
                    format!("--root 不是 git 顶层（顶层是 {top}）"),
                    format!("改用 --root {top}"),
                ));
            }
        }
    }
    if let Some(head) = git_head(root) {
        f.push(Finding::ok("git-head", head));
    }

    // 2. 清单
    let text = match manifest_text(root) {
        None => {
            f.push(Finding::error(
                "manifest-present",
                "没有 assay/manifest.toml",
                "新项目：见 templates/rust-app/assay/manifest.toml",
            ));
            return (f, None);
        }
        Some(t) => t,
    };
    let manifest = match load_manifest(root) {
        Err(e) => {
            f.push(Finding::error("manifest-parses", e, "修清单"));
            return (f, None);
        }
        Ok(m) => {
            f.push(Finding::ok(
                "manifest-parses",
                format!(
                    "协议 {:?}",
                    m.assay.as_ref().and_then(|a| a.protocol.clone())
                ),
            ));
            m
        }
    };
    let current_fp = fnv1a(&text);

    // 3. 棘轮
    let ratchet = load_ratchet(root);
    f.push(Finding::ok(
        "ratchet-present",
        format!("棘轮条目 {}", ratchet.entry.len()),
    ));

    // 4. 证据目录可写
    let ev_dir = assay_dir(root).join("evidence");
    match fs::create_dir_all(&ev_dir).and_then(|_| {
        let probe = ev_dir.join(".doctor-probe");
        fs::write(&probe, b"ok").and_then(|_| fs::remove_file(&probe))
    }) {
        Ok(()) => f.push(Finding::ok(
            "evidence-writable",
            ev_dir.display().to_string(),
        )),
        Err(e) => f.push(Finding::error(
            "evidence-writable",
            format!("证据目录不可写：{e}"),
            "检查权限或版本控制忽略规则",
        )),
    }

    // 5. 地图新鲜度（与 `check` 同一判据，这里只报不改）
    let fresh = map_content(&manifest, &ratchet)
        == fs::read_to_string(assay_dir(root).join("MAP.md")).unwrap_or_default();
    if fresh {
        f.push(Finding::ok("map-fresh", "MAP.md 与清单一致"));
    } else {
        f.push(Finding::warn(
            "map-fresh",
            "MAP.md 与清单不一致（或缺失）",
            "assay map",
        ));
    }

    // 6. **声明的命令能否解析到**——本模块唯一的对外接触点，且不执行
    let mut unresolved: BTreeSet<String> = BTreeSet::new();
    for c in &manifest.check {
        for (label, cmd) in [
            ("command", Some(&c.command)),
            ("red_command", c.red_command.as_ref()),
        ] {
            let Some(cmd) = cmd else { continue };
            if cmd.trim().is_empty() {
                continue;
            }
            let tok = first_token(cmd);
            if resolve_command_in(tok, root).is_none() {
                unresolved.insert(format!("{} ({label}): {tok}", c.id));
            }
        }
    }
    if unresolved.is_empty() {
        f.push(Finding::ok(
            "commands-resolvable",
            format!(
                "{} 条检查声明的命令首词全部可在 PATH 或仓库内解析",
                manifest.check.len()
            ),
        ));
    } else {
        for u in &unresolved {
            f.push(Finding::error(
                "commands-resolvable",
                format!("命令首词解析不到：{u}"),
                "修 PATH / 装工具 / 改清单里的命令；引擎不猜替代品",
            ));
        }
    }

    // 7. 声明的 inputs 路径存在
    let mut missing_inputs = Vec::new();
    for c in &manifest.check {
        for i in &c.inputs {
            let rel = i.trim_end_matches('/');
            if !rel.is_empty() && !root.join(rel).exists() {
                missing_inputs.push(format!("{}: {i}", c.id));
            }
        }
    }
    if missing_inputs.is_empty() {
        f.push(Finding::ok("inputs-exist", "声明的 inputs 路径全部存在"));
    } else {
        f.push(Finding::warn(
            "inputs-exist",
            format!("inputs 指向不存在的路径：{}", missing_inputs.join(", ")),
            "改 inputs 或补文件；计划选择会因此偏保守（退回全量）",
        ));
    }

    // 8. 存在性核对（**不是解析**）：claim 引用的义务 id 在 ledger/ 里出现过吗
    let mut missing_obl = Vec::new();
    let ledger = root.join("ledger");
    if ledger.is_dir() {
        let mut corpus = String::new();
        for sub in ["decisions", "evidence", "intent"] {
            if let Ok(rd) = fs::read_dir(ledger.join(sub)) {
                for e in rd.flatten() {
                    if let Ok(t) = fs::read_to_string(e.path()) {
                        corpus.push_str(&t);
                    }
                }
            }
        }
        for cl in &manifest.claim {
            for o in &cl.obligations {
                if !corpus.contains(o.as_str()) {
                    missing_obl.push(format!("{} -> {o}", cl.id));
                }
            }
        }
    }
    if !ledger.is_dir() {
        f.push(Finding::warn(
            "obligations-exist",
            "没有 ledger/ 目录，跳过义务存在性核对",
            "只用 Assay 清单的项目可忽略",
        ));
    } else if missing_obl.is_empty() {
        f.push(Finding::ok(
            "obligations-exist",
            "claim 引用的义务 id 均在 ledger/ 中出现（存在性核对，非解析）",
        ));
    } else {
        f.push(Finding::warn(
            "obligations-exist",
            format!("引用了 ledger/ 中找不到的义务：{}", missing_obl.join(", ")),
            "核对 D 记录的义务 id",
        ));
    }

    // 9. 证据新鲜度：**逐条失效判据**（定义变了才算失效，清单文本变只报不判）
    let mut stale: Vec<String> = Vec::new();
    let mut engine_versions: BTreeSet<String> = BTreeSet::new();
    for c in &manifest.check {
        let ev = read_evidence(root, &c.id);
        if let Some(e) = &ev {
            if !e.engine_version.is_empty() {
                engine_versions.insert(e.engine_version.clone());
            }
        }
        match freshness(c, ev.as_ref()) {
            Freshness::Current => {}
            Freshness::Missing => stale.push(format!("{}（无证据）", c.id)),
            Freshness::LegacyRecord => stale.push(format!(
                "{}（旧格式 v0，无版本信封——已知旧格式，可重跑迁移）",
                c.id
            )),
            Freshness::UnknownSchema => {
                stale.push(format!("{}（schema 版本不认识，拒绝解释）", c.id))
            }
            Freshness::DefinitionChanged(w) => stale.push(format!("{}（{w}）", c.id)),
            Freshness::LastVerdict(v) => stale.push(format!("{}（最近判定 {v}）", c.id)),
        }
    }
    if stale.is_empty() {
        f.push(Finding::ok(
            "evidence-current",
            "每条证据都对得上它当前的检查定义",
        ));
    } else {
        f.push(Finding::warn(
            "evidence-current",
            format!(
                "{} 条证据与其检查定义不符或缺失：{}",
                stale.len(),
                stale.join(", ")
            ),
            "assay next 给出逐条动作；assay run 重跑",
        ));
    }
    // 清单文本指纹：**只报不判**。改一个注释会让它变，但那不使任何证据失效——
    // 逐条失效判据见上面的 `evidence-current`。
    f.push(Finding::ok(
        "evidence-provenance",
        format!(
            "清单文本指纹 fnv1a:{}；证据由引擎版本 [{}] 写出（引擎 / 清单协议 / 证据 schema 是三条独立版本轴）",
            current_fp.trim_start_matches("fnv1a:"),
            engine_versions.into_iter().collect::<Vec<_>>().join(", ")
        ),
    ));

    (f, Some(manifest))
}

pub fn doctor(root: &Path, format: OutputFormat) -> i32 {
    let (findings, _) = doctor_findings(root);
    let blocking = findings.iter().filter(|f| f.is_blocking()).count();
    let warnings = findings.iter().filter(|f| f.level == "warn").count();

    if format == OutputFormat::Json {
        let report = DoctorReport {
            schema_version: "assay-doctor/v1",
            root: root.display().to_string(),
            findings,
            blocking,
            warnings,
        };
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("[doctor] JSON 序列化失败：{e}");
                return 2;
            }
        }
    } else {
        println!("== assay doctor ==");
        for f in &findings {
            let mark = match f.level.as_str() {
                "ok" => "✅",
                "warn" => "⚠️ ",
                _ => "❌",
            };
            println!("  {mark} [{}] {}", f.id, f.detail);
            if let Some(fix) = &f.fix {
                println!("       → {fix}");
            }
        }
        println!("== 结论：{} 个阻断 / {} 个警告 ==", blocking, warnings);
    }

    if blocking > 0 {
        2
    } else {
        0
    }
}

// ── next ────────────────────────────────────────────────────────────────────

fn best_discharge<'a>(m: &'a Manifest, cl: &Claim) -> Option<&'a super::Check> {
    m.check
        .iter()
        .filter(|c| c.claims.iter().any(|x| x == &cl.id))
        .find(|c| super::discharge(cl, c).is_ok())
}

/// 纯函数：由"清单 + 棘轮 + 证据 + 环境"算出**有序动作表**。可测。
fn compute_actions(
    m: &Manifest,
    r: &Ratchet,
    ev: &dyn Fn(&str) -> Option<EvidenceView>,
    unresolved: &BTreeSet<String>,
    map_fresh: bool,
) -> (Counts, Vec<Action>, Vec<String>) {
    let mut actions: Vec<Action> = Vec::new();
    let mut stale_or_missing: Vec<String> = Vec::new();

    let certified: BTreeSet<&str> = r
        .entry
        .iter()
        .filter(|e| e.certified)
        .map(|e| e.id.as_str())
        .collect();
    let unproven = m
        .check
        .iter()
        .filter(|c| !certified.contains(c.id.as_str()))
        .count();

    // 1. 环境：解析不到的命令 —— 最高优先级，因为后面全都跑不了
    for u in unresolved {
        actions.push(Action {
            order: 0,
            kind: "env".into(),
            target: u.clone(),
            why: "声明的命令首词解析不到，任何 run 都会失败在起跑线".into(),
            command: "assay doctor".into(),
        });
    }

    // 2. 出生证明：从未红过的检查不计入棘轮
    for c in &m.check {
        if !certified.contains(c.id.as_str()) {
            actions.push(Action {
                order: 0,
                kind: "birth".into(),
                target: c.id.clone(),
                why: match c.red_command.as_deref() {
                    Some(_) => "有 red_command 但尚未被机器复现".into(),
                    None => "缺出生证明（从未见过失败的检查与没有检查统计等价）".into(),
                },
                command: format!("assay run --check {} --certify", c.id),
            });
        }
    }

    // 3. 绑定：声明没有任何检查
    for cl in super::gaps(m) {
        actions.push(Action {
            order: 0,
            kind: "bind".into(),
            target: cl.id.clone(),
            why: "这条声明没有任何检查绑定".into(),
            command: "编辑 assay/manifest.toml 增加 [[check]]".into(),
        });
    }

    // 4. 充分性：有检查但没有任何一条能兑现（档/范围不足）
    for cl in &m.claim {
        if best_discharge(m, cl).is_none() && !super::gaps(m).iter().any(|g| g.id == cl.id) {
            let why = m
                .check
                .iter()
                .filter(|c| c.claims.iter().any(|x| x == &cl.id))
                .map(|c| super::discharge(cl, c).err().unwrap_or_default())
                .collect::<Vec<_>>()
                .join("; ");
            actions.push(Action {
                order: 0,
                kind: "discharge".into(),
                target: cl.id.clone(),
                why: format!("没有检查能兑现它：{why}"),
                command: "升档/扩大范围，或登记 waiver（带 reason + signed）".into(),
            });
        }
    }

    // 5. 陈旧/缺失证据 —— `next` 的核心。逐条失效判据，**不用整份清单指纹**。
    // `why` 必须是**具体原因**，不能是"缺失/已改/非 pass"的三选一：读的人要能直接判断
    // 该重跑还是该改清单。含糊的 why 等于把判断推回给人——那就没省下任何东西。
    let mut stale_set: BTreeMap<&str, String> = BTreeMap::new();
    for c in &m.check {
        let reason = match freshness(c, ev(&c.id).as_ref()) {
            Freshness::Current => None,
            Freshness::Missing => Some("无证据（从未跑过）".to_string()),
            Freshness::LegacyRecord => {
                let when = ev(&c.id).map(|e| e.started_at).unwrap_or_default();
                Some(format!("旧格式 v0 记录，无版本信封（{when}）"))
            }
            Freshness::UnknownSchema => Some("证据 schema 版本不认识——拒绝解释".to_string()),
            Freshness::DefinitionChanged(w) => Some(w),
            Freshness::LastVerdict(v) => {
                let when = ev(&c.id).map(|e| e.started_at).unwrap_or_default();
                Some(format!("最近判定 {v}（{when}）"))
            }
        };
        if let Some(reason) = reason {
            stale_or_missing.push(format!("{}（{reason}）", c.id));
            stale_set.insert(c.id.as_str(), reason);
        }
    }
    for (id, reason) in &stale_set {
        actions.push(Action {
            order: 0,
            kind: "stale".into(),
            target: (*id).to_string(),
            why: reason.clone(),
            command: format!("assay run --check {id}"),
        });
    }

    // 6. 未收口的变更
    for ch in &m.change {
        let required = super::checks_required_by(m, ch);
        let open: Vec<&str> = required
            .iter()
            .filter(|c| !evidence_is_current_pass(c, ev(&c.id).as_ref()))
            .map(|c| c.id.as_str())
            .collect();
        if !open.is_empty() {
            actions.push(Action {
                order: 0,
                kind: "close".into(),
                target: ch.id.clone(),
                why: format!("变更要求的证据未收口：{}", open.join(", ")),
                command: format!("assay run --change {}", ch.id),
            });
        }
    }

    // 7. 地图漂移
    if !map_fresh {
        actions.push(Action {
            order: 0,
            kind: "drift".into(),
            target: "MAP.md".into(),
            why: "MAP.md 与清单不一致（手改即红灯）".into(),
            command: "assay map".into(),
        });
    }

    // 排序：env > birth > bind > discharge > stale/close > drift
    let rank = |k: &str| match k {
        "env" => 0,
        "birth" => 1,
        "bind" => 2,
        "discharge" => 3,
        "close" => 4,
        "stale" => 5,
        _ => 6,
    };
    actions.sort_by(|a, b| {
        rank(&a.kind)
            .cmp(&rank(&b.kind))
            .then_with(|| a.target.cmp(&b.target))
    });
    for (i, a) in actions.iter_mut().enumerate() {
        a.order = i + 1;
    }

    let counts = Counts {
        claims: m.claim.len(),
        checks: m.check.len(),
        changes: m.change.len(),
        certified: certified.len(),
        unproven,
    };
    (counts, actions, stale_or_missing)
}

pub fn next(root: &Path, format: OutputFormat) -> i32 {
    let Ok(manifest) = load_manifest(root) else {
        eprintln!("[清单错误] 读不到或解析不了 assay/manifest.toml；先跑 `assay doctor`");
        return 2;
    };
    let ratchet = load_ratchet(root);
    let ev = |id: &str| read_evidence(root, id);

    let mut unresolved: BTreeSet<String> = BTreeSet::new();
    for c in &manifest.check {
        for cmd in [Some(&c.command), c.red_command.as_ref()]
            .into_iter()
            .flatten()
        {
            if cmd.trim().is_empty() {
                continue;
            }
            let tok = first_token(cmd);
            if resolve_command_in(tok, root).is_none() {
                unresolved.insert(format!("{}: {tok}", c.id));
            }
        }
    }
    let map_fresh = map_content(&manifest, &ratchet)
        == fs::read_to_string(assay_dir(root).join("MAP.md")).unwrap_or_default();

    let (counts, mut actions, stale_or_missing) =
        compute_actions(&manifest, &ratchet, &ev, &unresolved, map_fresh);
    if actions.is_empty() {
        actions.push(Action {
            order: 1,
            kind: "done".into(),
            target: "-".into(),
            why: "没有阻断动作：出生证明齐全、证据当前、变更已收口".into(),
            command: "assay run".into(),
        });
    }

    if format == OutputFormat::Json {
        let report = NextReport {
            schema_version: "assay-next/v1",
            root: root.display().to_string(),
            counts,
            actions,
            stale_or_missing,
            note: "只读导览：不跑检查、不判断分歧里谁对、不解释检查输出",
        };
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("[next] JSON 序列化失败：{e}");
                return 2;
            }
        }
    } else {
        println!("== assay next ==");
        println!(
            "  声明 {} | 检查 {} | 变更 {} | 已认证出生证明 {} | 未认证 {}",
            counts.claims, counts.checks, counts.changes, counts.certified, counts.unproven
        );
        println!("  下一步（{} 条）：", actions.len());
        for a in &actions {
            println!("  {:>2}. [{}] {}", a.order, a.kind, a.target);
            println!("      why: {}", a.why);
            println!("      run: {}", a.command);
        }
        println!("  说明：只读导览；不跑检查、不判断分歧里谁对、不解释检查输出");
    }
    // 导览不是门禁：有动作不等于失败。只有清单本身读不了才非零。
    0
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_token_takes_the_command_head() {
        assert_eq!(first_token("pwsh -NoProfile -File a.ps1"), "pwsh");
        assert_eq!(first_token("  cargo test  "), "cargo");
        assert_eq!(first_token(""), "");
    }

    #[test]
    fn resolve_rejects_empty_and_unknown_bare_names() {
        let root = Path::new(".");
        assert!(resolve_command_in("", root).is_none());
        assert!(resolve_command_in("   ", root).is_none());
        // 一个几乎不可能存在的可执行名：解析不到必须是 None，不许猜
        assert!(resolve_command_in("assay-no-such-binary-xyzzy", root).is_none());
    }

    #[test]
    fn resolve_accepts_repo_relative_paths() {
        let root = std::env::temp_dir().join(format!("assay-readiness-{}", std::process::id()));
        let _ = fs::create_dir_all(root.join("sub"));
        fs::write(root.join("tool.cmd"), b"@echo off\n").unwrap();
        fs::write(root.join("sub").join("tool.cmd"), b"@echo off\n").unwrap();

        assert!(resolve_command_in("./tool.cmd", &root).is_some());
        assert!(resolve_command_in("sub/tool.cmd", &root).is_some());
        assert!(resolve_command_in("missing.cmd", &root).is_none());
        // 含分隔符但不存在 ⇒ None（不是"回退到 PATH 再猜一次"）
        assert!(resolve_command_in("sub/nope.cmd", &root).is_none());
        // 裸文件名按 PATH 查，**不**按仓库根查——与 shell 语义一致，可预测；
        // 要指仓库内的可执行文件就写 ./tool.cmd。
        assert!(resolve_command_in("tool.cmd", &root).is_none());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn freshness_distinguishes_missing_from_definition_change_from_verdict() {
        let check = super::super::Check {
            id: "K-1".into(),
            command: "cargo test".into(),
            strength: 5,
            scope: Some("unbounded".into()),
            claims: vec!["C-1".into()],
            ..Default::default()
        };
        let mut ev = EvidenceView {
            verdict: "pass".into(),
            envelope: Envelope::V1,
            cmd_fp: fnv1a("cargo test"),
            strength: 5,
            scope: "unbounded".into(),
            claims: vec!["C-1".into()],
            ..Default::default()
        };

        assert_eq!(freshness(&check, None), Freshness::Missing);
        assert_eq!(freshness(&check, Some(&ev)), Freshness::Current);
        assert!(evidence_is_current_pass(&check, Some(&ev)));

        // 改了命令 ⇒ 定义变了（不是"仍然有效"）
        ev.cmd_fp = fnv1a("cargo test --all");
        assert!(matches!(
            freshness(&check, Some(&ev)),
            Freshness::DefinitionChanged(_)
        ));

        // 定义没变但上次红 ⇒ 不是当前通过
        ev.cmd_fp = fnv1a("cargo test");
        ev.verdict = "fail".into();
        assert_eq!(
            freshness(&check, Some(&ev)),
            Freshness::LastVerdict("fail".into())
        );
        assert!(!evidence_is_current_pass(&check, Some(&ev)));

        // v0 记录：没有信封 ⇒ 没有可比的检查定义字段（已知旧格式，不是"未知"）
        ev.verdict = "pass".into();
        ev.envelope = Envelope::Legacy;
        assert_eq!(freshness(&check, Some(&ev)), Freshness::LegacyRecord);
        assert!(!evidence_is_current_pass(&check, Some(&ev)));

        // 有版本号但不认识 ⇒ 拒绝解释，既不算当前，也不猜
        ev.envelope = Envelope::Unknown;
        assert_eq!(freshness(&check, Some(&ev)), Freshness::UnknownSchema);
        assert!(!evidence_is_current_pass(&check, Some(&ev)));
    }
}
