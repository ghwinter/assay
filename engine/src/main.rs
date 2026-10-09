//! assay engine v0.1 — 协议 v0.1 的哑执行器
//!
//! 做：读清单 → 按阶段选检查 → 跑命令 → 写证据 → 生成地图 → 比对棘轮 → 输出最小反例
//! 不做：解释任何检查输出的语义；认识任何工具；拥有任何产物；替人做判断
//!
//! 用法：
//!   assay run   [--root <path>] [--stage <s>] [--check <id>]
//!   assay map   [--root <path>]
//!   assay check [--root <path>]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

// ── 清单模型（协议 v0.1 §2） ────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct Manifest {
    #[serde(default)]
    assay: Option<Meta>,
    #[serde(default)]
    claim: Vec<Claim>,
    #[serde(default)]
    check: Vec<Check>,
    #[serde(default)]
    waiver: Vec<Waiver>,
    #[serde(default)]
    change: Vec<Change>,
}

/// 变更记录（协议 v0.4 §2.5）。
/// 存在的理由：一次真实摩擦——变化只能事后从 diff 里猜。
/// 它必须在改代码**之前**写下：这次变化要求哪些证据。
#[derive(Deserialize, Default)]
struct Change {
    id: String,
    #[serde(default)]
    intent: String,
    #[serde(default)]
    claims_added: Vec<String>,
    #[serde(default)]
    claims_removed: Vec<String>,
    #[serde(default)]
    evidence_required: Vec<String>,
    #[serde(default)]
    touches: Vec<String>,
    #[serde(default)]
    obligations_added: Vec<String>,
    #[serde(default)]
    obligations_removed: Vec<String>,
    #[serde(default)]
    obligations_touched: Vec<String>,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    signed: String,
}

#[derive(Deserialize, Default)]
struct Meta {
    #[serde(default)]
    protocol: Option<String>,
}

#[derive(Deserialize, Default)]
struct Claim {
    id: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    rejected_by: String,
    #[serde(default)]
    capability: String,
    #[serde(default)]
    strength_required: u8,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    bound: Option<u64>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    trigger: Option<String>,
    #[serde(default)]
    trust_assumption: Option<String>,
    #[serde(default)]
    obligations: Vec<String>,
    #[serde(default)]
    assumption_ids: Vec<String>,
    #[serde(default)]
    origin: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
struct Check {
    id: String,
    #[serde(default)]
    claims: Vec<String>,
    #[serde(default)]
    capability: String,
    #[serde(default)]
    strength: u8,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    bound: Option<u64>,
    #[serde(default)]
    command: String,
    #[serde(default)]
    stage: String,
    #[serde(default)]
    budget_s: Option<u64>,
    #[serde(default)]
    seen_red: Option<String>,
    #[serde(default)]
    red_command: Option<String>,
    #[serde(default)]
    skip_reason: Option<String>,
    #[serde(default)]
    checker_version: Option<String>,
}

#[derive(Deserialize, Default)]
struct Waiver {
    check: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    signed: String,
}

// ── 棘轮（协议 v0.1 §6） ────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct Ratchet {
    #[serde(default)]
    entry: Vec<Entry>,
}

/// 判定是否"充分"（协议 v0.2 §4）——**强度与范围两个维度一起判**：
///   · 档：检查 `strength` ≥ 声明 `strength_required`
///   · 范围：无界声明必须有 `unbounded` 检查；有界声明需 `unbounded` 或界 ≥ 声明的界
///
/// 实测教训（实验 E2，映射见 ../README.md）：一个名叫 `verify` 的检查器只覆盖 ≤25 步。
/// **只看"档"会让它静默冒充穷举/证明**——这是强度通胀的入口。
/// 反过来，只看范围也会误杀：有界声明被界更大的有界检查兑现，是完全正当的。
fn discharge(cl: &Claim, c: &Check) -> Result<(), String> {
    if c.strength < cl.strength_required {
        return Err(format!(
            "档不足：检查 {} < 声明要求 {}",
            c.strength, cl.strength_required
        ));
    }
    let cscope = c.scope.as_deref().unwrap_or("sampled"); // 缺省 fail closed
    match cl.scope.as_deref().unwrap_or("unbounded") {
        "unbounded" => {
            if cscope == "unbounded" {
                Ok(())
            } else {
                Err(format!("声明是无界的，检查范围只有 {cscope}"))
            }
        }
        "bounded" => {
            let need = cl.bound.unwrap_or(0);
            match cscope {
                "unbounded" => Ok(()),
                "bounded" => {
                    let have = c.bound.unwrap_or(0);
                    if have >= need {
                        Ok(())
                    } else {
                        Err(format!("界不足：检查 ≤{have} 步 < 声明 ≤{need} 步"))
                    }
                }
                other => Err(format!("声明是 ≤{need} 步有界命题，检查范围却是 {other}")),
            }
        }
        _ => Ok(()),
    }
}

fn scope_of(s: Option<&str>, b: Option<u64>) -> String {
    match s.unwrap_or("sampled") {
        "unbounded" => "unbounded".to_string(),
        "bounded" => format!("bounded ≤{}", b.unwrap_or(0)),
        "sampled" => "sampled".to_string(),
        o => format!("未知({o})"),
    }
}

/// 范围强弱的可比排序：unbounded > bounded(界越大越强) > sampled
fn scope_strength(s: Option<&str>, b: Option<u64>) -> (u8, u64) {
    match s.unwrap_or("sampled") {
        "unbounded" => (3, u64::MAX),
        "bounded" => (2, b.unwrap_or(0)),
        _ => (1, 0),
    }
}

fn claim_scope_of(cl: &Claim) -> String {
    match cl.scope.as_deref().unwrap_or("unbounded") {
        "unbounded" => "unbounded".to_string(),
        "bounded" => format!("bounded ≤{}", cl.bound.unwrap_or(0)),
        "sampled" => "sampled".to_string(),
        o => format!("未知({o})"),
    }
}

/// 一条声明被兑现，只要**至少一条**检查足以兑现它；
/// 挂在同一声明上的弱检查（例如 pre-commit 的采样冒烟）是早期预警，不构成"未充分判定"。
fn first_discharge<'a>(m: &'a Manifest, cl: &Claim) -> Option<&'a Check> {
    m.check
        .iter()
        .filter(|c| c.claims.iter().any(|x| x == &cl.id))
        .find(|c| discharge(cl, c).is_ok())
}

