# S-01 — 协议文档零后端工具名
# 红 = PROTOCOL.md 里点名了一个具体后端工具（"引擎不认识工具"就从结构保证退化成自觉）
. "$PSScriptRoot\_lib.ps1"

$p = Join-Path (Get-Root) 'PROTOCOL.md'
if (-not (Test-Path $p)) { Fail "找不到 $p" }

$hits = @(Select-String -Path $p -Pattern 'verus|quint|apalache|capella|tlaplus|tla\+|alloy|lean4|\btlc\b' -CaseSensitive:$false)
if ($hits.Count -gt 0) {
    $ce = ($hits | Select-Object -First 3 | ForEach-Object { "$($_.LineNumber):$($_.Line.Trim())" }) -join ' ｜ '
    Fail "协议点名了后端工具（$($hits.Count) 处）。最小反例：$ce"
}
Ok "PROTOCOL.md 零后端工具名（0 命中；只描述能力，不描述工具）"
