# =====================================================================
#  build.ps1 — build both language variants of
#  "Compression and Verification"
#
#  Usage:  pwsh -File build.ps1            # build both
#          pwsh -File build.ps1 -Lang en    # build English only
#          pwsh -File build.ps1 -Lang zh    # build Chinese only
#
#  The English build (en/) is the authoritative text.
#  The Chinese build (zh/) is a controlled rendering; the term mapping
#  is fixed in TERMINOLOGY.md.
# =====================================================================
param(
    [ValidateSet('both', 'en', 'zh')]
    [string]$Lang = 'both'
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path

# ---- locate xelatex -------------------------------------------------
if (-not (Get-Command xelatex -ErrorAction SilentlyContinue)) {
    $candidates = @(
        'D:\Programs\MiKTeX\miktex\bin\x64',
        (Join-Path $env:LOCALAPPDATA 'Programs\MiKTeX\miktex\bin\x64'),
        'C:\Program Files\MiKTeX\miktex\bin\x64',
        'C:\Program Files (x86)\MiKTeX\miktex\bin\x64',
        'C:\texlive\2024\bin\windows',
        'C:\texlive\2025\bin\windows'
    )
    foreach ($c in $candidates) {
        if (Test-Path (Join-Path $c 'xelatex.exe')) {
            $env:PATH = "$c;$env:PATH"
            Write-Host "xelatex found at $c" -ForegroundColor DarkGray
            break
        }
    }
}
if (-not (Get-Command xelatex -ErrorAction SilentlyContinue)) {
    throw 'xelatex not found. Install TeX Live or MiKTeX, or add its bin directory to PATH.'
}

# ---- target set -----------------------------------------------------
$targets = switch ($Lang) {
    'en'    { @('en') }
    'zh'    { @('zh') }
    default { @('en', 'zh') }
}

foreach ($t in $targets) {
    $dir = Join-Path $root $t
    if (-not (Test-Path $dir)) { Write-Warning "missing directory: $dir"; continue }

    Push-Location $dir
    # `build` must be a DIRECTORY; guard against a stray file of that name.
    if ((Test-Path 'build') -and -not (Get-Item 'build').PSIsContainer) {
        Remove-Item 'build' -Force
    }
    New-Item -ItemType Directory -Force -Path 'build' | Out-Null

    Write-Host "=== [$t] xelatex x3 ===" -ForegroundColor Cyan
    foreach ($i in 1..3) {
        & xelatex -interaction=nonstopmode -output-directory=build main.tex | Out-Null
        if ($LASTEXITCODE -ne 0) { Write-Warning "[$t] pass $i returned $LASTEXITCODE" }
    }

    $log = 'build\main.log'
    if (Test-Path $log) {
        $errs  = Select-String -Path $log -Pattern '^!' | ForEach-Object { $_.Line }
        $undef = Select-String -Path $log -Pattern 'Reference .* undefined|Citation .* undefined'
        if ($errs)  { Write-Host "[$t] errors:" -ForegroundColor Red; $errs | Select-Object -First 20 }
        if ($undef) { Write-Host "[$t] undefined references:" -ForegroundColor Yellow; $undef | Select-Object -First 20 }
        if (-not $errs -and -not $undef) { Write-Host "[$t] clean: no errors, no undefined references." -ForegroundColor Green }
        Select-String -Path $log -Pattern 'Output written' | ForEach-Object { Write-Host "[$t] $($_.Line)" }
    }
    # remove intermediates, keep pdf + log
    Get-ChildItem 'build' -File |
        Where-Object { $_.Extension -notin '.pdf', '.log' } |
        Remove-Item -Force
    Pop-Location
}

Write-Host "`nArtifacts:" -ForegroundColor Green
foreach ($t in $targets) {
    $pdf = Join-Path $root "$t\build\main.pdf"
    if (Test-Path $pdf) { Write-Host "  $pdf" }
}
