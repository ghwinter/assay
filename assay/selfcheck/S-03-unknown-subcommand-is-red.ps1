# S-03 — 未知子命令必须非零退出，且不产出任何判定
# 红 = 引擎对不认识的东西静默通过（那它就从门禁退化成装饰）
. "$PSScriptRoot\_lib.ps1"

$r = Invoke-Engine (Get-Root) @('frobnicate')

if ($r.code -eq 0) { Fail "未知子命令 exit=0。输出：$($r.out)" }
# 判据与措辞无关：报错必须**点名**它不认识的东西。（引擎换成 clap 后文案变了，
# 原先断言中文字样，于是这条检查在两个引擎版本里静默失效——见 D-033。）
if ($r.out -notmatch 'frobnicate') { Fail "未知子命令报错时没有点名它不认识的东西。输出：$($r.out)" }
if ($r.out -match '^\s*\[(PASS|FAIL|SKIP)\]') { Fail "未知子命令仍然产出了判定。输出：$($r.out)" }
Ok "未知子命令 exit=$($r.code)，说明原因且不产出判定"
