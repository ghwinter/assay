# S-12 — L1 的产物全集由 git 定义（.gitignore 说了算），不由文件系统 glob 定义
#
# 真实摩擦（这条检查就是它买来的）：
#   L1 的 CODE_PATTERNS 里有 `tools/**/*.py`，而它直接 root.glob，不看 .gitignore。
#   于是一个被忽略的本地虚拟环境 `tools/.venv/` 里几百个 site-packages 文件
#   全被认成"无主产物"，L1 爆红。
#   后果不是"噪音多"，而是 **L1 的判定取决于这台机器上恰好存在哪些未跟踪文件**——
#   同一份提交在两台机器上可以一红一绿，与"证据可复跑"直接冲突。
#
# 判两半，缺一不可（都落在**已存在的忽略规则**上，因此不修改 .gitignore）：
#   ① 未跟踪且未忽略的新文件仍在作用域内（"提交前抓新文件"这一半不能丢）
#   ② 命中既有忽略规则的文件移出作用域（此处用 .gitignore 里的 `__pycache__/`）
. "$PSScriptRoot\_lib.ps1"

$root = Get-Root
$hot = Join-Path $root 'tools\l1-scope-probe.py'                 # 未忽略
$cold = Join-Path $root 'tools\__pycache__\l1-scope-probe.py'    # 被 __pycache__/ 忽略
$script = if ($env:ASSAY_TOOL_OVERRIDE) { $env:ASSAY_TOOL_OVERRIDE } else { Join-Path $root 'tools\ledger_lint.py' }

function Invoke-Lint {
    $cli = Join-Path $root 'tools\assay-cli.ps1'
    $out = & pwsh -NoProfile -File $cli py $script 2>&1 | Out-String
    return @{ code = $LASTEXITCODE; out = $out }
}

# 先确认前提成立：__pycache__/ 确实是忽略规则。前提不成立就明说，不许静默通过。
$ignored = & git -C $root check-ignore --no-index 'tools/__pycache__/x.py' 2>&1 | Out-String
if ($ignored.Trim() -eq '') {
    Fail "前提不成立：.gitignore 里没有能覆盖 tools/__pycache__/ 的规则，本检查无法判定"
}

try {
    New-Item -ItemType Directory -Force -Path (Split-Path $cold) | Out-Null
    Set-Content -Path $hot -Value "# l1 scope probe`n" -Encoding utf8
    Set-Content -Path $cold -Value "# l1 scope probe`n" -Encoding utf8

    $r = Invoke-Lint
    if ($r.out -notmatch 'tools/l1-scope-probe\.py') {
        Fail "未跟踪且未忽略的新文件没有被 L1 认成产物——『提交前抓新文件』这一半丢了。输出：$($r.out)"
    }
    if ($r.out -match 'l1-scope-probe\.py' -and $r.out -match '__pycache__') {
        Fail "被 .gitignore 忽略的文件仍被当成产物——作用域没跟着 git 走。输出：$($r.out)"
    }
} finally {
    Remove-Item $hot -Force -ErrorAction SilentlyContinue
    Remove-Item $cold -Force -ErrorAction SilentlyContinue
}

Ok "未忽略的新文件在作用域内（①）；命中忽略规则的文件移出作用域（②）"
