# AI 工作纪律（对一切 agent 生效）

动手前必读：`CONSTITUTION.md`，然后读 `LEDGER.md`（机械生成的世界模型单页），
细节按需查 `ledger/` 各账。**账本就是你在本仓库的世界模型。**
做决策与评审时对照 `thinking/blind_spots.md` 与
`thinking/methods.md`（元认知库，条数见 LEDGER.md；术语以 `thinking/glossary.md`
规范名表为准）——判例法生长：每次真实错误推理后回写。

**工作流主语言为 Rust**（D-007）；C/C++/Python/Shell 为第二/备选。试点主实现是
`pilot-rs/`（Rust）；`pilot/`（C++）是归档对照基准（D-008），只读不演进。

1. **没有 accepted 的决策，不得改代码。** 修改 `pilot-rs/**`、`pilot/**`
   或新增源文件之前，必须存在覆盖该变更的 `D-*.yaml` 且 `status: accepted`。
2. **新决策走完整流程**：起草 `D-*.yaml`（`status: draft`）→ 按 `ledger/triggers/`
   展开义务 → 人裁决为 `accepted`。义务必须覆盖触发器要求，或给出带 `reason + owner` 的豁免。
   落选方案的完整论证移入 `rejected_alternatives` 字段（D-018 约定），主文保持锐度。
3. **每条义务同时给 checker**。当前跑不了的工具用 deferred + reason + owner，不许沉默缺失。
4. **完成的定义**：`ledger_lint.py` 全绿（0 error）+ `bridge_check.py` 平衡
   + `digest.py --check` 新鲜 + 所涉义务 evidence 为 pass（或带签名的 deferred）。
   前三项由 pre-commit 门 `tools/gate.py` 强制（D-021，CI 同款）。
5. **汇报账目摘要**：改了哪些 G/D/O/E，而不是只给代码 diff。
6. 账本没覆盖的冲突，升级给人裁决，不要自行立法。

## AI 产出纪律（D-020 合并第(1)步，收编手册 discipline §4）

- **证据待验**：AI 每轮产出视为证据待验，过检查器或操作者签名后方可引用为结论。
- **未知标注**：陈述未知时标"未知"，不得以流畅文本填充（防伪精确；BS-011 同源）。
- **双轨标注**：产出标注轨道——强逻辑域（须过闭环验证方可引用）/ 探索域
  （不得被强逻辑域消费者直接消费）。
- **链长预算**：长推理链按间隔插入独立校验节点（测试/审计/第二通道），
  不靠加长生成自证。
- **产物表=跨轮状态**：跨轮记忆不可靠时，产物记录表是唯一跨轮状态；
  每轮先读前轮记录再动手。
- **新形态先举例**：提出新方法/新判例须先给实例与反例再入元认知库
  （防过早公理化）。

## 命令

路径单点：`tools/config.toml`（换机只改它）。以下 `PY` 取 config `[tools].python`。

```bash
PY=$(sed -n 's/^python = "\(.*\)"/\1/p' tools/config.toml)   # 路径单点（C:/ 形态，双 shell 通用）

"$PY" tools/gate.py                        # 四合一提交门（pre-commit 同款：lint/桥/digest/自测，O-032）
"$PY" tools/ledger_lint.py                 # 账目平衡（L1/L2/L3）
"$PY" tools/bridge_check.py                # 模型↔代码桥对账（O-025）
"$PY" tools/trace.py D-009                 # 影响分析
"$PY" tools/digest.py                      # 重新生成 LEDGER.md（账本变更后必须）
"$PY" tools/digest.py --check              # 新鲜度校验（CI 同款）
"$PY" tools/ledger_check.py --list         # 义务→检查器映射
"$PY" tools/ledger_check.py O-011          # 复跑某义务的检查器（可执行档）

cd pilot-rs                               # 主实现（Rust，D-007）
cargo test                                # 断言测试（O-013/014/017/018/030…）
cargo test --release -- --ignored --nocapture bench_throughput   # 吞吐（O-015）

cd ../pilot && bash build.sh              # C++ 归档对照（D-008，只读）

cd tla && java -cp ../tools/tla2tools.jar tlc2.TLC -cleanup queue.tla   # 协议模型检查（O-016/017；TLC 按文件名解析模块，必须在 tla/ 内运行）
"$PY" -c "import tomllib;print(tomllib.load(open('tools/config.toml','rb'))['tools']['verus'])"  # verus 路径
<verus> --cfg verus_verify proofs/queue_core.rs                 # 机器证明（O-028，<verus>=config 路径）
```

工具链（本机实测）：Rust 1.99.0 stable（rustup，主）+ 1.98.1（Verus 运行必需）；
w64devkit g++ 14.1（归档对照）；Python 3.12.13（uv，含 PyYAML 6.0.3）；
TLA+ tla2tools.jar + Java 21；Verus 0.2026.09.27 发布版（路径见 config）。
