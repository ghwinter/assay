# S-08 — 无界声明不能被有界检查兑现（v0.2 覆盖范围规则）
# 注入：声明说"无界"，检查其实只覆盖 ≤25 步。
# 红 = 有界检查冒充穷举/证明（这正是 quint-01 实测抓到的那次强度通胀）
. "$PSScriptRoot\_lib.ps1"

$manifest = @'
[assay]
protocol = "v0.7"

[[claim]]
id = "C-8"
text = "fixture claim: unbounded"
rejected_by = "fixture"
origin = "fixture"
capability = "exhaustive"
strength_required = 6
scope = "unbounded"

[[check]]
id = "K-8"
claims = ["C-8"]
capability = "exhaustive"
strength = 6
scope = "bounded"
bound = 25
command = "exit 0"
stage = "pr"
budget_s = 5
seen_red = "fixture injected: --max-steps 25"
'@

$root = New-Fixture 'scope' $manifest
Initialize-Fixture $root
$r = Invoke-Engine $root @('check')
Assert-Red '有界检查兑现无界声明' $r.out $r.code '无界'
Ok "有界检查未能冒充穷举（exit=$($r.code)）"
