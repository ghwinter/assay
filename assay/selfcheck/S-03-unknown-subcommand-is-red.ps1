# S-03 — 未知子命令必须非零退出，且不产出任何判定
# 红 = 引擎对不认识的东西静默通过（那它就从门禁退化成装饰）
. "$PSScriptRoot\_lib.ps1"

$r = Invoke-Engine (Get-Root) @('frobnicate')

if ($r.code -eq 0) { Fail "未知子命令 exit=0。输出：$($r.out)" }
if ($r.out -notmatch '未知子命令') { Fail "未知子命令没有说明原因。输出：$($r.out)" }
if ($r.out -match '^\s*\[(PASS|FAIL|SKIP)\]') { Fail "未知子命令仍然产出了判定。输出：$($r.out)" }
Ok "未知子命令 exit=$($r.code)，说明原因且不产出判定"
