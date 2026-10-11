# assay-cli.ps1 —— **唯一入口**。把"引擎"和"账本工具"两半合成一个命令。
#
# 存在的理由（一次真实摩擦）：
#   AGENTS.md 写着 `python tools/gate.py`，而本机上 `python` 是 Microsoft Store 的占位符，
#   且没有任何解释器装了 PyYAML——**七个账本工具全部 import yaml，全部跑不起来**，
#   于是"硬规则由 pre-commit 强制"这件事在本机是**纸面上的**。
#   四道门一次都没跑过，而 `assay check` 仍然全绿：绿的是清单，不是工作。
#
# 本脚本只做三件事，且都不改变任何判定语义：
#   1. 解析 Python（config 提示 → PATH → uv 托管 → 常见安装位置），并确认 PyYAML 可用
#   2. 把动词分派给引擎（Rust）或账本工具（Python）
#   3. 解析不到时**说清楚缺什么、怎么修**，不猜替代品
#
# 用法：
#   .\assay.cmd doctor            # 环境与就绪度（不跑检查）
#   .\assay.cmd next              # 现在该做什么（只读导览）
#   .\assay.cmd check             # 清单自检（门禁）
#   .\assay.cmd run --stage pr    # 跑检查
#   .\assay.cmd gate              # 四合一提交门（lint/桥/digest/自测）
#   .\assay.cmd lint | bridge | digest | trace <id> | ledger <O-id>
#   .\assay.cmd spec expand D-001 # 账本工具直通
#   .\assay.cmd wp D-001          # 从已裁决决策生成 AI 工作包
#   .\assay.cmd -- <任意原生命令>

