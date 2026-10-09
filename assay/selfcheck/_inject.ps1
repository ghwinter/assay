# _inject.ps1 — 出生证明的机械复现装置
#
# 协议 §5 说出生证明是"本协议最重要的一条"。但最初它只是一个**自述字符串**：
# `seen_red = "我注入过 X，看到了 Y"` —— 引擎无法验证这句话。
# 狗粮（assay 给自己写清单）当场暴露了这个洞：最重要的规则，是唯一完全靠自觉的。
#
# 修法：`red_command` —— 一条**期望失败**的命令（§5 的字面要求：亲眼看到它失败）。
#
#   ⚠ 退出码约定（这一条是全部机关所在）：
#      red_command 退出 **非零** ⇔ 证书成立 —— 因为"能失败"本身就是我们要证明的那件事。
#      red_command 退出 **0**    ⇔ 证书不成立。
#   **所有"没证明出来"的情况一律归到 0**（检查没红 / 注入点失效 / 内部报错），
#   这样"注入器坏了"不会被误读成"证书成立"。
#
# 本文件给那些"红不了就要改引擎"的检查用：
#   复制 crate → 打一个必须恰好命中一次的字面量补丁 → 编译到共享 target 目录 →
#   用这个**故意做坏的引擎**跑检查 → 要求它转红。
# 不碰工作树、不碰真引擎（也避开 Windows 对运行中 exe 的独占锁）。

. "$PSScriptRoot\_lib.ps1"

$ErrorActionPreference = 'Stop'

function Cert-Proven([string]$msg) { Write-Output "🎓 证书成立：$msg"; exit 1 }
function Cert-Unproven([string]$msg) { Write-Output "❌ 证书不成立：$msg"; exit 0 }

# A 类：往一个**文档/源文件**里注入（不需要重新编译）
function Invoke-DocInjection {
    param(
        [string]$Name,
        [string]$File,          # 相对 root
        [string]$Append,
        [string]$CheckScript    # relative to assay\selfcheck
    )
    $root = Get-Root
    $p = Join-Path $root $File
    if (-not (Test-Path $p)) { Cert-Unproven "注入目标不存在：$p" }
    $bak = Get-Content $p -Raw
    try {
        Add-Content -Path $p -Value $Append -Encoding utf8
        $out = & pwsh -NoProfile -File (Join-Path $root "assay\selfcheck\$CheckScript") 2>&1 | Out-String
        $code = $LASTEXITCODE
    } catch {
        Cert-Unproven "注入过程报错：$_"
    } finally {
        Set-Content -Path $p -Value $bak -Encoding utf8 -NoNewline
    }
    if ($code -eq 0) {
        Cert-Unproven "$Name：注入后检查仍然绿 ⇒ 这条检查红不了。输出：$out"
    }
    $ce = ($out -split "`r?`n" | Where-Object { $_ -match 'FAIL' } | Select-Object -First 1)
    Cert-Proven "$Name：注入后检查转红（exit=$code）。最小反例：$ce"
}

# B 类：把**引擎**做坏（需要重新编译一个副本）
function Invoke-EngineInjection {
    param(
        [string]$Name,
        [string]$Find,          # 必须恰好出现一次的字面量
        [string]$Replace,
        [string]$CheckScript
    )
    $root = Get-Root
    $tmp = Join-Path $env:TEMP "assay-inject-$Name"
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
    New-Item -ItemType Directory -Force -Path (Join-Path $tmp 'src') | Out-Null

    # 1) 复制 crate；改包名，避免产物覆盖真引擎
    $cargo = Get-Content (Join-Path $root 'engine\Cargo.toml') -Raw
    $cargo = $cargo -replace '(?m)^name = "assay"', 'name = "assay-inject"'
    Set-Content -Path (Join-Path $tmp 'Cargo.toml') -Value $cargo -Encoding utf8
    Copy-Item -Path (Join-Path $root 'engine\src\*') -Destination (Join-Path $tmp 'src') -Recurse

    # 2) 打补丁：注入点必须恰好命中一次，否则注入器自己要被修
    $target = Join-Path $tmp 'src\main.rs'
    $src = Get-Content $target -Raw
    $n = ([regex]::Matches($src, [regex]::Escape($Find))).Count
    if ($n -ne 1) {
        Cert-Unproven "$Name：注入点出现 $n 次（必须恰好 1 次）。引擎改了，注入器要跟着改：$Find"
    }
    Set-Content -Path $target -Value $src.Replace($Find, $Replace) -Encoding utf8 -NoNewline

    # 3) 编译到一个**共享**的 target 目录：不同补丁复用依赖产物，单次只需重编本 crate。
    #    （各自独立 target 目录会让依赖重编 6 遍，秒级变分钟级。）
    $td = Join-Path $env:TEMP 'assay-inject-target'
    New-Item -ItemType Directory -Force -Path $td | Out-Null
    $exe = Join-Path $td 'release\assay-inject.exe'
    # 先删旧产物：否则编译失败时会**回退到上一次注入版**，把"上一次的缺陷"读成"这一次的证书"。
    # （这正是它要防的那类错误：陈旧证据被当成新鲜证据。）
    Remove-Item $exe -Force -ErrorAction SilentlyContinue
    $build = & cargo build --release --offline --manifest-path (Join-Path $tmp 'Cargo.toml') --target-dir $td 2>&1 | Out-String
    if (-not (Test-Path $exe)) { Cert-Unproven "$Name：注入版编译失败：$build" }

    # 4) 用注入版跑检查，要求它红
    $env:ASSAY_ENGINE = $exe
    try {
        $out = & pwsh -NoProfile -File (Join-Path $root "assay\selfcheck\$CheckScript") 2>&1 | Out-String
        $code = $LASTEXITCODE
    } finally {
        Remove-Item Env:\ASSAY_ENGINE -ErrorAction SilentlyContinue
    }
    if ($code -eq 0) {
        Cert-Unproven "$Name：注入缺陷后检查仍然绿 ⇒ 这条检查红不了。输出：$out"
    }
    $ce = ($out -split "`r?`n" | Where-Object { $_ -match 'FAIL' } | Select-Object -First 1)
    Cert-Proven "$Name：注入缺陷后检查转红（exit=$code）。最小反例：$ce"
}
