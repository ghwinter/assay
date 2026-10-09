# S-04 — 档 4（调试期断言）必须被清单自检拒绝
# 注入：往 fixture 里放一条 strength=4 的检查，要求门禁说"不"。
# 红 = 引擎放行了档 4（发布版里根本不存在的判定被当成了证据）
. "$PSScriptRoot\_lib.ps1"

$manifest = @'
[[claim]]
id = "C-4"
text = "fixture claim"
rejected_by = "fixture"
capability = "proof"
strength_required = 4
scope = "unbounded"

[[check]]
id = "K-4"
claims = ["C-4"]
capability = "proof"
strength = 4
scope = "unbounded"
command = "exit 0"
stage = "pr"
budget_s = 5
seen_red = "fixture injected"
'@

$root = New-Fixture 'strength4' $manifest
Initialize-Fixture $root
$r = Invoke-Engine $root @('check')
Assert-Red '档 4 检查' $r.out $r.code '档 4'
Ok "档 4 检查被拒（exit=$($r.code)）"
