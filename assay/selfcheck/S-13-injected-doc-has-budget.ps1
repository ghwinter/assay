# S-13 — 常驻注入的 AGENTS.md 有预算，且不驻留机器相关的命令
#
# 理由（一次实测买来的）：
#   AGENTS.md 是**常驻注入**的——每一行都在花别人的上下文。没有上限的规则文件只会单向变长：
#   "加一条规则"的收益当场可见，"上下文变贵"的成本要等到会话开始漂移才显形。
#   同一份文件里曾驻着一行解释器命令，而那个解释器在本机执行即失败：
#   声明还在、锚点没了，四道门空转了两个引擎版本，而状态板一路全绿。
#
# 判两半：
#   ① 字符数 ≤ $Ceiling。上限**只在这里改一处**，且提额必须在同一次改动的
#      `[[change]].reason` 里写出理由——所以 AGENTS.md 里不写这个数字（会腐化的数字不进口径文件）。
#   ② 不出现机器相关的命令头（解释器/构建器）。判据刻意窄：只管"命令头"，
#      不管阅读指针（`ledger/`、`thinking/blind_spots.md` 这类路径是合法的）。
. "$PSScriptRoot\_lib.ps1"

$Ceiling = 6000
$root = Get-Root
$doc = Join-Path $root 'AGENTS.md'
if (-not (Test-Path $doc)) { Fail "找不到 $doc" }

$text = [System.IO.File]::ReadAllText($doc)

# ① 预算
$n = $text.Length
if ($n -gt $Ceiling) {
    $msg = "AGENTS.md 超预算：$n 字符 > 上限 $Ceiling。按序处理：" +
           "先搬走（挪进 RUNBOOK.md 或对应账）→ 再压缩 → 最后才提高 `$Ceiling`，" +
           "且必须在同一次改动的 [[change]].reason 里写出提额理由。"
    Fail $msg
}

# ② 不驻留机器相关的命令头
$heads = 'python', 'python3', 'pwsh', 'powershell', 'bash', 'cargo', 'java', 'node', 'npm'
$lines = $text -split "`r?`n"
$hits = @()
for ($i = 0; $i -lt $lines.Count; $i++) {
    foreach ($h in $heads) {
        if ($lines[$i] -match "(?:^|[\s``])$([regex]::Escape($h))\s") {
            $hits += ("AGENTS.md:{0}: {1}" -f ($i + 1), $lines[$i].Trim())
            break
        }
    }
}
if ($hits.Count -gt 0) {
    Fail ("AGENTS.md 出现机器相关命令头（{0} 处）。最小反例：{1}" -f $hits.Count, ($hits -join ' ｜ '))
}

Ok ("AGENTS.md {0}/{1} 字符（余量 {2}），零机器相关命令头" -f $n, $Ceiling, ($Ceiling - $n))
