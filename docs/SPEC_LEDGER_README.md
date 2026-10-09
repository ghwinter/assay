# Spec Ledger（规约账本）v0.1

把"从目的到代码的全部决定"作为一等公民放进版本库：**决策记义务，义务挂检查器，
证据入账，CI 强制账目平衡**。AI 在账本约束下工作，人只审计账不平的地方。

宪法见 [CONSTITUTION.md](CONSTITUTION.md)（硬/软二分、三律、增长律）；
AI 纪律见 [AGENTS.md](AGENTS.md)。
**工作流主语言：Rust（D-007）；C/C++/Python/Shell 第二/备选。**

## 目录

```text
LEDGER.md                 账本世界模型单页（digest 机械生成，勿手改；AI 会话入口）
ledger/
  intent/      G-*.yaml    目标账（系统为什么存在，验收义务清单）
  decisions/   D-*.yaml    决策账（问题/备选/选择/理由/义务/产物；D-001 已被 D-007 取代）
  triggers/    *.yaml      触发器库（什么决定自动生成什么义务）
  evidence/    O-*.yaml    证据账（每条义务的检查结果；含升级历史）
  bridge.yaml              模型↔代码桥：常量/性质/语义点三张映射表（D-016）
tools/
  config.toml              机器路径单点（换机只改它）
  gate.py                  四合一提交门：lint/桥/digest/自测（pre-commit 薄壳调用，D-021）
  ledger_lint.py           L1/L2/L3 + schema + 触发器覆盖检查（黄金用例自测见 tools/tests/）
  bridge_check.py          模型↔代码机械对账（B1 常量登记/B2 性质认领/B3 符号三向存在）
  ledger_check.py          检查器运行器：按 evidence 声明复跑（--list / O-xxx / --type）
  digest.py                生成/校验 LEDGER.md（--check 漂移即红灯）
  trace.py                 影响分析（G/D/O/触发器键/文件任一入口）
  tla2tools.jar            TLA+ 检查器（随 git 固定）
.github/workflows/ci.yml  四档 CI：ledger（lint/自测/桥/digest）+ rust + tla + verus（降级档）
pilot-rs/                 主实现试点：Rust 有界 MPMC 队列（零依赖，零 unsafe）
  src/qcore.rs             队列核心单一事实源：Verus 契约 cfg 门控住在实现文件里（D-017）
pilot/                    C++ 对照归档（D-008，只读；证据为历史记录）
tla/                      queue.tla + queue.cfg：队列协议的 TLA+ 模型（D-006）
proofs/                   Verus 验证入口（include 实现本体；契约在 pilot-rs/src/qcore.rs）
experiments/protocol.md   A/B/C 实验预登记（生成质量/决策推翻/反事实审计）
```

## 快速开始

```bash
PY=$(sed -n 's/^python = "\(.*\)"/\1/p' tools/config.toml)   # 路径单点（C:/ 形态，双 shell 通用）
"$PY" tools/gate.py                        # 四合一提交门（pre-commit 同款）
"$PY" tools/ledger_lint.py                 # 账目平衡
"$PY" tools/bridge_check.py                # 模型↔代码桥对账
"$PY" tools/digest.py --check              # LEDGER.md 新鲜度
"$PY" tools/trace.py D-009                 # Rust 并发策略的完整账
cd pilot-rs && cargo test                  # 主实现测试
cargo test --release -- --ignored --nocapture bench_throughput
cd ../tla && java -cp ../tools/tla2tools.jar tlc2.TLC -cleanup queue.tla
```

## 当前状态（2026-10-07，阶段迭代后）

- 账本实时状态以 [LEDGER.md](LEDGER.md) 为准（机械生成的单页视图）：
  目标/决策/义务/证据计数、活跃拓扑、触发器库与元认知库条数均在该页，勿在本页复制数字
- **守门人自证（D-015）**：ledger_lint 拆出可导入 run()，17 例黄金用例（L1-L3/触发器/
  豁免/schema/跨决策义务去重）；机器路径收敛 tools/config.toml；ledger_check 运行器
  按 evidence 复跑检查器（test/bench/lint/model/proof 五档可执行，manual 诚实 SKIP）
- **模型↔代码桥硬化（D-016）**：BS-010 的"评审+快照"薄墙升级为机械对账——
  桥表 8 个语义点（verus 覆盖 5/8，关闭/唤醒时序诚实登记为 shell 层）；
  cfg 检查的 5 条性质全部有义务锚点；BS-010 判例已回写硬化记录
- **证明=实现单一事实源（D-017）**：队列核心抽取 pilot-rs/src/qcore.rs，Verus 契约以
  cfg_attr(verus_verify, verus_spec) 住在实现文件——cargo 构建契约不存在（纯净零依赖），
  verus --cfg verus_verify 契约生效，4 verified 0 errors；证明对象与编译对象永不脱钩
- **四档 CI（D-018）**：ledger（lint/黄金用例/桥对账/digest 新鲜度）+ rust + tla +
  verus（发布资产下载易碎，诚实 continue-on-error）；LEDGER.md 世界模型单页 +
  mermaid 活跃拓扑机械生成
- **实验 A 实验组完成（D-019）**：timed_pop 走完整账本流程——blocking_api 触发器展开
  2 条义务，5 条新测试首跑全绿，TLA⁺ 模型零扩展覆盖（谓词重检同构论证）；
  人工介入点 1（操作者批量授权）；对照组（无账本会话）与异体评审登记待运行
- 全量回归（2026-10-07）：lint PASS / 17 自测 OK / 桥平衡 / digest 新鲜 /
  cargo test 10 passed 2 ignored / 吞吐达标（G-003 预算 50k）/ TLC 2,426 状态无错误 /
  verus 4 verified 0 errors
- 下一个升级候选：loom（调度模拟）、抛异常/panic 安全专项、对照组实验 A、G-003 p99（真实锚点出现时）

## 过程记录（账本起作用的实例）

1. C++ 试点期间：压力测试初版出现真实数据竞争（循环变量按引用捕获），
   义务 O-003 的升级优先级据此上调——已留档。
2. O-006 原表述"总 FIFO 可外部检验"不可检验，被精化为单消费者保序形式——
   义务必须写成可检验形式。
3. 语言转向全程走账：D-001 superseded_by D-007，C++ 产物改由 D-008 认领归档，
   无一文件失主（L1 在转向期间持续全绿）。
4. **义务下沉的第一次实录**：迁入 safe Rust 后，O-011/O-019 从"审查/测试档"
   升到"编译器强制档"——宪法增长律（软→硬迁移）在语言层兑现。
5. **TLC 实操教训**（沉淀于 tla/queue.tla 注释）：TLA+ 无序列映射字面量（用函数
   字面量代替）；let-in 缩进规则易踩坑（可改直接表达式）；算术算子需显式
   EXTENDS Naturals；标准 Bags 模块无 Bag 构造算子（用计数表达恰好一次）；
   TLC 不支持量化时序公式与参数化算子包装的时序体（时序公式直接内联）。
6. **守门人先自证**（D-015）：为 lint 写黄金用例时发现跨决策义务 id 重复不被检测
   ——给守门人加检查器，守门人自己长了一级。
7. **桥对账抓出自身缺陷**（D-016）：bridge_check 首跑 13 个误报，根因是检查器
   自己的两处正则缺陷（findall 元组、CONSTANT 前缀）——对账工具先被对账。
8. **digest 红灯抓到操作者**（D-018）：账本变更后未重生成 LEDGER.md 即提交，
   `--check` 漂移红灯拦下——检查器对执行者自身的账目同样生效。