#[derive(Deserialize, Default, Clone)]
struct Entry {
    id: String,
    #[serde(default)]
    claims: Vec<String>,
    #[serde(default)]
    obligations: Vec<String>,
    #[serde(default)]
    capability: String,
    #[serde(default)]
    strength: u8,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    bound: Option<u64>,
    #[serde(default)]
    cmd_fp: String,
    /// 出生证明是否被**机器复现过**（`red_command` 真的失败过一次）。
    /// 自述字符串永远进不了这个字段——这正是 v0.4 要修的那个洞。
    #[serde(default)]
    certified: bool,
}

#[derive(Serialize)]
struct EvidenceLineage {
    #[serde(skip_serializing_if = "Option::is_none")]
    change_id: Option<String>,
    claim_ids: Vec<String>,
    obligation_ids: Vec<String>,
    origins: Vec<String>,
    assumption_ids: Vec<String>,
    trust_assumptions: Vec<String>,
}

#[derive(Serialize)]
struct EvidenceSubject {
    #[serde(skip_serializing_if = "Option::is_none")]
    revision: Option<String>,
    manifest_fingerprint: String,
}

#[derive(Serialize)]
struct EvidenceChecker {
    id: String,
    capability: String,
    strength: u8,
    scope: Option<String>,
    bound: Option<u64>,
    stage: String,
    command_fingerprint: String,
    engine_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    checker_version: Option<String>,
}

#[derive(Serialize)]
struct EvidenceResult {
    verdict: String,
    exit_code: Option<i32>,
    started_at: String,
    duration_s: f64,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    counterexample: Option<String>,
}

#[derive(Serialize)]
struct BirthCertificate {
    status: String,
    note: String,
}

#[derive(Serialize)]
struct EvidenceRecord {
    schema_version: &'static str,
    event_id: String,
    run_id: String,
    lineage: EvidenceLineage,
    subject: EvidenceSubject,
    checker: EvidenceChecker,
    result: EvidenceResult,
    birth_certificate: BirthCertificate,
}

// ── 工具 ────────────────────────────────────────────────────────────────────

fn fnv1a(s: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a:{h:016x}")
}

fn claim_has_exactly_one_source(claim: &Claim) -> bool {
    let has_obligations =
        !claim.obligations.is_empty() && claim.obligations.iter().all(|id| !id.trim().is_empty());
    let has_origin = claim
        .origin
        .as_deref()
        .is_some_and(|origin| !origin.trim().is_empty());
    let has_blank_origin = claim
        .origin
        .as_deref()
        .is_some_and(|origin| origin.trim().is_empty());
    !has_blank_origin && (has_obligations ^ has_origin)
}

/// UTC ISO8601（不引第三方时间库；证据只需可排序、无歧义）
fn iso8601_utc() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // civil_from_days（Howard Hinnant 的算法）
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn new_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("RUN-{nanos}-{}", std::process::id())
}

fn tail(s: &str, n: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= n {
        return t.to_string();
    }
    let skip = t.chars().count() - n;
    t.chars().skip(skip).collect()
}

fn append_jsonl(path: &Path, line: &str) -> std::io::Result<()> {
    let mut record = line.as_bytes().to_vec();
    record.push(b'\n');
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(&record)?;
    file.flush()
}

/// 把清单给的字符串原样交给平台壳。引擎不解析它，也不认识里面的工具。
/// 语义随平台：Windows `cmd /C`，其它 `sh -c`。
fn exec_shell(root: &Path, command: &str) -> (i32, String, f64) {
    let t0 = SystemTime::now();
    #[cfg(windows)]
    let out = {
        // raw_arg：绕过 Rust 的 argv 引号转义，让 cmd 收到清单里的原样字符串
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").raw_arg(command).current_dir(root);
        cmd.output()
    };
    #[cfg(not(windows))]
    let out = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(root)
        .output();
    let secs = t0.elapsed().map(|d| d.as_secs_f64()).unwrap_or(0.0);
    match out {
        Ok(o) => {
            let mut all = String::from_utf8_lossy(&o.stdout).to_string();
            all.push_str(&String::from_utf8_lossy(&o.stderr));
            (o.status.code().unwrap_or(-1), all, secs)
        }
        Err(e) => (-1, format!("执行失败：{e}"), secs),
    }
}

/// 出生证明的三种状态（协议 v0.4 §5）。
/// `Reproduced` 是唯一被机器证明过的：它来自一条**期望失败**的命令，这一跑真的失败了。
/// `SelfReport` 是人/模型写的一句话——引擎无法验证它。狗粮暴露了这个洞：
/// 协议里"最重要的一条"曾经是唯一一条完全靠自觉的。
#[derive(PartialEq, Clone, Copy)]
enum Birth {
    Reproduced,
    SelfReport,
    Missing,
}

fn birth_rank(b: Birth) -> &'static str {
    match b {
        Birth::Reproduced => "复现",
        Birth::SelfReport => "自述",
        Birth::Missing => "缺失",
    }
}

fn birth_code(b: Birth) -> &'static str {
    match b {
        Birth::Reproduced => "reproduced",
        Birth::SelfReport => "self_reported",
        Birth::Missing => "missing",
    }
}

/// 一次变更要求哪些检查（协议 v0.4 §2.5）：
/// 显式列出的 ∪ (新增声明 ∪ 触及的声明) 上绑定的所有检查。
/// 这不是"影响分析"——它只是把清单里已有的绑定当作边。**不引入新理论。**
fn checks_required_by<'a>(m: &'a Manifest, ch: &Change) -> Vec<&'a Check> {
    let mut ids: Vec<String> = ch.evidence_required.clone();
    ids.extend(ch.claims_added.iter().cloned());
    ids.extend(ch.touches.iter().cloned());
    let mut obligation_ids = ch.obligations_added.clone();
    obligation_ids.extend(ch.obligations_touched.iter().cloned());
    let mut out: Vec<&'a Check> = Vec::new();
    for c in &m.check {
        let hit = ids.iter().any(|x| x == &c.id)
            || c.claims.iter().any(|cl| ids.iter().any(|x| x == cl))
            || c.claims.iter().any(|claim_id| {
                m.claim
                    .iter()
                    .find(|claim| claim.id == *claim_id)
                    .map(|claim| {
                        claim
                            .obligations
                            .iter()
                            .any(|obligation_id| obligation_ids.contains(obligation_id))
                    })
                    .unwrap_or(false)
            });
        if hit && !out.iter().any(|o| o.id == c.id) {
            out.push(c);
        }
    }
    out
}

// ── 载入 ────────────────────────────────────────────────────────────────────

