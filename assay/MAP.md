# MAP

> **机械生成**（assay engine）—— 禁止手改；漂移即红灯（`assay check`）。

## 摘要

| 项 | 值 |
|---|---|
| 声明 | 9 |
| 检查 | 9 |
| 能力分布 | `self-test` × 7、`structural` × 2 |
| 强度分布 | 档 5 × 9 |
| **棘轮高度** | **9**（已兑现且带出生证明） |
| `unproven` 检查 | 0 |
| 出生证明 | 可复现 **9** ／ 自述 **0** ／ 缺失 **0** |
| 变更记录 | 2 |
| 未绑定任何检查的声明 | 0 |

## 声明 → 来源 → 检查

| 声明 | 来源 | 要求能力 | 要求档 | 要求范围 | 判定它的检查 | 检查档 | 检查范围 | 是否充分 |
|---|---|---|---|---|---|---|---|---|
| C-001 协议文档只描述能力，不点名任何判定后端 | origin: assay-protocol | structural | 5 | unbounded | S-01 | 5 | unbounded | ✅ |
| C-002 引擎源码不认识任何判定后端（哑执行器的边界） | origin: assay-protocol | structural | 5 | unbounded | S-02 | 5 | unbounded | ✅ |
| C-003 引擎对不认识的输入非零退出，且不产出任何判定 | origin: assay-protocol | self-test | 5 | unbounded | S-03 | 5 | unbounded | ✅ |
| C-004 登记在档 4（调试期断言）的检查被清单自检拒绝 | origin: assay-protocol | self-test | 5 | unbounded | S-04 | 5 | unbounded | ✅ |
| C-005 缺出生证明的检查被单列，且不计入棘轮 | origin: assay-protocol | self-test | 5 | unbounded | S-05 | 5 | unbounded | ✅ |
| C-006 已入棘轮的检查不能被静默削弱 | origin: assay-protocol | self-test | 5 | unbounded | S-06 | 5 | unbounded | ✅ |
| C-007 手改地图即红灯 | origin: assay-protocol | self-test | 5 | unbounded | S-07 | 5 | unbounded | ✅ |
| C-008 无界声明不能被有界检查兑现（v0.2 覆盖范围规则） | origin: assay-protocol | self-test | 5 | unbounded | S-08 | 5 | unbounded | ✅ |
| C-009 同一条声明上出现分歧时，绿的那条不得被读成「该声明成立」 | origin: assay-protocol | self-test | 5 | unbounded | S-09 | 5 | unbounded | ✅ |

## 检查

| 检查 | 能力 | 档 | 范围 | 阶段 | 预算 | 出生证明 | 命令 |
|---|---|---|---|---|---|---|---|
| S-01 | structural | 5 | unbounded | pre-commit | 30s | 🎓 可复现 | `ssay\selfcheck\S-01-protocol-has-no-tool-names.ps1` |
| S-02 | structural | 5 | unbounded | pre-commit | 30s | 🎓 可复现 | `ile assay\selfcheck\S-02-engine-knows-no-tools.ps1` |
| S-03 | self-test | 5 | unbounded | pre-commit | 180s | 🎓 可复现 | `assay\selfcheck\S-03-unknown-subcommand-is-red.ps1` |
| S-04 | self-test | 5 | unbounded | pr | 180s | 🎓 可复现 | `ile assay\selfcheck\S-04-strength4-is-rejected.ps1` |
| S-05 | self-test | 5 | unbounded | pr | 180s | 🎓 可复现 | `assay\selfcheck\S-05-missing-birth-certificate.ps1` |
| S-06 | self-test | 5 | unbounded | pr | 180s | 🎓 可复现 | `ssay\selfcheck\S-06-ratchet-cannot-be-weakened.ps1` |
| S-07 | self-test | 5 | unbounded | pr | 180s | 🎓 可复现 | `le -File assay\selfcheck\S-07-map-drift-is-red.ps1` |
| S-08 | self-test | 5 | unbounded | pr | 180s | 🎓 可复现 | `le -File assay\selfcheck\S-08-scope-rule-holds.ps1` |
| S-09 | self-test | 5 | unbounded | pr | 180s | 🎓 可复现 | `ile assay\selfcheck\S-09-contested-is-reported.ps1` |

## 变更（改代码之前就该写下的东西）

| 变更 | 意图 | 要求的检查 | 义务变化 | 声明变化 |
|---|---|---|---|---|
| CH-001 | say 自己写清单，并把出生证明从自述变成可复现（新增 red_command） | S-01, S-02, S-03, S-04, S-05, S-06, S-07, S-08 | +[] −[] 触及[] | +["C-001", "C-002", "C-003", "C-004", "C-005", "C-006", "C-007", "C-008"] −[] 触及[] |
| CH-002 | 新增「分歧」规则（引擎报出同一声明上互相矛盾的判定），并给它配一条能红的检查 | S-01, S-09 | +[] −[] 触及[] | +["C-009"] −[] 触及["C-001"] |

## 缺口清单（未绑定任何检查的声明）

**无**。

## 未充分判定的声明（有检查，但没有任何一条足以兑现）

**无**。

## 信任假设（无法被机械检查的前提）

- **C-001**：S-01 用的是手写词表（后端工具名）。它证不了协议里没有别的领域名词，也管不住 `verify` 这类通用子命令名。承担者：人（读 PROTOCOL.md 时发现即补词表）
- **C-002**：S-02 是文本检查：证明源码里没有工具名，不能证明引擎行为上不依赖某个工具。承担者：人（复审唯一构造 Command 的那一处 exec_shell）
- **C-009**：引擎只能报出「清单里已有的绑定」上的分歧。它不知道两个检查是否在判同一件事——那是人的判断。承担者：清单作者

## 触发条件与签名（入场／退场；写下来才可被检查）

**无**。

## unproven 检查（没有出生证明，不计入棘轮）

**无**。

## 仅自述的出生证明（引擎验证不了「我看过它红」这句话）

**无**。
