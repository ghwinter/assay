# S-10 — doctor 必须认出"声明的命令解析不到"
#
# 真实摩擦（这条检查就是它买来的）：
#   AGENTS.md 写着 `python tools/gate.py`，而本机上 PATH 的 python 是
#   Microsoft Store 的 App Execution Alias 占位符——命令"写得出来"，锚点不存在。
#   四道门因此一次都没跑过，而 `assay check` 仍然全绿。
#
# 红 = 清单声明了一个首词解析不到的命令，而 doctor 没报出来。
# 判据分两半：① 退出码非零；② 输出里点名了那个词（"红了但不是因为预期原因"不算数）。
. "$PSScriptRoot\_lib.ps1"

$manifest = @'
[assay]
protocol = "v0.7"

[[claim]]
id = "C-doc"
text = "fixture claim"
rejected_by = "fixture"
capability = "structural"
strength_required = 5
origin = "fixture"

[[check]]
id = "K-doc"
claims = ["C-doc"]
capability = "structural"
strength = 5
scope = "unbounded"
command = "assay-command-that-does-not-exist-xyzzy --run"
stage = "pr"
budget_s = 5
seen_red = "fixture"
'@

$root = New-Fixture 'doctor-unresolvable' $manifest
$r = Invoke-Engine $root @('doctor')
Assert-Red 'doctor 应当因命令解析不到而红灯' $r.out $r.code 'commands-resolvable'
if ($r.out -notmatch 'assay-command-that-does-not-exist-xyzzy') {
    Fail "doctor 红了，但没有点名解析不到的那个词。输出：$($r.out)"
}

# 反向：把命令改成一个真实存在的可执行文件首词，就不该因这一条红。
# （没有这个反向，一条"凡有命令即红"的坏检查也能过。）
$ok = New-Fixture 'doctor-resolvable' ($manifest -replace 'assay-command-that-does-not-exist-xyzzy', 'git')
$r2 = Invoke-Engine $ok @('doctor')
if ($r2.out -match 'commands-resolvable.*解析不到') {
    Fail "命令首词是 git 却被判为解析不到——检查过严，等于凡有命令即红。输出：$($r2.out)"
}

Ok "解析不到的命令被抓出且点名（exit=$($r.code)）；可解析的命令不误伤"
