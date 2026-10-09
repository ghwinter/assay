# 标准工作流：Spec Ledger 项目作业程序

适用条件：预计 ≥10³ 行、含并发/持久化/外部依赖、或多 AI 会话协作的项目。
不适用：≤数百行的脚本（直接写）。

## 第 0 步：项目初始化（一次性，约 1 小时）

0. （推荐）`python tools/spec.py init <目录> <项目名>` 自动完成下述 1–3 步：
   拷贝五件套、账本空置、config python 自移植、预生成 LEDGER.md、git init+hooks。
   首提留给你写完 G-001 之后（见脚本输出清单）。手工做则按下述步骤：
   - `CONSTITUTION.md`（宪法，原样）
   - `AGENTS.md`（改：主语言、本机命令；机器路径单点收敛 `tools/config.toml`）
   - `ledger/triggers/`（触发器库，继承并按领域增补）
   - `tools/`（ledger_lint + trace + ledger_check + bridge_check + digest + config.toml）
     + `.githooks/`（git config core.hooksPath .githooks）+ `.github/workflows/ci.yml`
   - `thinking/`（判例法，全仓库通用，直接继承）
   - `experiments/protocol.md`（按项目改指标）
2. `ledger/intent|decisions|evidence` 清空——**禁止 BDUF**：不预写任何决策，账本与代码共演化
3. git init + 首提 `chore: 项目初始化（Spec Ledger 工作流）`

## 第 1 步：意图 → 目标账（人的活，AI 起草、你裁决）

- 你写一段话：引擎要什么、给谁用、什么环境下什么数字算成功
- AI 起草 `G-*.yaml`：每个目标带**可测性质 + 场景定义 + 锚点**（下游谁需要？
  BS-001 防线：无锚点预算标 synthetic）
- 后端领域触发器增补建议：持久化/恢复、网络部分失败、背压、优雅关停、
  配置与热加载、超时与重试语义

## 第 2 步：第一条纵切（半天）

选一个最小但真实的功能（不是 demo）。流程全走一遍，目的是验证流水线本身：

```
AI 起草 D（备选集 + 理由带锚点 + 触发器展开义务；
           落选方案的完整论证移入 rejected_alternatives 字段，主文保持锐度）
→ 你裁决 accepted
→ AI 实现 + 每条义务挂检查器（阶梯最低档起步）
→ ledger-lint 绿 + bridge 平衡 + digest 新鲜 + 测试过 = 完成定义
→ 提交（Conventional Commits）
```

## 第 3 步：日常循环（每个功能/变更）

**新功能**：起草 D → 裁决 → 实现 → checker → lint 绿 → 提交。
**目标/约束变更**（红灯路径）：
1. `trace.py <id>` 查影响面（哪些 D/O/工件受累及）
2. 先测量缺口（D-012 模式），方案按（修复成本 × 失效集）排序
3. 受累及决策显式 supersession，义务历史化，重测，绿灯

纪律：红灯说"代码≠决策"，三条合法响应（改代码 / 带锚点改决策 / 撤回无锚点阈值）；
被阻塞的提交是系统在工作。

## 第 4 步：证据升级节奏（按需，不预先）

- 遇并发协议 → TLA+（模型检查活性/安全）
- 核心结构需要证明 → Verus
- 竞争疑点 → loom / TSAN
- 上线后出事 → cdb 证据采集 → 判例化进 thinking/ + 触发器回写

## 第 5 步：度量与退出（四周裁决）

预登记指标（AI 首次通过率、返工率、变更传播成本）。
到期按数据裁剪账本条目或关停工作流——**账本自身接受被推翻**。

## AI 介入的实际操作（每天怎么用）

每个会话开头：AI 读 AGENTS.md + **LEDGER.md**（机械生成的世界模型单页；细节按需查
ledger/ 各账）+ thinking/（评审对照）。
你的日常动作只有三种：**裁决 G/D、标注金标准、验收证据**。
强制介入点只有一条：**没有 accepted 的决策，AI 不得改 src/**——钩子红灯兜底，CI 红灯兜远端。

## 示例走查：后端游戏服务器（CI 概念的实际形状）

CI 的新定义：不是"编译+测试"，是**工程知识图谱的一致性门**——提交门问"账实相符吗"
（四合一：lint/桥/digest/自测），迭代问"变更传播到哪"（trace→测量→supersession）。
闭环：目的 → G（五问+open_questions）→ D（spec expand 展开触发器义务）→ 实现
（每义务挂检查器，阶梯最低档起步）→ spec check 归一化证据 → 提交门 → CI 轨道 →
回写（判例/触发器/义务下沉注销）。

以"后端游戏服务器：运行时、数据库、持久化、中间件、组件模块、计算资源底层分配"
为例，构件→触发器→检查器的点亮路径：

| 构件决策 | 触发器（库/候选池） | 首档检查器 | 升级档 |
|---|---|---|---|
| 语言与运行时 | shared_mutable_state, blocking_api | lint+test | TLA+（协议） |
| 数据库/持久化 | adds_persistence | kill -9 恢复 test | TLA+（崩溃恢复） |
| 网络中间件 | adds_network_io | 故障注入 test | — |
| 后台任务（匹配/计时） | adds_background_task | test | — |
| 配置热加载 | adds_config_surface | test / TLA+ 状态机 | — |
| 插件/模组 API | adds_public_api | property test | — |
| 多实例/分区 | goes_multi_instance | — | TLA+（split_brain） |
| 调度器/资源分配内核 | value_semantics + shared_mutable_state | test+bench | Verus（核心结构）+ TLA+（调度公平性） |

候选池触发器激活需 D 裁决（ledger/triggers/CANDIDATES.md）；性能类预算无下游锚点
标 synthetic（BS-001）。AI 会话装载顺序：CONSTITUTION → AGENTS → LEDGER.md →
thinking/；命令面：spec init/expand/check、trace、digest；重器叶子：cargo/TLC/verus/cdb。
