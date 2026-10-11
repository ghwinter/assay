# S-07 — 手改地图即红灯
# 注入：往 MAP.md 追加一个字节（地图是机械生成的，不是文档）。
# 红 = 地图可以被手改而不被发现（那它就变成了又一个会腐烂的中心真相）
. "$PSScriptRoot\_lib.ps1"

$manifest = @'
[assay]
protocol = "v0.7"

[[claim]]
id = "C-7"
text = "fixture claim"
rejected_by = "fixture"
origin = "fixture"
capability = "simulate"
strength_required = 5
scope = "sampled"

[[check]]
id = "K-7"
claims = ["C-7"]
capability = "simulate"
strength = 5
scope = "sampled"
command = "exit 0"
stage = "pr"
budget_s = 5
seen_red = "fixture injected"
'@

$root = New-Fixture 'mapdrift' $manifest
Initialize-Fixture $root
$r1 = Invoke-Engine $root @('check')
Assert-Green '干净 fixture' $r1.out $r1.code

Add-Content -Path (Join-Path $root 'assay\MAP.md') -Value '<!-- 手工补的一句话 -->' -Encoding utf8

$r2 = Invoke-Engine $root @('check')
Assert-Red '地图被手改' $r2.out $r2.code 'MAP.md'
Ok "地图漂移被拦（exit=$($r2.code)）"
