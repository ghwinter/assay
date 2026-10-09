# S-06 — 已入棘轮的检查不能被静默削弱
# 注入：一条已兑现的检查把强度从 6 降到 5（不写 waiver）。
# 红 = 削弱被放过（"已绿的检查不许被删弱"从机械退回自觉）
. "$PSScriptRoot\_lib.ps1"

$v1 = @'
[[claim]]
id = "C-6"
text = "fixture claim"
rejected_by = "fixture"
capability = "proof"
strength_required = 6
scope = "unbounded"

[[check]]
id = "K-6"
claims = ["C-6"]
capability = "proof"
strength = 6
scope = "unbounded"
command = "exit 0"
stage = "pr"
budget_s = 5
seen_red = "fixture injected"
'@

$root = New-Fixture 'ratchet' $v1
Initialize-Fixture $root
$r1 = Invoke-Engine $root @('run')
Assert-Green 'fixture 首跑' $r1.out $r1.code
$r2 = Invoke-Engine $root @('check')
Assert-Green '棘轮已建立、清单自检' $r2.out $r2.code
$ratchet = Get-Content (Join-Path $root 'assay\ratchet.toml') -Raw
if ($ratchet -notmatch 'K-6') { Fail "fixture 前置条件不成立：棘轮里没有 K-6。指针：$root\assay\ratchet.toml" }

# 注入削弱：强度 6 → 5，声明要求同步降到 5（这样"覆盖"不是红灯，只有棘轮是）
$v2 = $v1.Replace('strength_required = 6', 'strength_required = 5').Replace('strength = 6', 'strength = 5')
Set-Content -Path (Join-Path $root 'assay\manifest.toml') -Value $v2 -Encoding utf8
Initialize-Fixture $root   # 重生成地图，让地图漂移不是本次红灯的原因

$r3 = Invoke-Engine $root @('check')
Assert-Red '棘轮削弱' $r3.out $r3.code '强度 6→5'
Ok "强度下调被拦（exit=$($r3.code)）"
