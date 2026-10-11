# assay engine

**Assay CLI v0.6.0**：读取清单 → 解释或执行检查计划 → 追加版本化证据 → 生成地图 → 比对棘轮 → 报分歧。支持清单协议 v0.6 和 v0.7。

> **它不认识任何判定工具**（E1／E2 那类后端都是它 exec 的对象，不是它的一部分），
> **也不拥有任何产物**——声明、检查、证据、地图、棘轮全部住在目标仓里。
> 删掉引擎只失去自动化，不失去知识。这是它区别于 MDA/CASE 工具的根本性质。

## 命令

```powershell
assay --help
assay --version
assay version
assay plan --root <目标仓> --base <git-revision> [--change <id>] [--format text|json]
assay run   [--root <目标仓>] [--stage pre-commit|pr|merge|nightly] [--check <id>] [--change <id>] [--certify]
assay run   [--root <目标仓>] --base <git-revision> [--change <id>] [--certify]
assay map   [--root <目标仓>]      # 生成 MAP.md
assay check [--root <目标仓>]      # 清单自检 + 覆盖 + 出生证明三态 + 变更记录 + 棘轮 + 地图漂移
```

退出码：全绿 `0`／有红 `1`／清单本身有问题 `2`。

从 Assay 仓库根目录安装本机 CLI：

```powershell
cargo install --path engine
```

不安装时可用 `cargo run --manifest-path <assay-root>/engine/Cargo.toml -- ...`；目标仓通过 `--root` 指定，不需要复制或嵌套 Assay 引擎。

## 变化计划

- `assay plan --base REV` 汇总 Git 相对差异、输入范围命中、未映射路径和待跑检查；`--format json` 输出 `assay-plan/v1`。
- `assay run --base REV` 使用同一选择算法执行；默认 `assay run` 仍执行全量或所选阶段的检查。
- `[[check]].inputs` 只接受精确文件路径或目录前缀（尾部 `/`）；无范围检查始终纳入，未命中任何范围的变化会选中全部检查。
- 选择依据是维护者显式登记的路径，不是语义影响分析；不声明范围或不确定时使用全量 `run`。
- 目标仓必须是 Git 仓库根目录，且必须显式传入 base revision。

**实测成本**（本仓 9 条检查，Windows，release）：

| 动作 | 耗时 |
|---|---|
| `run --stage pre-commit`（3 条检查） | **2.6 s** |
| `run`（9 条检查，证书沿用） | **8.2 s** |
| `run --certify`（9 条证书全部当场重跑，含 6 次注入版编译） | **51 s** |

证书复现是**一次性**成本，不是每次提交的成本。这就是把"出生证明"从自述变成机械的价钱。

## 行为要点（都是实测定下来的）

| 行为 | 为什么这样 |
|---|---|
| 命令经**平台壳**执行（Windows `cmd /C`、其它 `sh -c`），且 Windows 下用 `raw_arg` 绕开 Rust 的引号转义 | 实测：`pwsh -Command` 会把 `"路径" --flag` 当表达式解析而 ParserError；`cmd /C` + `arg()` 又会把引号转义成 `\"` 而 cmd 不认 |
| 新检查进入棘轮的条件 = **这一跑绿过 + 有出生证明** | "兑现"= 它红过（有能力失败）且它绿过（确实能过）。缺一条不进 |
| `red_command` **非零 = 证书成立** | 它要证明的就是"这条检查能失败"。所以**所有"没证明出来"的情况一律归 0**——一个坏掉的见证者必须闭嘴，不能点头 |
| 证书只在**尚未 `certified`** 或 `--certify` 时才当场重跑 | 否则每次门禁都要编译一个做坏的引擎副本 |
| `certified` 位只能由机器写 | 自述字符串永远换不来它。这是 v0.4 修的那个洞 |
| `run` 结束后**自动重生成地图** | 棘轮变化会改地图内容 ⇒ 否则每次 run 都留下"漂移陷阱" |
| 每次检查执行追加一条 JSONL 事件 | `.jsonl` 是历史事实源；同名 `.json` 只是最新记录投影 |
| 失败输出**只给末尾 12 行 + 指针** | 门禁输出也是 token 预算；一整屏等于没有输出 |
| `check` 把 **`trust_assumption`** 单列 | 无法机械检查的前提必须显式登记，不能藏在"看起来已经验证过"后面 |
| 分歧只报不判 | 引擎能看见"同一声明上的检查互相矛盾"，但**判断谁对是人的事** |

## 它自己的出生证明（本引擎的门禁也被故意违反过一次）

早期五条（手工注入，原始输出见 `tmp/verus-01/assay/evidence/ratchet.first-red.txt`）：

| 门 | 注入 | 观察到的结果 |
|---|---|---|
| 地图漂移 | 手写 MAP.md 后直接 `check` | `❌ MAP.md 已漂移 —— 跑 assay map 重新生成` ⇒ FAIL |
| 棘轮（命令指纹） | 给命令里的路径加引号（语义等价） | `❌ K-001（命令指纹变化）无 waiver` ⇒ FAIL |
| 棘轮（解绑声明） | `claims = ["C-001"]` | `❌ K-001（解绑声明 ["C-002"]）无 waiver` ⇒ FAIL |
| 豁免通道 | 加 `[[waiver]]` | `⚠ 有 waiver：…｜签名 demo` ⇒ OK |
| 覆盖缺口 | 解绑后 C-002 无检查 | `❌ 没有未绑定检查的声明（缺口 1）` ⇒ FAIL（**豁免只覆盖棘轮那一条**，缺口是另一件事） |

现在这五条已被**狗粮**接管：本仓的 `assay/manifest.toml` 有 9 条检查，每条都带一条
`red_command`，`assay run --certify` 会当场把每一条都跑红一次。见 [../README.md](../README.md)。

## 依赖登记（照 VENDOR.md 的习惯：每个依赖必须有理由）

见 [VENDOR.md](VENDOR.md)。当前 4 个依赖，分别支撑清单读取、证据写入与 CLI 参数解析。

证据事件格式见 [`../protocol/schemas/evidence-v1.schema.json`](../protocol/schemas/evidence-v1.schema.json)。

## 仓库迭代与发布

**当前没有 CI。** `.github/workflows/` 已整体移除（D-035）。强制层是本地提交门：
`.githooks/pre-commit` → `tools/gate.py`（账目 lint / 桥对账 / digest 新鲜 / 守门人自测），
要求 `git config core.hooksPath .githooks` 已设置——不设这一步，钩子只是一段文本。

被移除的是两个 workflow：多平台矩阵的 ci.yml（引擎/试点/示例的测试、格式化与 clippy、
三个清单的 check 与地图新鲜度）与按 `vX.Y.Z` 标签发布的 release.yml。
恢复条件写在 D-035：需要先有远端、且外部判定器（TLA+ / Verus）有可复现的跨 runner 安装步骤。

已提交的工具配置使用 PATH 命令名，不包含开发机绝对路径。

## 明确不做

- 不认识判定工具，不解析任何检查的输出语义
- 不拥有产物（不写任何领域知识）
- 不做插件系统、不联网、不做 UI
- **不推断语义影响**：`--change` 沿清单里已有的 `claim ↔ check` 绑定选检查；`--base` 只按维护者显式登记的路径范围规划，无法证明这些关系完整
- **不为"将来可能的项目"加特性**：每个特性都要能说出是"哪个项目的哪一次真实摩擦"买来的
