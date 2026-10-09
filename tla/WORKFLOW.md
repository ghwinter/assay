# TLA+ 检查档工作流（协议层：安全 + 活性）

定位：检查**并发协议模型**——状态、时序、活性（如"阻塞者必然苏醒"）。
不检查实现代码；模型↔代码一致性由桥对账（O-025，D-016）机械维持；断点快照（O-023）是运行时诊断档。
何时用：出现并发协议、阻塞/唤醒、关闭时序等义务时（触发器 shared_mutable_state /
blocking_api 展开的高档证据需求）。

## 依赖清单

| 依赖 | 版本/位置 | 说明 |
|---|---|---|
| tla2tools.jar | 本仓库 `tools/tla2tools.jar`（TLC 2.19，2.2MB） | 随 git 版本固定，升级即换 jar 全量重检 |
| Java | 21（本机实测）；11+ 均可 | 仅运行 TLC |
| 无其他库 | — | 模型是纯文本 .tla/.cfg |

接入新项目：拷贝 `tools/tla2tools.jar` 即可（jar 进 git，版本随仓库锁定）。

## 流程（六步）

1. **建模**：`tla/<name>.tla`——协议状态机（显式阻塞状态 pc，不用 stuttering 伪装）+
   安全不变量 + 公平性（WF_vars 逐动作）+ 活性性质。终态加自环动作避免 TLC 误报死锁。
2. **配置**：`tla/<name>.cfg`——SPECIFICATION / INVARIANT×n / PROPERTY / CONSTANT。
   规模从最小起步（2 生产者/容量 2），先跑通再放大。
3. **运行**：`cd tla && java -cp ../tools/tla2tools.jar tlc2.TLC -cleanup <name>.tla`
   （必须在 tla/ 目录内——模块解析相对 cwd）。大模型可加 `-workers 8`。
4. **判读**："Model checking completed. No error has been found." + 状态数。
5. **入账**：结果写入 `ledger/evidence/O-xxx.yaml`（notes 记状态数与性质清单）。
6. **修订纪律**：义务表述若不可检验，先精化义务再改模型（D-003/O-006 先例）。

## 坑清单（2026-10 实战沉淀，全数在 queue.tla 注释留档）

1. 无序列映射字面量 `<<e : x ∈ S>>`——用函数字面量 `[i ∈ S |-> e]`
2. `let...in` 缩进解析易错——改直接表达式
3. 算术算子需显式 `EXTENDS Naturals`
4. 标准 Bags 模块**没有** `Bag` 构造算子——用 `Cardinality({j ∈ 1..Len(s) : s[j] = e})` 计数
5. TLC 不支持量化时序公式（`\A c: P ~> Q`）——活性直接内联 `[](P => <>(Q))`
6. TLC 不支持参数化算子包装时序体（自定义 `LeadsTo(P,Q)` 会报 cannot handle）
7. cfg 文件必须存在；无 cfg 直接跑会 ConfigFileException
8. 终态需自环动作，否则 TLC 把正常终止报成 deadlock

## 证据语义边界

通过 = 模型在给定常数下全空间满足所列不变量与活性。它**不**证明 C++/Rust 代码
正确——那是测试档、语言强制档与 Verus 档的职责；它证明的是协议设计本身。
