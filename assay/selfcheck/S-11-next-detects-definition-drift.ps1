# S-11 — next 必须报出"证据与它当前的检查定义不符"，且**不得**退回整份清单指纹
#
# 这条检查判两件不同的事，缺一不可：
#   ① 正向：检查命令改了 ⇒ 旧证据不再对得上 ⇒ 报 stale 并点名该检查
#   ② 反向：检查定义**没变**时，清单文本里改一个无关字节**不得**报 stale
#      （否则全部证据天天报陈旧，人开始无视它——那比不报更糟）
#
# 反向那一半是这条检查真正的价值：它挡住"拿整份清单指纹当失效判据"这个错法。
# 实现上刻意**不重新实现 FNV**：先让引擎自己跑一次拿到真实指纹，
# 再按语义去改/不改清单。指纹算法换了，这条检查照样成立。
. "$PSScriptRoot\_lib.ps1"

$manifest = @'
[assay]
protocol = "v0.7"

[[claim]]
id = "C-nx"
text = "fixture claim"
rejected_by = "fixture"
capability = "structural"
strength_required = 5
origin = "fixture"

[[check]]
id = "K-nx"
claims = ["C-nx"]
capability = "structural"
strength = 5
scope = "unbounded"
command = "git --version"
stage = "pr"
budget_s = 5
seen_red = "fixture"
'@

$root = New-Fixture 'next-stale' $manifest
Initialize-Fixture $root

# 让引擎自己产生一份真证据（也顺带证明 run 在 fixture 上可用）
$run = Invoke-Engine $root @('run', '--check', 'K-nx')
if ($run.code -ne 0) { Fail "fixture 的 run 失败（exit=$($run.code)）：$($run.out)" }

# ② 反向：改清单文本里**与检查无关**的部分（加一行注释）⇒ 不得报 stale
$mp = Join-Path $root 'assay\manifest.toml'
Add-Content -Path $mp -Value "`n# S-11：与检查定义无关的改动" -Encoding utf8
$r2 = Invoke-Engine $root @('next')
if ($r2.code -ne 0) { Fail "next 是只读导览，不该非零退出（exit=$($r2.code)）：$($r2.out)" }
# 只断言**陈旧**这一路：fixture 的检查本来就没出生证明，[birth] K-nx 是正当输出，
# 不能拿"输出里有没有 K-nx"当判据（那样这条检查会因为无关原因红）。
if ($r2.out -match '\[stale\]\s+K-nx') {
    Fail "检查定义未变，next 却因清单文本变了而报 K-nx 陈旧——这正是要防的错法。输出：$($r2.out)"
}

# ① 正向：改掉检查命令 ⇒ 旧证据失效，必须点名并说清是哪种失效
(Get-Content $mp -Raw) -replace 'command = "git --version"', 'command = "git --version --porcelain"' |
    Set-Content -Path $mp -Encoding utf8 -NoNewline
$r1 = Invoke-Engine $root @('next')
if ($r1.out -notmatch '\[stale\]\s+K-nx') {
    Fail "检查命令已改，next 却没有把 K-nx 标为陈旧。输出：$($r1.out)"
}
if ($r1.out -notmatch '检查命令已改') {
    Fail "报到了 K-nx 陈旧，但没说清是哪种失效。输出：$($r1.out)"
}

Ok "定义没变不误报（②）；命令改了就点名并说明原因（①）"