fn assay_dir(root: &Path) -> PathBuf {
    root.join("assay")
}

fn load_manifest(root: &Path) -> Result<Manifest, String> {
    let p = assay_dir(root).join("manifest.toml");
    let raw = fs::read_to_string(&p).map_err(|e| format!("读不到清单 {}: {e}", p.display()))?;
    toml::from_str(&raw).map_err(|e| format!("清单解析失败 {}: {e}", p.display()))
}

fn load_ratchet(root: &Path) -> Ratchet {
    let p = assay_dir(root).join("ratchet.toml");
    match fs::read_to_string(&p) {
        Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
        Err(_) => Ratchet::default(),
    }
}

fn entry_of(m: &Manifest, c: &Check, certified: bool) -> Entry {
    let mut claims = c.claims.clone();
    claims.sort();
    let mut obligations: Vec<String> = c
        .claims
        .iter()
        .filter_map(|claim_id| m.claim.iter().find(|claim| claim.id == *claim_id))
        .flat_map(|claim| claim.obligations.iter().cloned())
        .collect();
    obligations.sort();
    obligations.dedup();
    Entry {
        id: c.id.clone(),
        claims,
        obligations,
        capability: c.capability.clone(),
        strength: c.strength,
        scope: c.scope.clone(),
        bound: c.bound,
        cmd_fp: fnv1a(&c.command),
        certified,
    }
}

