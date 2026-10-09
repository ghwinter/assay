# S-02 — 引擎源码零后端工具名
# 红 = 引擎开始认识某个具体工具（那它就不再是"哑执行器"，协议 §8 的边界被破坏）
. "$PSScriptRoot\_lib.ps1"

$src = Join-Path (Get-Root) 'engine\src'
if (-not (Test-Path $src)) { Fail "找不到 $src" }

$hits = Get-ChildItem -Path $src -Filter *.rs -Recurse |
    Select-String -Pattern 'verus|quint|apalache|capella|tlaplus|tla\+|alloy|lean4|\btlc\b' -CaseSensitive:$false
if ($hits) {
    Fail ("引擎源码点名了后端工具（{0} 处）。最小反例：{1}" -f $hits.Count,
        (($hits | Select-Object -First 3 | ForEach-Object { "$($_.Filename):$($_.LineNumber):$($_.Line.Trim())" }) -join ' ｜ '))
}
Ok "engine/src 零后端工具名（0 命中）"
