# MAP

> **机械生成**（assay engine）—— 禁止手改；漂移即红灯（`assay check`）。

## 摘要

| 项 | 值 |
|---|---|
| 声明 | 3 |
| 检查 | 3 |
| 能力分布 | `coverage-search` × 3 |
| 强度分布 | 档 2 × 3 |
| **棘轮高度** | **2**（已兑现且带出生证明） |
| `unproven` 检查 | 0 |
| 出生证明 | 可复现 **3** ／ 自述 **0** ／ 缺失 **0** |
| 变更记录 | 0 |
| 未绑定任何检查的声明 | 0 |

## 声明 → 来源 → 检查

| 声明 | 来源 | 要求能力 | 要求档 | 要求范围 | 判定它的检查 | 检查档 | 检查范围 | 是否充分 |
|---|---|---|---|---|---|---|---|---|
| C-001 256 counts match oracle | O-040 | coverage-search | 2 | sampled | K-001 | 2 | sampled | ✅ |
| C-002 rk drains after producer close | O-044 | coverage-search | 2 | sampled | K-002 | 2 | sampled | ✅ |
| C-003 put matches the byte reference | O-045, O-046 | coverage-search | 2 | sampled | K-003 | 2 | sampled | ✅ |

## 检查

| 检查 | 能力 | 档 | 范围 | 输入范围 | 阶段 | 预算 | 出生证明 | 命令 |
|---|---|---|---|---|---|---|---|---|---|
| K-001 | coverage-search | 2 | sampled | src/, Cargo.toml, Cargo.lock, assay/manifest.toml | pre-commit | 120s | 🎓 可复现 | `cargo test --locked --manifest-path Cargo.toml` |
| K-002 | coverage-search | 2 | sampled | tla/, assay/manifest.toml | pr | 120s | 🎓 可复现 | `python tla/check.py` |
| K-003 | coverage-search | 2 | sampled | src/, tests/, Cargo.toml, Cargo.lock, assay/manifest.toml | pre-commit | 120s | 🎓 可复现 | `cargo test --locked --manifest-path Cargo.toml` |

## 变更（改代码之前就该写下的东西）

**无**。

## 缺口清单（未绑定任何检查的声明）

**无**。

## 未充分判定的声明（有检查，但没有任何一条足以兑现）

**无**。

## 信任假设（无法被机械检查的前提）

**无**。

## 触发条件与签名（入场／退场；写下来才可被检查）

**无**。

## unproven 检查（没有出生证明，不计入棘轮）

**无**。

## 仅自述的出生证明（引擎验证不了「我看过它红」这句话）

**无**。
