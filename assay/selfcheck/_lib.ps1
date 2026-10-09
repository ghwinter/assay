# _lib.ps1 — selfcheck 共用件（被各检查脚本 dot-source）
# 约定：退出码 0 = pass，非 0 = fail；失败必须打印「最小反例 + 指针」（协议 §2.4）

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = 'Stop'

function Get-Root { return (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path }

function Get-Engine {
    # 注入用：ASSAY_ENGINE 可把检查指向一个**故意做坏的引擎副本**。
    # 这是出生证明从"自述"变成"可复现"的机关——见 _inject.ps1。
    if ($env:ASSAY_ENGINE) {
        if (-not (Test-Path $env:ASSAY_ENGINE)) { Fail "ASSAY_ENGINE 指向的文件不存在：$env:ASSAY_ENGINE" }
        return $env:ASSAY_ENGINE
    }
    $exe = Join-Path (Get-Root) 'engine\target\release\assay.exe'
    if (-not (Test-Path $exe)) { Fail "引擎未构建：$exe —— 先跑 cargo build --release" }
    return $exe
}

function Fail([string]$msg) { Write-Output "FAIL: $msg"; exit 1 }
function Ok([string]$msg) { Write-Output "PASS: $msg"; exit 0 }

# 把一条命名的 fixture 建在 %TEMP% 下，返回它的 root
function New-Fixture([string]$name, [string]$manifest) {
    $d = Join-Path $env:TEMP "assay-fixture-$name"
    if (Test-Path $d) { Remove-Item -Recurse -Force $d }
    New-Item -ItemType Directory -Force -Path (Join-Path $d 'assay') | Out-Null
    Set-Content -Path (Join-Path $d 'assay\manifest.toml') -Value $manifest -Encoding utf8
    return $d
}

# 跑引擎；返回 @{ code; out }
function Invoke-Engine([string]$root, [string[]]$argv) {
    $exe = Get-Engine
    $full = @($argv) + @('--root', $root)
    $out = & $exe @full 2>&1 | Out-String
    return @{ code = $LASTEXITCODE; out = $out }
}

# 干净 fixture 的标准动作：map（消除地图漂移这个无关红灯）
function Initialize-Fixture([string]$root) {
    $r = Invoke-Engine $root @('map')
    if ($r.code -ne 0) { Fail "fixture 的 map 失败（exit=$($r.code)）：$($r.out)" }
}

function Assert-Red([string]$label, [string]$out, [int]$code, [string]$mustMatch) {
    if ($code -eq 0) { Fail "$label：应当红灯，实际 exit=0。输出：$out" }
    if ($mustMatch -and ($out -notmatch $mustMatch)) {
        Fail "$label：红灯了，但不是因为预期的原因（未匹配 /$mustMatch/）。输出：$out"
    }
}

function Assert-Green([string]$label, [string]$out, [int]$code) {
    if ($code -ne 0) { Fail "$label：应当绿灯，实际 exit=$code。输出：$out" }
}
