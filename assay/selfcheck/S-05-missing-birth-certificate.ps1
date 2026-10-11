# S-05 — 缺出生证明的检查必须被单列，且不计入棘轮
# 注入：一条没有 seen_red 的检查（它绿，但没人有理由信它）。
# 红 = 没有出生证明的检查被当成了证据（棘轮收了它，或地图里不再单列）
. "$PSScriptRoot\_lib.ps1"

$manifest = @'
[assay]
protocol = "v0.7"

[[claim]]
id = "C-5"
text = "fixture claim"
rejected_by = "fixture"
origin = "fixture"
capability = "simulate"
strength_required = 5
scope = "sampled"

[[check]]
id = "K-5-NOPROOF"
claims = ["C-5"]
capability = "simulate"
strength = 5
scope = "sampled"
command = "exit 0"
stage = "pr"
budget_s = 5
'@

$root = New-Fixture 'unproven' $manifest
Initialize-Fixture $root

# 1) 地图里必须单列（在 unproven 小节之内）
$map = Get-Content (Join-Path $root 'assay\MAP.md') -Raw
$idx = $map.IndexOf('## unproven')
if ($idx -lt 0) { Fail "MAP.md 没有 unproven 小节。指针：$root\assay\MAP.md" }
if ($map.Substring($idx) -notmatch 'K-5-NOPROOF') {
    Fail "MAP.md 的 unproven 小节没有列出 K-5-NOPROOF。指针：$root\assay\MAP.md"
}

# 2) 跑一趟：它绿了，但棘轮不许收它
$r = Invoke-Engine $root @('run')
$ratchetPath = Join-Path $root 'assay\ratchet.toml'
$ratchet = if (Test-Path $ratchetPath) { Get-Content $ratchetPath -Raw } else { '' }
if ($ratchet -match 'K-5-NOPROOF') {
    Fail "没有出生证明的检查进了棘轮。指针：$ratchetPath"
}
Ok "无出生证明的检查被单列、且未进棘轮（run exit=$($r.code)）"
