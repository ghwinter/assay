# MAP

> **机械生成**（assay engine）—— 禁止手改；漂移即红灯（`assay check`）。

## 摘要

| 项 | 值 |
|---|---|
| 声明 | 13 |
| 检查 | 13 |
| 能力分布 | `self-test` × 7、`structural` × 6 |
| 强度分布 | 档 5 × 13 |
| **棘轮高度** | **13**（已兑现且带出生证明） |
| `unproven` 检查 | 0 |
| 出生证明 | 可复现 **13** ／ 自述 **0** ／ 缺失 **0** |
| 变更记录 | 4 |
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
| C-010 首词解析不到时，doctor 报出并点名是哪个检查的哪个字段 | O-048 | structural | 5 | unbounded | S-10 | 5 | unbounded | ✅ |
| C-011 定义不符时，next 报出并给出具体原因；定义未变时不得误报 | O-049 | structural | 5 | unbounded | S-11 | 5 | unbounded | ✅ |
| C-012 it 定义：未忽略的新文件在作用域内，被忽略的文件移出作用域 | O-050 | structural | 5 | unbounded | S-12 | 5 | unbounded | ✅ |
| C-013 ENTS.md 有机器判定的字符预算，且不驻留机器相关的命令 | O-052 | structural | 5 | unbounded | S-13 | 5 | unbounded | ✅ |

## 检查

| 检查 | 能力 | 档 | 范围 | 输入范围 | 阶段 | 预算 | 出生证明 | 命令 |
|---|---|---|---|---|---|---|---|---|---|
| S-01 | structural | 5 | unbounded | — | pre-commit | 30s | 🎓 可复现 | `ssay\selfcheck\S-01-protocol-has-no-tool-names.ps1` |
| S-02 | structural | 5 | unbounded | — | pre-commit | 30s | 🎓 可复现 | `ile assay\selfcheck\S-02-engine-knows-no-tools.ps1` |
| S-03 | self-test | 5 | unbounded | — | pre-commit | 180s | 🎓 可复现 | `assay\selfcheck\S-03-unknown-subcommand-is-red.ps1` |
| S-04 | self-test | 5 | unbounded | — | pr | 180s | 🎓 可复现 | `ile assay\selfcheck\S-04-strength4-is-rejected.ps1` |
| S-05 | self-test | 5 | unbounded | — | pr | 180s | 🎓 可复现 | `assay\selfcheck\S-05-missing-birth-certificate.ps1` |
| S-06 | self-test | 5 | unbounded | — | pr | 180s | 🎓 可复现 | `ssay\selfcheck\S-06-ratchet-cannot-be-weakened.ps1` |
| S-07 | self-test | 5 | unbounded | — | pr | 180s | 🎓 可复现 | `le -File assay\selfcheck\S-07-map-drift-is-red.ps1` |
| S-08 | self-test | 5 | unbounded | — | pr | 180s | 🎓 可复现 | `le -File assay\selfcheck\S-08-scope-rule-holds.ps1` |
| S-09 | self-test | 5 | unbounded | — | pr | 180s | 🎓 可复现 | `ile assay\selfcheck\S-09-contested-is-reported.ps1` |
| S-10 | structural | 5 | unbounded | engine/src/readiness.rs, engine/src/cli.rs, assay/selfcheck/ | pre-commit | 60s | 🎓 可复现 | `say\selfcheck\S-10-unresolvable-command-is-red.ps1` |
| S-11 | structural | 5 | unbounded | engine/src/readiness.rs, assay/selfcheck/ | pr | 120s | 🎓 可复现 | `y\selfcheck\S-11-next-detects-definition-drift.ps1` |
| S-12 | structural | 5 | unbounded | tools/ledger_lint.py, tools/assay-cli.ps1, assay/selfcheck/ | pre-commit | 120s | 🎓 可复现 | `File assay\selfcheck\S-12-l1-scope-follows-git.ps1` |
| S-13 | structural | 5 | unbounded | AGENTS.md, RUNBOOK.md, assay/selfcheck/ | pre-commit | 30s | 🎓 可复现 | `e assay\selfcheck\S-13-injected-doc-has-budget.ps1` |

## 变更（改代码之前就该写下的东西）

| 变更 | 意图 | 要求的检查 | 义务变化 | 声明变化 |
|---|---|---|---|---|
| CH-001 | say 自己写清单，并把出生证明从自述变成可复现（新增 red_command） | S-01, S-02, S-03, S-04, S-05, S-06, S-07, S-08 | +[] −[] 触及[] | +["C-001", "C-002", "C-003", "C-004", "C-005", "C-006", "C-007", "C-008"] −[] 触及[] |
| CH-002 | 新增「分歧」规则（引擎报出同一声明上互相矛盾的判定），并给它配一条能红的检查 | S-01, S-09 | +[] −[] 触及[] | +["C-009"] −[] 触及["C-001"] |
| CH-003 | ctor（就绪度）+ next（有序动作表）+ 启动器；并修 L1 的作用域判定 | S-01, S-02, S-10, S-11, S-12 | +["O-048", "O-049", "O-050"] −[] 触及[] | +["C-010", "C-011", "C-012"] −[] 触及["C-001", "C-002"] |
| CH-004 | ENTS.md 搬进按需加载的 RUNBOOK.md，并给它装一条机器判定的预算 | S-01, S-13 | +["O-052"] −[] 触及[] | +["C-013"] −[] 触及["C-001"] |

## 缺口清单（未绑定任何检查的声明）

**无**。

## 未充分判定的声明（有检查，但没有任何一条足以兑现）

**无**。

## 信任假设（无法被机械检查的前提）

- **C-001**：S-01 用的是手写词表（后端工具名）。它证不了协议里没有别的领域名词，也管不住 `verify` 这类通用子命令名。承担者：人（读 PROTOCOL.md 时发现即补词表）
- **C-002**：S-02 是文本检查：证明源码里没有工具名，不能证明引擎行为上不依赖某个工具。承担者：人（复审唯一构造 Command 的那一处 exec_shell）
- **C-009**：引擎只能报出「清单里已有的绑定」上的分歧。它不知道两个检查是否在判同一件事——那是人的判断。承担者：清单作者
- **C-010**：doctor 只查首词能否在 PATH 或仓库内解析到，**不执行**任何被声明的命令。因此它证不了'命令能跑通'，只证得了'命令找得到'。承担者：清单作者（首次 run 时确认）
- **C-011**：next 判的是'证据对不对得上这条检查的当前定义'（命令/档/范围/绑定的声明），不判'结论还成不成立'。语义是否仍适用只有人能判。承担者：清单作者
- **C-012**：本检查只覆盖 `tools/**/*.py` 这一条 glob 路径，作为 CODE_PATTERNS 全集的代表。其余模式的同类回归要靠同一条实现共享，不由本检查逐条证明。承担者：ledger_lint 维护者
- **C-013**：本检查只判两件事：字符数与命令头。它判不了'这行内容值不值它的上下文成本'——那是编辑判断。承担者：AGENTS.md 的维护者；提额必须在同一次改动的 [[change]].reason 里写理由

## 触发条件与签名（入场／退场；写下来才可被检查）

**无**。

## unproven 检查（没有出生证明，不计入棘轮）

**无**。

## 仅自述的出生证明（引擎验证不了「我看过它红」这句话）

**无**。