fn save_ratchet(root: &Path, r: &Ratchet) -> Result<(), String> {
    let mut s = String::from(
        "# 棘轮：已兑现检查的指纹（协议 v0.1 §6）。由引擎写；放宽必须走 [[waiver]]。\n\n",
    );
    for e in &r.entry {
        s.push_str("[[entry]]\n");
        s.push_str(&format!("id = {:?}\n", e.id));
        s.push_str(&format!(
            "claims = [{}]\n",
            e.claims
                .iter()
                .map(|c| format!("{c:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        s.push_str(&format!(
            "obligations = [{}]\n",
            e.obligations
                .iter()
                .map(|id| format!("{id:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        s.push_str(&format!("capability = {:?}\n", e.capability));
        s.push_str(&format!("strength = {}\n", e.strength));
        if let Some(sc) = &e.scope {
            s.push_str(&format!("scope = {sc:?}\n"));
        }
        if let Some(b) = e.bound {
            s.push_str(&format!("bound = {b}\n"));
        }
        s.push_str(&format!("cmd_fp = {:?}\n", e.cmd_fp));
        if e.certified {
            s.push_str("certified = true\n");
        }
        s.push('\n');
    }
    let p = assay_dir(root).join("ratchet.toml");
    fs::write(&p, s).map_err(|e| format!("写棘轮失败 {}: {e}", p.display()))
}

// ── 地图（协议 v0.1 §7） ───────────────────────────────────────────────────

fn map_content(m: &Manifest, r: &Ratchet) -> String {
    let mut s = String::new();
    s.push_str(
        "# MAP\n\n> **机械生成**（assay engine）—— 禁止手改；漂移即红灯（`assay check`）。\n\n",
    );

    let unproven: Vec<&Check> = m
        .check
        .iter()
        .filter(|c| c.seen_red.is_none() && c.red_command.is_none())
        .collect();
    let reproducible = m.check.iter().filter(|c| c.red_command.is_some()).count();

    let mut caps: std::collections::BTreeMap<&str, usize> = Default::default();
    for c in &m.check {
        *caps.entry(c.capability.as_str()).or_default() += 1;
    }
    let mut strs: std::collections::BTreeMap<u8, usize> = Default::default();
    for c in &m.check {
        *strs.entry(c.strength).or_default() += 1;
    }

    s.push_str("## 摘要\n\n| 项 | 值 |\n|---|---|\n");
    s.push_str(&format!("| 声明 | {} |\n", m.claim.len()));
    s.push_str(&format!("| 检查 | {} |\n", m.check.len()));
    s.push_str(&format!(
        "| 能力分布 | {} |\n",
        caps.iter()
            .map(|(k, v)| format!("`{k}` × {v}"))
            .collect::<Vec<_>>()
            .join("、")
    ));
    s.push_str(&format!(
        "| 强度分布 | {} |\n",
        strs.iter()
            .map(|(k, v)| format!("档 {k} × {v}"))
            .collect::<Vec<_>>()
            .join("、")
    ));
    s.push_str(&format!(
        "| **棘轮高度** | **{}**（已兑现且带出生证明） |\n",
        r.entry.len()
    ));
    s.push_str(&format!("| `unproven` 检查 | {} |\n", unproven.len()));
    s.push_str(&format!(
        "| 出生证明 | 可复现 **{reproducible}** ／ 自述 **{}** ／ 缺失 **{}** |\n",
        m.check.len() - reproducible - unproven.len(),
        unproven.len()
    ));
    s.push_str(&format!("| 变更记录 | {} |\n", m.change.len()));
    let gaps = gaps(m);
    s.push_str(&format!("| 未绑定任何检查的声明 | {} |\n", gaps.len()));

    s.push_str("\n## 声明 → 来源 → 检查\n\n| 声明 | 来源 | 要求能力 | 要求档 | 要求范围 | 判定它的检查 | 检查档 | 检查范围 | 是否充分 |\n|---|---|---|---|---|---|---|---|---|\n");
    for cl in &m.claim {
        let source = if cl.obligations.is_empty() {
            format!("origin: {}", cl.origin.as_deref().unwrap_or("—"))
        } else {
            cl.obligations.join(", ")
        };
        let mine: Vec<&Check> = m
            .check
            .iter()
            .filter(|c| c.claims.iter().any(|x| x == &cl.id))
            .collect();
        if mine.is_empty() {
            s.push_str(&format!(
                "| {} {} | {} | {} | {} | {} | — | — | — | ❌ 无检查 |\n",
                cl.id,
                tail(&cl.text, 30),
                source,
                cl.capability,
                cl.strength_required,
                claim_scope_of(cl)
            ));
        } else {
            for c in mine {
                let verdict = match discharge(cl, c) {
                    Ok(()) => "✅".to_string(),
                    Err(why) => format!("❌ {why}"),
                };
                s.push_str(&format!(
                    "| {} {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                    cl.id,
                    tail(&cl.text, 30),
                    source,
                    cl.capability,
                    cl.strength_required,
                    claim_scope_of(cl),
                    c.id,
                    c.strength,
                    scope_of(c.scope.as_deref(), c.bound),
                    verdict
                ));
            }
        }
    }

    s.push_str("\n## 检查\n\n| 检查 | 能力 | 档 | 范围 | 阶段 | 预算 | 出生证明 | 命令 |\n|---|---|---|---|---|---|---|---|\n");
    for c in &m.check {
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} | {}s | {} | `{}` |\n",
            c.id,
            c.capability,
            c.strength,
            scope_of(c.scope.as_deref(), c.bound),
            c.stage,
            c.budget_s.map(|b| b.to_string()).unwrap_or("—".into()),
            if c.red_command.is_some() {
                "🎓 可复现"
            } else if c.seen_red.is_some() {
                "⚠ 自述"
            } else {
                "❌ 无"
            },
            tail(&c.command, 50)
        ));
    }

    s.push_str("\n## 变更（改代码之前就该写下的东西）\n\n");
    if m.change.is_empty() {
        s.push_str("**无**。\n");
    } else {
        s.push_str("| 变更 | 意图 | 要求的检查 | 义务变化 | 声明变化 |\n|---|---|---|---|---|\n");
        for ch in &m.change {
            let req = checks_required_by(m, ch);
            s.push_str(&format!(
                "| {} | {} | {} | +{:?} −{:?} 触及{:?} | +{:?} −{:?} 触及{:?} |\n",
                ch.id,
                tail(&ch.intent, 40),
                if req.is_empty() {
                    "❌ 无（它凭什么算完成？）".to_string()
                } else {
                    req.iter()
                        .map(|c| c.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                ch.obligations_added,
                ch.obligations_removed,
                ch.obligations_touched,
                ch.claims_added,
                ch.claims_removed,
                ch.touches
            ));
        }
    }

    s.push_str("\n## 缺口清单（未绑定任何检查的声明）\n\n");
    if gaps.is_empty() {
        s.push_str("**无**。\n");
    } else {
        for g in &gaps {
            s.push_str(&format!(
                "- **{}** {}（要求 `{}` 档 {}）\n",
                g.id, g.text, g.capability, g.strength_required
            ));
        }
    }

    s.push_str("\n## 未充分判定的声明（有检查，但没有任何一条足以兑现）\n\n");
    let mut anyu = false;
    for cl in &m.claim {
        let mine: Vec<&Check> = m
            .check
            .iter()
            .filter(|c| c.claims.iter().any(|x| x == &cl.id))
            .collect();
        if mine.is_empty() || first_discharge(m, cl).is_some() {
            continue;
        }
        anyu = true;
        let why: Vec<String> = mine
            .iter()
            .filter_map(|c| {
                discharge(cl, c).err().map(|w| {
                    format!(
                        "{}（{} {}）：{}",
                        c.id,
                        c.strength,
                        scope_of(c.scope.as_deref(), c.bound),
                        w
                    )
                })
            })
            .collect();
        s.push_str(&format!(
            "- **{}**（要求 {} / {}）：{}\n",
            cl.id,
            cl.strength_required,
            claim_scope_of(cl),
            why.join("；")
        ));
    }
    if !anyu {
        s.push_str("**无**。\n");
    }

    s.push_str("\n## 信任假设（无法被机械检查的前提）\n\n");
    let mut any = false;
    for cl in &m.claim {
        if let Some(t) = &cl.trust_assumption {
            any = true;
            s.push_str(&format!("- **{}**：{}\n", cl.id, t));
        }
    }
    if !any {
        s.push_str("**无**。\n");
    }

    s.push_str("\n## 触发条件与签名（入场／退场；写下来才可被检查）\n\n");
    let mut anyt = false;
    for cl in &m.claim {
        if cl.trigger.is_some() || cl.owner.is_some() {
            anyt = true;
            s.push_str(&format!(
                "- **{}**{}{}\n",
                cl.id,
                cl.trigger
                    .as_ref()
                    .map(|t| format!(" 触发：{t}"))
                    .unwrap_or_default(),
                cl.owner
                    .as_ref()
                    .map(|o| format!(" ｜ 签名人：{o}"))
                    .unwrap_or_default()
            ));
        }
    }
    if !anyt {
        s.push_str("**无**。\n");
    }

    s.push_str("\n## unproven 检查（没有出生证明，不计入棘轮）\n\n");
    if unproven.is_empty() {
        s.push_str("**无**。\n");
    } else {
        for c in unproven {
            s.push_str(&format!(
                "- **{}**（{}）—— 需先故意违反一次并看到它红\n",
                c.id, c.command
            ));
        }
    }

    let selfreport: Vec<&Check> = m
        .check
        .iter()
        .filter(|c| c.red_command.is_none() && c.seen_red.is_some())
        .collect();
    s.push_str("\n## 仅自述的出生证明（引擎验证不了「我看过它红」这句话）\n\n");
    if selfreport.is_empty() {
        s.push_str("**无**。\n");
    } else {
        s.push_str("修法：给它一条 `red_command`（一条**期望失败**的命令），出生证明就从自述变成可复现。\n\n");
        for c in selfreport {
            s.push_str(&format!(
                "- **{}**：{}\n",
                c.id,
                tail(c.seen_red.as_deref().unwrap_or(""), 90)
            ));
        }
    }
    s
}

fn gaps(m: &Manifest) -> Vec<&Claim> {
    m.claim
        .iter()
        .filter(|cl| !m.check.iter().any(|c| c.claims.iter().any(|x| x == &cl.id)))
        .collect()
}

// ── run ─────────────────────────────────────────────────────────────────────

fn cmd_run(
    root: &Path,
    stage: Option<&str>,
    only: Option<&str>,
    change: Option<&str>,
    certify: bool,
) -> i32 {
    let m = match load_manifest(root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[清单错误] {e}");
            return 2;
        }
    };
    let ev_dir = assay_dir(root).join("evidence");
    if let Err(e) = fs::create_dir_all(&ev_dir) {
        eprintln!("[证据目录创建失败] {}: {e}", ev_dir.display());
        return 2;
    }
    let manifest_path = assay_dir(root).join("manifest.toml");
    let manifest_text = match fs::read_to_string(&manifest_path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("[清单指纹读取失败] {}: {e}", manifest_path.display());
            return 2;
        }
    };
    let manifest_fingerprint = fnv1a(&manifest_text);
    let source_revision = std::env::var("ASSAY_SOURCE_REVISION").ok();
    let run_id = new_run_id();

    let targets: Vec<&Check> = match change {
        Some(cid) => match m.change.iter().find(|c| c.id == cid) {
            None => {
                eprintln!("[未知变更] {cid}：清单里没有这条 [[change]]。");
                eprintln!("  变更必须在改代码之前写下——否则它就只能事后从 diff 里猜。");
                return 2;
            }
            Some(ch) => {
                let req = checks_required_by(&m, ch);
                if req.is_empty() {
                    eprintln!("[变更没有要求任何证据] {cid}：它凭什么算完成？");
                    return 2;
                }
                println!("变更 {cid}：{}", ch.intent);
                println!(
                    "  要求的检查（{} 条）：{}",
                    req.len(),
                    req.iter()
                        .map(|c| c.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                req
            }
        },
        None => m
            .check
            .iter()
            .filter(|c| stage.map(|s| c.stage == s).unwrap_or(true))
            .filter(|c| only.map(|o| c.id == o).unwrap_or(true))
            .collect(),
    };

    if targets.is_empty() {
        println!("[无匹配检查] stage={stage:?} check={only:?}");
        return 1;
    }

    let mut fails = 0;
    let mut passed: Vec<String> = Vec::new();
    let mut births: Vec<(String, Birth)> = Vec::new();
    let mut persisted = std::collections::BTreeSet::new();
    let mut verdicts: std::collections::BTreeMap<String, String> = Default::default();
    // 棘轮先读：出生证明只在「还没被复现过」或 `--certify` 时才真的重跑一次。
    // 理由：复现一次要编译一个做坏的引擎副本（秒级），没必要每跑必做；
    // 但**自述永远换不来这一位**——`certified` 只有机器见过那次失败才写得进去。
    let mut r = load_ratchet(root);
    for c in &targets {
        let started = iso8601_utc();
        let (verdict, code, detail, secs) = if let Some(why) = &c.skip_reason {
            ("skip".to_string(), None, format!("skip：{why}"), 0.0f64)
        } else {
            let (code, all, secs) = exec_shell(root, &c.command);
            let v = if code == 0 { "pass" } else { "fail" };
            (v.to_string(), Some(code), tail(&all, 2000), secs)
        };

        // 出生证明：一条**期望失败**的命令。它绿了，说明这条检查红不了。
        let prev_certified = r
            .entry
            .iter()
            .find(|e| e.id == c.id)
            .map(|e| e.certified)
            .unwrap_or(false);
        let need_cert = certify || !prev_certified;
        let (birth, birth_note) = match (&c.red_command, &c.seen_red) {
            (Some(rc), _) if need_cert => {
                let (rc_code, rc_out, rc_secs) = exec_shell(root, rc);
                if rc_code == 0 {
                    (
                        Birth::Missing,
                        format!("注入命令 exit=0（{rc_secs:.2}s）—— 它没能让检查红，证明不成立"),
                    )
                } else {
                    (
                        Birth::Reproduced,
                        format!(
                            "注入命令 exit={rc_code}（{rc_secs:.2}s）：{}",
                            tail(&rc_out, 160)
                        ),
                    )
                }
            }
            (Some(_), _) => (
                Birth::Reproduced,
                "已复现过（棘轮记录）；`run --certify` 可当场重跑".to_string(),
            ),
            (None, Some(s)) => (
                Birth::SelfReport,
                format!("自述（引擎无法验证）：{}", tail(s, 160)),
            ),
            (None, None) => (Birth::Missing, "无".to_string()),
        };
        births.push((c.id.clone(), birth));
        verdicts.insert(c.id.clone(), verdict.clone());

        let linked_claims: Vec<&Claim> = c
            .claims
            .iter()
            .filter_map(|claim_id| m.claim.iter().find(|claim| claim.id == *claim_id))
            .collect();
        let mut obligation_ids: Vec<String> = linked_claims
            .iter()
            .flat_map(|claim| claim.obligations.iter().cloned())
            .collect();
        obligation_ids.sort();
        obligation_ids.dedup();
        let mut origins: Vec<String> = linked_claims
            .iter()
            .filter_map(|claim| claim.origin.clone())
            .collect();
        origins.sort();
        origins.dedup();
        let mut assumption_ids: Vec<String> = linked_claims
            .iter()
            .flat_map(|claim| claim.assumption_ids.iter().cloned())
            .collect();
        assumption_ids.sort();
        assumption_ids.dedup();
        let mut trust_assumptions: Vec<String> = linked_claims
            .iter()
            .filter_map(|claim| claim.trust_assumption.clone())
            .collect();
        trust_assumptions.sort();
        trust_assumptions.dedup();
        let mut claim_ids = c.claims.clone();
        claim_ids.sort();
        claim_ids.dedup();
        let rec = EvidenceRecord {
            schema_version: "assay-evidence/v1",
            event_id: format!("{run_id}:{}", c.id),
            run_id: run_id.clone(),
            lineage: EvidenceLineage {
                change_id: change.map(str::to_string),
                claim_ids,
                obligation_ids,
                origins,
                assumption_ids,
                trust_assumptions,
            },
            subject: EvidenceSubject {
                revision: source_revision.clone(),
                manifest_fingerprint: manifest_fingerprint.clone(),
            },
            checker: EvidenceChecker {
                id: c.id.clone(),
                capability: c.capability.clone(),
                strength: c.strength,
                scope: c.scope.clone(),
                bound: c.bound,
                stage: c.stage.clone(),
                command_fingerprint: fnv1a(&c.command),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                checker_version: c.checker_version.clone(),
            },
            result: EvidenceResult {
                verdict: verdict.clone(),
                exit_code: code,
                started_at: started,
                duration_s: (secs * 100.0).round() / 100.0,
                detail: detail.clone(),
                counterexample: None,
            },
            birth_certificate: BirthCertificate {
                status: birth_code(birth).to_string(),
                note: birth_note.clone(),
            },
        };
        let jp = ev_dir.join(format!("{}.json", c.id));
        let lp = ev_dir.join(format!("{}.jsonl", c.id));
        match serde_json::to_string(&rec) {
            Ok(line) => match append_jsonl(&lp, &line) {
                Ok(()) => {
                    persisted.insert(c.id.clone());
                    match serde_json::to_string_pretty(&rec) {
                        Ok(latest) => {
                            if let Err(e) = fs::write(&jp, latest) {
                                eprintln!("[最新证据视图写入失败] {}: {e}", jp.display());
                                fails += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("[最新证据视图序列化失败] {}: {e}", c.id);
                            fails += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[证据历史写入失败] {}: {e}", lp.display());
                    fails += 1;
                }
            },
            Err(e) => {
                eprintln!("[证据序列化失败] {}: {e}", c.id);
                fails += 1;
            }
        }

        if verdict == "pass" {
            passed.push(c.id.clone());
        }

        let mark = match verdict.as_str() {
            "pass" => "[PASS]",
            "fail" => "[FAIL]",
            _ => "[SKIP]",
        };
        let bmark = match birth {
            Birth::Reproduced => "🎓",
            Birth::SelfReport => "⚠",
            Birth::Missing => "❌",
        };
        if verdict == "fail" {
            fails += 1;
            println!("{mark} {} {bmark} — exit={code:?} {secs:.2}s", c.id);
            println!("       最小反例/指针（末尾 12 行）：");
            for l in detail
                .lines()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .iter()
                .rev()
            {
                println!("       {l}");
            }
        } else {
            let first = detail.lines().next().unwrap_or("");
            println!("{mark} {} {bmark} — {secs:.2}s  {}", c.id, tail(first, 80));
        }
        if birth != Birth::Reproduced {
            println!("       出生证明 {}：{birth_note}", birth_rank(birth));
        }
    }

    // ── 分歧（协议 v0.4 §7）────────────────────────────────────────────────
    // 同一条声明上，不同的检查给出不同判定。它只把**清单里已有的绑定**当作边，不引入新理论。
    // 实测摩擦：同一个不变量在两个后端各写一遍，互不知情，没有东西检查它们是否在说同一件事。
    // 一旦分歧，**绿的那条不得被读成"这条声明成立"**。
    if only.is_none() {
        let mut any = false;
        for cl in &m.claim {
            let mine: Vec<&Check> = targets
                .iter()
                .filter(|c| c.claims.iter().any(|x| x == &cl.id))
                .copied()
                .collect();
            if mine.len() < 2 {
                continue;
            }
            let p: Vec<&str> = mine
                .iter()
                .filter(|c| verdicts.get(&c.id).map(|v| v == "pass").unwrap_or(false))
                .map(|c| c.id.as_str())
                .collect();
            let f: Vec<&str> = mine
                .iter()
                .filter(|c| verdicts.get(&c.id).map(|v| v == "fail").unwrap_or(false))
                .map(|c| c.id.as_str())
                .collect();
            if !p.is_empty() && !f.is_empty() {
                if !any {
                    println!("\n⚔ 分歧：同一条声明上的检查给出了不同判定");
                    any = true;
                }
                println!(
                    "   {} — 绿 {p:?} vs 红 {f:?}：**绿的那条不得被读成「这条声明成立」**",
                    cl.id
                );
            }
        }
        if any {
            println!("   （分歧不是新机制：它只是把清单里已有的绑定当作边。要不要就此加一条新声明，由人决定。）");
        }
    }

    // 棘轮：只有「过了 + 有出生证明」的检查才计入，且只增不减
    let mut added = 0;
    for c in &targets {
        // 兑现 = 这一跑绿过 且 有出生证明；两条缺一不进棘轮
        let b = births
            .iter()
            .find(|(id, _)| id == &c.id)
            .map(|(_, b)| *b)
            .unwrap_or(Birth::Missing);
        if b != Birth::Missing
            && passed.iter().any(|p| p == &c.id)
            && persisted.contains(&c.id)
            && !r.entry.iter().any(|e| e.id == c.id)
        {
            r.entry.push(entry_of(&m, c, b == Birth::Reproduced));
            added += 1;
        }
    }
    if added > 0 {
        if let Err(e) = save_ratchet(root, &r) {
            eprintln!("[棘轮写入失败] {e}");
        } else {
            println!(
                "\n棘轮 +{added} ⇒ 高度 {}（兑现 = 绿过 + 有出生证明）",
                r.entry.len()
            );
        }
    }

    // 棘轮会改变地图内容：跑完即重生成，消掉"地图漂移陷阱"
    let r2 = load_ratchet(root);
    let _ = fs::write(assay_dir(root).join("MAP.md"), map_content(&m, &r2));

    if fails > 0 {
        1
    } else {
        0
    }
}

// ── map ─────────────────────────────────────────────────────────────────────

fn cmd_map(root: &Path) -> i32 {
    let m = match load_manifest(root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[清单错误] {e}");
            return 2;
        }
    };
    let r = load_ratchet(root);
    let p = assay_dir(root).join("MAP.md");
    match fs::write(&p, map_content(&m, &r)) {
        Ok(_) => {
            println!("已生成 {}", p.display());
            0
        }
        Err(e) => {
            eprintln!("[写地图失败] {e}");
            2
        }
    }
}

// ── check（清单自检 + 地图漂移 + 棘轮） ─────────────────────────────────────

fn expect(cond: bool, fail: &mut i32, msg: String) {
    if cond {
        println!("  ✅ {msg}");
    } else {
        println!("  ❌ {msg}");
        *fail = 1;
    }
}

fn cmd_check(root: &Path) -> i32 {
    let m = match load_manifest(root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[清单错误] {e}");
            return 2;
        }
    };
    let mut fail = 0;

    let proto = m
        .assay
        .as_ref()
        .and_then(|a| a.protocol.clone())
        .unwrap_or_else(|| "未声明".into());
    println!("== 清单自检（协议 {proto}） ==");
    if proto.starts_with("v0.6") {
        let bad_sources: Vec<&str> = m
            .claim
            .iter()
            .filter(|claim| !claim_has_exactly_one_source(claim))
            .map(|claim| claim.id.as_str())
            .collect();
        expect(
            bad_sources.is_empty(),
            &mut fail,
            format!(
                "每条正式化主张恰有一个来源（义务引用或 origin）（缺失/冲突：{bad_sources:?}）"
            ),
        );
        let duplicate_links: Vec<String> = m
            .claim
            .iter()
            .filter(|claim| {
                let mut unique = std::collections::BTreeSet::new();
                claim.obligations.iter().any(|id| !unique.insert(id))
            })
            .map(|claim| claim.id.clone())
            .collect();
        expect(
            duplicate_links.is_empty(),
            &mut fail,
            format!("主张义务引用无重复（重复：{duplicate_links:?}）"),
        );
    }
    let mut ids = std::collections::BTreeSet::new();
    let dup: Vec<&str> = m
        .check
        .iter()
        .map(|c| c.id.as_str())
        .filter(|id| !ids.insert(*id))
        .collect();
    expect(
        dup.is_empty(),
        &mut fail,
        format!("检查 id 唯一（重复：{dup:?}）"),
    );
    let mut cids = std::collections::BTreeSet::new();
    let cdup: Vec<&str> = m
        .claim
        .iter()
        .map(|c| c.id.as_str())
        .filter(|id| !cids.insert(*id))
        .collect();
    expect(
        cdup.is_empty(),
        &mut fail,
        format!("声明 id 唯一（重复：{cdup:?}）"),
    );

    let unknown: Vec<&str> = m
        .check
        .iter()
        .flat_map(|c| c.claims.iter().map(|x| x.as_str()))
        .filter(|x| !m.claim.iter().any(|cl| cl.id == *x))
        .collect();
    expect(
        unknown.is_empty(),
        &mut fail,
        format!("检查引用的声明都存在（悬空：{unknown:?}）"),
    );

    let bad_rej: Vec<&str> = m
        .claim
        .iter()
        .filter(|cl| cl.rejected_by.trim().is_empty())
        .map(|cl| cl.id.as_str())
        .collect();
    expect(
        bad_rej.is_empty(),
        &mut fail,
        format!("每条声明都写了 rejected_by（缺：{bad_rej:?}）"),
    );

    let strength4: Vec<&str> = m
        .check
        .iter()
        .filter(|c| c.strength == 4)
        .map(|c| c.id.as_str())
        .collect();
    expect(
        strength4.is_empty(),
        &mut fail,
        format!("没有检查登记在档 4（调试期断言）（违反：{strength4:?}）"),
    );

    println!("\n== 覆盖 ==");
    let g = gaps(&m);
    expect(
        g.is_empty(),
        &mut fail,
        format!("没有未绑定检查的声明（缺口 {}）", g.len()),
    );
    for x in &g {
        println!("     缺口：{} {}", x.id, tail(&x.text, 50));
    }
    let under: Vec<String> = m
        .claim
        .iter()
        .filter(|cl| {
            !m.check.iter().any(|c| c.claims.iter().any(|x| x == &cl.id))
                || first_discharge(&m, cl).is_none()
        })
        .filter(|cl| m.check.iter().any(|c| c.claims.iter().any(|x| x == &cl.id)))
        .map(|cl| {
            let why: Vec<String> = m
                .check
                .iter()
                .filter(|c| c.claims.iter().any(|x| x == &cl.id))
                .filter_map(|c| {
                    discharge(cl, c).err().map(|w| {
                        format!(
                            "{}（{} {}）：{}",
                            c.id,
                            c.strength,
                            scope_of(c.scope.as_deref(), c.bound),
                            w
                        )
                    })
                })
                .collect();
            format!(
                "{}（要求 {} / {}）无任何检查足以兑现 —— {}",
                cl.id,
                cl.strength_required,
                claim_scope_of(cl),
                why.join("；")
            )
        })
        .collect();
    expect(
        under.is_empty(),
        &mut fail,
        format!("没有未充分判定的声明（{}）", under.join(" ｜ ")),
    );

    println!("\n== 出生证明 ==");
    let repro: Vec<&str> = m
        .check
        .iter()
        .filter(|c| c.red_command.is_some() && c.skip_reason.is_none())
        .map(|c| c.id.as_str())
        .collect();
    let selfrep: Vec<&str> = m
        .check
        .iter()
        .filter(|c| c.red_command.is_none() && c.seen_red.is_some() && c.skip_reason.is_none())
        .map(|c| c.id.as_str())
        .collect();
    let unp: Vec<&str> = m
        .check
        .iter()
        .filter(|c| c.seen_red.is_none() && c.red_command.is_none() && c.skip_reason.is_none())
        .map(|c| c.id.as_str())
        .collect();
    println!(
        "  🎓 可复现（有 `red_command`，机器能证明它红得了）：{} 条",
        repro.len()
    );
    if selfrep.is_empty() {
        println!("  ✅ 没有仅靠自述的出生证明");
    } else {
        println!(
            "  ⚠ 仅自述（引擎验证不了「我看过它红」）：{} 条 {selfrep:?}",
            selfrep.len()
        );
        println!("     这是本协议最大的自觉缺口：§5 号称最重要，却曾是唯一完全靠自报的一条。");
    }
    if unp.is_empty() {
        println!("  ✅ 没有缺失出生证明的检查");
    } else {
        println!("  ❌ 缺失出生证明（`unproven`，不计入棘轮）：{unp:?}");
    }

    println!("\n== 变更记录 ==");
    if m.change.is_empty() {
        println!("  （无）—— 若靠 diff 事后推断变化，就没有东西能说「这次变化欠了哪些证据」");
    } else {
        let mut chids = std::collections::BTreeSet::new();
        let chdup: Vec<&str> = m
            .change
            .iter()
            .map(|c| c.id.as_str())
            .filter(|id| !chids.insert(*id))
            .collect();
        expect(
            chdup.is_empty(),
            &mut fail,
            format!("变更 id 唯一（重复：{chdup:?}）"),
        );
        let mut bad: Vec<String> = Vec::new();
        let known_obligations: std::collections::BTreeSet<&str> = m
            .claim
            .iter()
            .flat_map(|claim| claim.obligations.iter().map(String::as_str))
            .collect();
        for ch in &m.change {
            for x in ch
                .claims_added
                .iter()
                .chain(ch.claims_removed.iter())
                .chain(ch.touches.iter())
            {
                if !m.claim.iter().any(|cl| &cl.id == x) {
                    bad.push(format!("{}→不存在的声明 {}", ch.id, x));
                }
            }
            for x in &ch.evidence_required {
                if !m.check.iter().any(|c| &c.id == x) {
                    bad.push(format!("{}→不存在的检查 {}", ch.id, x));
                }
            }
            for x in ch
                .obligations_added
                .iter()
                .chain(ch.obligations_touched.iter())
            {
                if !known_obligations.contains(x.as_str()) {
                    bad.push(format!("{}→不存在的义务 {}", ch.id, x));
                }
            }
            if !ch.claims_removed.is_empty()
                && (ch.reason.trim().is_empty() || ch.signed.trim().is_empty())
            {
                bad.push(format!(
                    "{}→注销了声明却没有 reason/signed（削弱必须签字）",
                    ch.id
                ));
            }
            if !ch.obligations_removed.is_empty()
                && (ch.reason.trim().is_empty() || ch.signed.trim().is_empty())
            {
                bad.push(format!(
                    "{}→注销了义务却没有 reason/signed（削弱必须签字）",
                    ch.id
                ));
            }
            if checks_required_by(&m, ch).is_empty() {
                bad.push(format!("{}→没有要求任何证据（它凭什么算完成？）", ch.id));
            }
        }
        expect(
            bad.is_empty(),
            &mut fail,
            format!("变更记录自洽（问题：{}）", bad.join("、")),
        );
    }

    println!("\n== 棘轮 ==");
    let r = load_ratchet(root);
    let waivers: std::collections::BTreeSet<&str> =
        m.waiver.iter().map(|w| w.check.as_str()).collect();
    let wv = |id: &str| m.waiver.iter().find(|w| w.check == id);
    let mut weakened = 0;
    for e in &r.entry {
        match m.check.iter().find(|c| c.id == e.id) {
            None => {
                weakened += 1;
                let ok = waivers.contains(e.id.as_str());
                println!(
                    "  {} 检查 {} 已从清单消失{}",
                    if ok { "⚠" } else { "❌" },
                    e.id,
                    match wv(&e.id) {
                        Some(w) => format!(
                            "（waiver：{}｜签名 {}；仅豁免棘轮这一条）",
                            w.reason, w.signed
                        ),
                        None => "（无 waiver）".to_string(),
                    }
                );
                if !ok {
                    fail = 1;
                }
            }
            Some(c) => {
                let cur = entry_of(&m, c, e.certified);
                let mut reasons = Vec::new();
                if c.strength < e.strength {
                    reasons.push(format!("强度 {}→{}", e.strength, c.strength));
                }
                // 只有"范围变弱"才算削弱；补登 unbounded、扩大界都属增强，不该要豁免
                if scope_strength(c.scope.as_deref(), c.bound)
                    < scope_strength(e.scope.as_deref(), e.bound)
                {
                    reasons.push(format!(
                        "范围削弱 {} → {}",
                        scope_of(e.scope.as_deref(), e.bound),
                        scope_of(c.scope.as_deref(), c.bound)
                    ));
                }
                if cur.cmd_fp != e.cmd_fp {
                    reasons.push("命令指纹变化".to_string());
                }
                let lost: Vec<String> = e
                    .claims
                    .iter()
                    .filter(|x| !cur.claims.contains(*x))
                    .cloned()
                    .collect();
                if !lost.is_empty() {
                    reasons.push(format!("解绑声明 {lost:?}"));
                }
                let lost_obligations: Vec<String> = e
                    .obligations
                    .iter()
                    .filter(|id| !cur.obligations.contains(*id))
                    .cloned()
                    .collect();
                if !lost_obligations.is_empty() {
                    reasons.push(format!("解绑义务 {lost_obligations:?}"));
                }
                if !reasons.is_empty() {
                    weakened += 1;
                    let ok = waivers.contains(e.id.as_str());
                    println!(
                        "  {} {} （{}）{}",
                        if ok { "⚠" } else { "❌" },
                        c.id,
                        reasons.join("、"),
                        match wv(&c.id) {
                            Some(w) => format!(
                                "有 waiver：{}｜签名 {}（**仅豁免棘轮这一条**；覆盖缺口、出生证明等红灯需另行处理）",
                                w.reason, w.signed
                            ),
                            None => "无 waiver".to_string(),
                        }
                    );
                    if !ok {
                        fail = 1;
                    }
                }
            }
        }
    }
    if weakened == 0 {
        println!("  ✅ 无削弱（棘轮高度 {}）", r.entry.len());
    }
    let newc = m
        .check
        .iter()
        .filter(|c| !r.entry.iter().any(|e| e.id == c.id))
        .count();
    println!("  待纳入棘轮：{newc}（跑 `assay run` 后自动纳入，前提是有出生证明）");

    println!("\n== 地图新鲜度 ==");
    let p = assay_dir(root).join("MAP.md");
    let want = map_content(&m, &r);
    match fs::read_to_string(&p) {
        Ok(have) => {
            if have == want {
                println!("  ✅ MAP.md 与清单一致");
            } else {
                println!("  ❌ MAP.md 已漂移 —— 跑 `assay map` 重新生成");
                fail = 1;
            }
        }
        Err(_) => {
            println!("  ❌ MAP.md 不存在 —— 跑 `assay map`");
            fail = 1;
        }
    }

    if fail == 0 {
        println!("\n== 结论：OK ==");
    } else {
        println!("\n== 结论：FAIL ==");
    }
    fail
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut sub = String::new();
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut stage: Option<String> = None;
    let mut only: Option<String> = None;
    let mut change: Option<String> = None;
    let mut certify = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                i += 1;
                root = PathBuf::from(args.get(i).cloned().unwrap_or_default());
            }
            "--stage" => {
                i += 1;
                stage = args.get(i).cloned();
            }
            "--check" => {
                i += 1;
                only = args.get(i).cloned();
            }
            "--change" => {
                i += 1;
                change = args.get(i).cloned();
            }
            "--certify" => certify = true,
            other => {
                if sub.is_empty() {
                    sub = other.to_string();
                }
            }
        }
        i += 1;
    }

    let code = match sub.as_str() {
        "run" => cmd_run(
            &root,
            stage.as_deref(),
            only.as_deref(),
            change.as_deref(),
            certify,
        ),
        "map" => cmd_map(&root),
        "check" => cmd_check(&root),
        "" => {
            eprintln!("usage: assay <run|map|check> [--root <path>] [--stage <s>] [--check <id>] [--change <id>] [--certify]");
            2
        }
        other => {
            eprintln!("未知子命令 {other:?}；可用：run / map / check");
            2
        }
    };
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_has_exactly_one_nonempty_source() {
        let mut claim = Claim::default();
        assert!(!claim_has_exactly_one_source(&claim));

        claim.obligations.push("O-001".to_string());
        assert!(claim_has_exactly_one_source(&claim));

        claim.origin = Some("assay-protocol".to_string());
        assert!(!claim_has_exactly_one_source(&claim));

        claim.obligations.clear();
        assert!(claim_has_exactly_one_source(&claim));

        claim.origin = Some("  ".to_string());
        assert!(!claim_has_exactly_one_source(&claim));
    }

    #[test]
    fn jsonl_append_preserves_records_on_separate_lines() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "assay-evidence-{}-{nanos}.jsonl",
            std::process::id()
        ));

        append_jsonl(&path, r#"{"run":1}"#).unwrap();
        append_jsonl(&path, r#"{"run":2}"#).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let records: Vec<serde_json::Value> = contents
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["run"], 1);
        assert_eq!(records[1]["run"], 2);
        drop(contents);
        fs::remove_file(path).unwrap();
    }
}