[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

function Write-Note([string]$m) { Write-Host $m -ForegroundColor DarkGray }
function Write-Bad([string]$m) { Write-Host $m -ForegroundColor Red }

# ── Python 解析 ─────────────────────────────────────────────────────────────

function Get-ConfigPythonHint {
    $cfg = Join-Path $Root 'tools\config.toml'
    if (-not (Test-Path $cfg)) { return $null }
    foreach ($ln in Get-Content $cfg) {
        if ($ln -match '^\s*python\s*=\s*"(.*)"\s*$') {
            $v = $Matches[1]
            # 裸名交给 PATH 解析；含分隔符的才当路径。共享配置里只会是裸名。
            if ($v -match '[\\/]') { return ($v -replace '/', '\') }
            $c = Get-Command $v -ErrorAction SilentlyContinue
            if ($c -and $c.Source -and $c.Source -notmatch 'WindowsApps') { return $c.Source }
            return $null
        }
    }
    return $null
}

function Test-Interpreter([string]$exe) {
    if (-not $exe) { return $false }
    try {
        $null = & $exe -c "import sys" 2>$null
        return ($LASTEXITCODE -eq 0)
    } catch { return $false }
}

function Test-HasYaml([string]$exe) {
    try {
        $null = & $exe -c "import yaml" 2>$null
        return ($LASTEXITCODE -eq 0)
    } catch { return $false }
}

function Get-PythonCandidates {
    $out = @()
    $hint = Get-ConfigPythonHint
    if ($hint) { $out += ($hint -replace '/', '\') }
    foreach ($n in 'python', 'python3') {
        $c = Get-Command $n -ErrorAction SilentlyContinue
        # 跳过 Microsoft Store 的 App Execution Alias 占位符（0 字节的重解析点）
        if ($c -and $c.Source -and $c.Source -notmatch 'WindowsApps') { $out += $c.Source }
    }
    $uvRoot = Join-Path $env:APPDATA 'uv\python'
    if (Test-Path $uvRoot) {
        $out += (Get-ChildItem $uvRoot -Directory -Filter 'cpython-3.1*' -ErrorAction SilentlyContinue |
                 Sort-Object Name -Descending |
                 ForEach-Object { Join-Path $_.FullName 'python.exe' })
    }
    foreach ($base in @("$env:LOCALAPPDATA\Programs\Python", 'C:\Python313', 'C:\Python312')) {
        if (Test-Path $base) {
            $out += (Get-ChildItem $base -Filter python.exe -Recurse -Depth 2 -ErrorAction SilentlyContinue |
                     Select-Object -ExpandProperty FullName)
        }
    }
    return ($out | Where-Object { $_ } | Select-Object -Unique)
}

# 用户日常用的 venv：按**约定俗成的目录**发现，**不写死任何机器相关的绝对路径**
# （写死就等于把上一轮刚修掉的那个错误换个地方再犯一次）。
# 名字降序 ⇒ 版本号大的优先；有多个时由 ASSAY_PYTHON 显式指定来定夺。
function Get-DailyVenvPythons {
    $roots = @(
        (Join-Path $env:USERPROFILE 'envs'),
        (Join-Path $env:USERPROFILE 'venvs'),
        (Join-Path $env:USERPROFILE '.venvs'),
        (Join-Path $env:USERPROFILE '.virtualenvs')
    )
    $out = @()
    foreach ($r in $roots) {
        if (-not (Test-Path $r)) { continue }
        $out += (Get-ChildItem $r -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending |
                 ForEach-Object { Join-Path $_.FullName 'Scripts\python.exe' } |
                 Where-Object { Test-Path $_ })
        $out += (Get-ChildItem $r -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending |
                 ForEach-Object { Join-Path $_.FullName 'bin\python' } |
                 Where-Object { Test-Path $_ })
    }
    return $out
}

function Resolve-PythonPrefix {
    # 返回一个**命令前缀数组**，把 python + 脚本 拼在它后面即可执行。
    # 优先级（从"你最想让谁跑"到"兜底"）：
    #   1. ASSAY_PYTHON 环境变量      —— 显式指定，机器本地，不进版本库
    #   2. tools/config.toml 的裸名   —— 共享配置，可移植
    #   3. 日常 venv（%USERPROFILE%\envs|venvs|.venvs|.virtualenvs\*\Scripts）—— 约定俗成目录，不写死路径
    #   4. tools\.venv                —— 仓库本地兜底
    #   5. uv 现场建 tools\.venv
    #   6. uv 临时环境（--with pyyaml）
    #   7. 任何能 import yaml 的解释器
    # 前四步都要求"能跑 **且** 有 PyYAML"——七个账本工具都 import yaml，
    # 少了它整个账本侧不可用（这正是原先四道门空转的成因）。
    $venvPy = Join-Path $Root 'tools\.venv\Scripts\python.exe'

    # 显式指定必须被**执行**，不能被"更合适的"候选悄悄顶掉。
    # 一个被无声忽略的显式指令，比没有这个开关更坏：你以为你指定了。
    if ($env:ASSAY_PYTHON) {
        $want = ($env:ASSAY_PYTHON -replace '/', '\')
        if (-not (Test-Interpreter $want)) {
            Write-Bad "❌ ASSAY_PYTHON 指向的解释器跑不起来：$want"
            Write-Bad "   （要么修它，要么清掉这个环境变量。它不会被静默忽略。）"
            exit 127
        }
        if (-not (Test-HasYaml $want)) {
            Write-Bad "❌ ASSAY_PYTHON 指定的解释器缺 PyYAML：$want"
            Write-Bad "   七个账本工具都 import yaml。装它：<该解释器> -m pip install pyyaml"
            Write-Bad "   （或用 uv：uv pip install --python `"$want`" pyyaml）"
            exit 127
        }
        return @($want)
    }

    $ordered = @()
    $hint = Get-ConfigPythonHint
    if ($hint) { $ordered += $hint }
    $ordered += Get-DailyVenvPythons
    $ordered += $venvPy

    foreach ($exe in ($ordered | Where-Object { $_ } | Select-Object -Unique)) {
        if ((Test-Interpreter $exe) -and (Test-HasYaml $exe)) { return @($exe) }
    }

    $uv = Get-Command uv -ErrorAction SilentlyContinue
    if (-not $uv) { $uv = Get-Command "$env:USERPROFILE\.local\bin\uv.exe" -ErrorAction SilentlyContinue }
    $uvPath = if ($uv) { $uv.Source } else { $null }

    if ($uvPath) {
        Write-Note "note: 没有现成的解释器带 PyYAML，正在用 uv 准备 tools\.venv（一次即可）"
        & $uvPath venv --python 3.12 (Join-Path $Root 'tools\.venv') 2>&1 | Out-Null
        & $uvPath pip install --python $venvPy pyyaml 2>&1 | Out-Null
        if ((Test-Interpreter $venvPy) -and (Test-HasYaml $venvPy)) {
            Set-ConfigPython $venvPy
            return @($venvPy)
        }
    }

    $starved = @()
    foreach ($exe in Get-PythonCandidates) {
        if (-not (Test-Interpreter $exe)) { continue }
        if (Test-HasYaml $exe) { return @($exe) }
        $starved += $exe
    }
    if ($uvPath -and $starved.Count -gt 0) {
        Write-Note "note: 解释器 $($starved[0]) 缺 PyYAML，改用 uv 临时环境（--with pyyaml，不改动它）"
        return @($uvPath, 'run', '--no-project', '--with', 'pyyaml', 'python')
    }
    return $null
}

# 把解析到的解释器**记下来**，但**不写回 config.toml**：
#   共享配置必须保持可移植（裸名），这条由 tools/tests/test_repository_boundary.py 强制。
#   机器相关的解析结果留在本机，不进版本库。
function Set-ConfigPython([string]$exe) {
    $null = $exe
    Write-Note "note: 本机账本工具将使用 $exe（不写回 config.toml：共享配置须保持可移植）"
}

function Invoke-PythonTool([string[]]$PyArgs) {
    # 注意：PowerShell 会把**单元素数组**在返回时拆成标量，所以这里必须再包一层 @()，
    # 否则 $prefix[0] 取到的是路径的第一个字符（实测踩过：'D' is not recognized...）。
    $prefix = @(Resolve-PythonPrefix)
    if ($prefix.Count -eq 0 -or -not $prefix[0]) {
        Write-Bad "❌ 找不到可用的 Python（需要 3.11+ 且能 import yaml）。"
        Write-Bad "   修法（任选其一）："
        Write-Bad "     · 装 uv 后重试（uv 会自动准备 PyYAML，不污染系统解释器）：https://docs.astral.sh/uv/"
        Write-Bad "     · 或 pip install pyyaml，并把 tools/config.toml 的 python 指向该解释器"
        Write-Bad "   引擎那一半不需要 Python：assay doctor / next / run / plan / map / check 仍可用。"
        exit 127
    }
    $exe = $prefix[0]
    $pre = if ($prefix.Count -gt 1) { $prefix[1..($prefix.Count - 1)] } else { @() }
    # 把**实际用了哪个解释器**打出来。原先那次事故（文档里的命令在本机跑不了、
    # 四道门空转而状态板全绿）能瞒两个引擎版本，就是因为没有任何一处说出这句话。
    if (-not $env:ASSAY_QUIET) { Write-Note "[py] $exe" }
    & $exe @pre @PyArgs
    exit $LASTEXITCODE
}

# ── 引擎 ────────────────────────────────────────────────────────────────────

function Get-EnginePath {
    # 引擎可能不在这棵树里：新项目只继承账本工具与入口，不继承引擎源码。
    # 所以按 1→4 找，找不到就**说清楚哪一半还能用**，而不是抛一句"找不到文件"。
    $exe = Join-Path $Root 'engine\target\release\assay.exe'
    if (Test-Path $exe) { return $exe }

    if ($env:ASSAY_ENGINE_BIN -and (Test-Path $env:ASSAY_ENGINE_BIN)) { return $env:ASSAY_ENGINE_BIN }

    $onPath = Get-Command assay -ErrorAction SilentlyContinue
    if ($onPath -and $onPath.Source -and $onPath.Source -notmatch 'assay\.cmd$') { return $onPath.Source }

    $crate = Join-Path $Root 'engine\Cargo.toml'
    if (Test-Path $crate) {
        $cargo = Get-Command cargo -ErrorAction SilentlyContinue
        if (-not $cargo) {
            Write-Bad "❌ 引擎未构建且找不到 cargo：$exe"
            Write-Bad "   修法：cargo build --release --manifest-path engine\Cargo.toml"
            exit 127
        }
        Write-Note "note: 引擎未构建，先 cargo build --release（只此一次）"
        & $cargo.Source build --release --manifest-path $crate | Out-Null
        if (Test-Path $exe) { return $exe }
        Write-Bad "❌ 构建后仍找不到 $exe"
        exit 2
    }

    Write-Bad "❌ 本仓库没有引擎，PATH 上也没有 assay。"
    Write-Bad "   引擎动词（doctor / next / check / run / plan / map）需要一个 assay 可执行文件："
    Write-Bad "     · 装它，或设 ASSAY_ENGINE_BIN 指向一个 assay.exe"
    Write-Bad "     · 或把引擎源码放进 engine\（见 assay 仓库）"
    Write-Bad "   **账本动词不依赖引擎，仍然可用**：gate / lint / bridge / digest / trace / ledger / spec / wp"
    exit 127
}

function Invoke-Engine([string[]]$EngineArgs) {
    $exe = Get-EnginePath
    & $exe @EngineArgs '--root' $Root
    exit $LASTEXITCODE
}

# ── 分派 ────────────────────────────────────────────────────────────────────

$verb = if ($Args.Count -gt 0) { $Args[0] } else { $null }
$rest = if ($Args.Count -gt 1) { $Args[1..($Args.Count - 1)] } else { @() }

$engineVerbs = @('run', 'plan', 'map', 'check', 'doctor', 'next', 'version')

switch ($verb) {
    { $_ -in $engineVerbs } { Invoke-Engine (@($verb) + $rest) }
    'help' {
        Write-Host "assay —— 唯一入口（引擎 + 账本工具）"
        Write-Host ""
        Write-Host "  引擎（Rust，不需要 Python）"
        Write-Host "    doctor              环境与就绪度；**不跑任何检查**"
        Write-Host "    next                现在该做什么（有序动作表）"
        Write-Host "    check               清单自检（门禁；不跑检查）"
        Write-Host "    run [--stage s] [--check id] [--change id] [--certify]"
        Write-Host "    plan --base <rev> [--format json]"
        Write-Host "    map                 由清单机械生成 MAP.md"
        Write-Host ""
        Write-Host "  账本工具（Python）"
        Write-Host "    gate                四合一提交门（lint/桥/digest/自测）"
        Write-Host "    lint                账目平衡 L1/L2/L3"
        Write-Host "    bridge              模型↔代码桥对账"
        Write-Host "    digest [--check]    重新生成 / 校验 LEDGER.md"
        Write-Host "    trace <G|D|O|键>    影响分析"
        Write-Host "    ledger <O-id>       复跑某义务的检查器（--list 列映射）"
        Write-Host "    spec <init|expand|check> ..."
        Write-Host "    wp <D-id>           从已裁决决策生成 AI 工作包"
        Write-Host "    bootstrap           准备/自检 Python 侧（缺 PyYAML 时现场建 tools/.venv）"
        Write-Host ""
        Write-Host "  其他"
        Write-Host "    py <脚本> [参数]    用解析到的 Python 跑任意脚本"
        Write-Host "    -- <命令>           原样执行"
        exit 0
    }
    'gate' { Invoke-PythonTool @((Join-Path $Root 'tools\gate.py')) }
    'lint' { Invoke-PythonTool @((Join-Path $Root 'tools\ledger_lint.py')) }
    'bridge' { Invoke-PythonTool @((Join-Path $Root 'tools\bridge_check.py')) }
    'digest' { Invoke-PythonTool (@((Join-Path $Root 'tools\digest.py')) + $rest) }
    'trace' { Invoke-PythonTool (@((Join-Path $Root 'tools\trace.py')) + $rest) }
    'ledger' { Invoke-PythonTool (@((Join-Path $Root 'tools\ledger_check.py')) + $rest) }
    'spec' { Invoke-PythonTool (@((Join-Path $Root 'tools\spec.py')) + $rest) }
    'wp' { Invoke-PythonTool (@((Join-Path $Root 'tools\prepare_work_package.py')) + $rest) }
    'py' { Invoke-PythonTool $rest }
    'bootstrap' {
        Write-Host "== assay bootstrap =="
        $prefix = @(Resolve-PythonPrefix)
        if ($prefix.Count -eq 0 -or -not $prefix[0]) {
            Write-Bad "❌ 无法准备 Python 侧；先装 uv：https://docs.astral.sh/uv/"
            exit 127
        }
        $exe = $prefix[0]
        $pre = if ($prefix.Count -gt 1) { $prefix[1..($prefix.Count - 1)] } else { @() }
        Write-Host "解释器：$exe"
        & $exe @pre '-c' 'import sys,yaml;print("python", sys.version.split()[0], "| yaml", yaml.__version__)'
        Write-Host "引擎：$(Get-EnginePath)"
        Write-Host "提示：让 pre-commit 真正生效需要 git config core.hooksPath .githooks"
        exit 0
    }
    '--' {
        if ($rest.Count -eq 0) { Write-Bad "用法：assay -- <命令>"; exit 2 }
        & $rest[0] @($rest[1..($rest.Count - 1)])
        exit $LASTEXITCODE
    }
    default {
        if (-not $verb) {
            Write-Host "assay —— 用 `assay help` 看动词表" -ForegroundColor Yellow
            exit 0
        }
        Write-Bad "❌ 未知动词 `"$verb`"。可用：$($engineVerbs -join ' / ') / gate / lint / bridge / digest / trace / ledger / spec / wp / py / help"
        exit 2
    }
}
