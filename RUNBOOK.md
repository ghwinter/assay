# RUNBOOK — 具体命令与本机工具链

> **本文件是按需加载的，`AGENTS.md` 是常驻注入的。** 所以具体命令、路径、版本号都住在这里。
> 理由是一次实测：`AGENTS.md` 里曾是 `python tools/gate.py`，而本机 PATH 上的 `python`
> 是 Microsoft Store 的 App Execution Alias 占位符、且没有解释器装 PyYAML——
> 于是四道门一次都没执行过，而状态板全绿。
> **常驻的规则里写机器相关的命令，就等于把一份会腐化的声明钉在每个人的上下文里。**

## 唯一入口

```text
assay help          # 动词表
assay doctor        # 就绪度；不跑检查
assay next          # 现在该做什么：有序动作表
assay check         # 清单自检（门禁；不跑检查）
assay map           # 由清单机械生成 MAP.md
assay run [--stage pre-commit|pr|merge|nightly] [--check <id>] [--change <id>] [--certify]
assay plan --base <rev> [--change <id>] [--format json]
assay gate          # 四合一提交门（lint / 桥 / digest / 自测）——pre-commit 同款
assay lint | bridge | digest [--check] | trace <id> | ledger <O-id> | spec <...> | wp <D-id>
assay bootstrap     # 准备/自检 Python 侧（缺 PyYAML 时现场建 tools/.venv）
```

解析顺序（`tools/assay-cli.ps1`）：

```text
1. ASSAY_PYTHON 环境变量                    显式指定（机器本地，不进版本库）
2. tools/config.toml 的 python 裸名          共享配置，可移植
3. 日常 venv                                  %USERPROFILE%\{envs,venvs,.venvs,.virtualenvs}\*\Scripts\python.exe
4. tools\.venv                               仓库本地兜底
5. uv 现场建 tools\.venv
6. uv 临时环境（--with pyyaml）
7. 任何能 import yaml 的解释器
```

前四步都要求"能跑 **且** 有 PyYAML"——七个账本工具都 `import yaml`，少了它整个账本侧不可用。
每次调用账本工具都会打印一行 `[py] <实际使用的解释器>`；设 `ASSAY_QUIET=1` 可关掉。
**解析不到就报错退出 127，不静默降级。**

- 共享配置里 `python` 保持**裸名**（由 `tools/tests/test_repository_boundary.py` 强制）；
  机器相关的绝对路径不进版本库——要固定用某个解释器就设 `ASSAY_PYTHON`。
- 只有复制了 Python 工具、没有入口脚本的下游项目，`.githooks/pre-commit` 会回退到裸 `python`。

## 机械操作的已知坑（不进口径文件，故写在这里）

- **绝不用数组区间重写源文件。** PowerShell 的 `$lines[1095..888]`（**降序**）是**反向切片**：
  它会把整段复制并倒置，实测毁过一整个源文件，靠已提交的绿色版本才恢复。
  移动代码只用**字面量替换**（不匹配即中止），或 `git mv`。
- 同源的坑：`@($arr[1..($arr.Count-1)])` 在 `$arr.Count -eq 1` 时，区间 `1..0` 会**降序**展开成
  `@(1,0)`，于是把首元素当成下标 1 用。本仓的启动器踩过一次，症状是
  `The term 'D' is not recognized`（路径首字母被当成了命令名）。
  单元素情况的写法：`if ($arr.Count -gt 1) { $arr[1..($arr.Count-1)] } else { @() }`。
- **检查脚本不得改写共享文件。** 一条自检曾用改写 `.gitignore` 的方式做探针，清理失败后把探针路径
  留在了里面，**静默污染了后续所有运行**；唯一发现它的原因是那条检查写了反向断言。
  探针请用**既有忽略规则覆盖的目录**（如 `__pycache__/`），不要动共享文件。

## 引擎（Rust，不需要 Python）

```text
cargo build --release --manifest-path engine/Cargo.toml
cargo test  --release --manifest-path engine/Cargo.toml
```

## 主实现与判定器

```text
cd pilot-rs && cargo test
cd pilot-rs && cargo test --release -- --ignored --nocapture bench_throughput   # O-015 吞吐阈值
cd tla && java -cp ../tools/tla2tools.jar tlc2.TLC -cleanup queue.tla
         # TLC 按文件名解析模块，必须在 tla/ 内运行
<verus> --cfg verus_verify proofs/queue_core.rs
         # <verus> 取自 tools/config.toml 的 [tools].verus（裸名，走 PATH）
cd pilot && bash build.sh          # C++ 归档对照（D-008，只读）
```

## 解释器与工具链现状（会腐化，故不写进 AGENTS.md）

- 账本工具（7 个 Python 脚本）全部 `import yaml`；缺 PyYAML 时**整个账本侧不可用**。
  用 `assay bootstrap` 查看实际使用的解释器与 yaml 版本。
- 本机实测坑：PATH 上的 `python` 可能是 0 字节的重解析点占位符——`Get-Command` 有结果，执行即失败。
- 引擎、清单协议、证据 schema 是**三条独立版本轴**；`assay version` 报告引擎与它支持的协议。
- `ASSAY_ENGINE` 环境变量可把自检指向一个**故意做坏的引擎副本**（`assay/selfcheck/_inject.ps1`）。

## 让 pre-commit 真正生效

```text
git config core.hooksPath .githooks
```

没设这一行，`.githooks/pre-commit` 就是一份不会被执行的文本——`assay doctor` 不检查它，
因为"钩子有没有装"是仓库本地的 git 配置，不是清单能声明的事实。**这条只能靠人记得。**
