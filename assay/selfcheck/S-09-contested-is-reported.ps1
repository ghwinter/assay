# S-09 — 同一条声明上出现分歧时，绿的那条不得被读成"该声明成立"
# 注入：一条声明上挂两条检查，一条绿一条红（同一个不变量的两个后端各写一遍，互不知情）。
# 红 = 引擎把绿的那条当成"声明成立"，分歧被吞掉（真实摩擦：两个实验台写了同一个窗口不变量，
#      没有东西检查它们说的是不是同一件事）
. "$PSScriptRoot\_lib.ps1"

$manifest = @'
[[claim]]
id = "C-9"
text = "fixture claim (contested)"
rejected_by = "fixture"
capability = "simulate"
strength_required = 5
scope = "sampled"

[[check]]
id = "K-9A"
claims = ["C-9"]
capability = "simulate"
strength = 5
scope = "sampled"
command = "exit 0"
stage = "pr"
budget_s = 5
seen_red = "fixture"

[[check]]
id = "K-9B"
claims = ["C-9"]
capability = "differential"
strength = 5
scope = "sampled"
command = "exit 1"
stage = "pr"
budget_s = 5
seen_red = "fixture"
'@

$root = New-Fixture 'contested' $manifest
Initialize-Fixture $root
$r = Invoke-Engine $root @('run')

if ($r.out -notmatch '分歧') {
    Fail "同一声明上绿红并存，引擎没有报分歧。输出：$($r.out)"
}
if ($r.out -notmatch 'K-9A' -or $r.out -notmatch 'K-9B') {
    Fail "分歧没有点名是哪两条检查。输出：$($r.out)"
}
if ($r.code -eq 0) {
    Fail "有检查红了，run 却 exit=0。输出：$($r.out)"
}
Ok "分歧被报出并点名（K-9A 绿 / K-9B 红），run exit=$($r.code)"
