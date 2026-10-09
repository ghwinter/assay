param(
    [string]$RepositoryRoot = (Join-Path $PSScriptRoot '..')
)

$ErrorActionPreference = 'Stop'
$RepositoryRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
$ExampleRoot = Join-Path $PSScriptRoot 'rust-service'

function Invoke-Checked {
    param(
        [string]$Label,
        [string]$Executable,
        [string[]]$Arguments,
        [string]$WorkingDirectory
    )

    Write-Host "== $Label =="
    Push-Location $WorkingDirectory
    try {
        & $Executable @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "$Label failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

$TempProject = Join-Path $env:TEMP ('assay-init-check-' + [guid]::NewGuid().ToString('N'))
Invoke-Checked 'Integrated project initializer' 'pwsh' @(
    '-NoProfile', '-File', (Join-Path $RepositoryRoot 'tools/new-rust-project.ps1'),
    '-Target', $TempProject, '-Name', 'initializer-smoke-test'
) $RepositoryRoot

foreach ($RequiredPath in @('AGENTS.md', 'CONSTITUTION.md', 'tools/spec.py', 'engineering/INTENT.md', 'assay/manifest.toml')) {
    if (-not (Test-Path -LiteralPath (Join-Path $TempProject $RequiredPath))) {
        throw "Initializer output is missing $RequiredPath"
    }
}
if ((Get-Content -LiteralPath (Join-Path $TempProject 'AGENTS.md') -Raw).Contains('pilot-rs')) {
    throw 'Initializer copied pilot-specific working rules into a generic Rust project'
}

Invoke-Checked 'Generate AI work package' 'python' @(
    (Join-Path $RepositoryRoot 'tools/prepare_work_package.py'), 'D-001',
    '--root', $ExampleRoot, '--output', 'engineering/work-packages/D-001.md', '--force'
) $RepositoryRoot
Invoke-Checked 'Expand decision triggers' 'python' @('tools/spec.py', 'expand', 'D-001') $ExampleRoot
Invoke-Checked 'Rust behavior tests' 'cargo' @('test', '--manifest-path', 'Cargo.toml') $ExampleRoot
Invoke-Checked 'Spec Ledger checker' 'python' @('tools/spec.py', 'check', 'O-001') $ExampleRoot
Invoke-Checked 'Example ledger gate' 'python' @('tools/gate.py') $ExampleRoot

$EngineManifest = Join-Path $RepositoryRoot 'engine/Cargo.toml'
Invoke-Checked 'Assay execute and record evidence' 'cargo' @(
    'run', '--quiet', '--manifest-path', $EngineManifest, '--',
    'run', '--root', $ExampleRoot, '--stage', 'pre-commit'
) $RepositoryRoot
Invoke-Checked 'Assay consistency check' 'cargo' @(
    'run', '--quiet', '--manifest-path', $EngineManifest, '--', 'check', '--root', $ExampleRoot
) $RepositoryRoot
Invoke-Checked 'Regenerate Assay map' 'cargo' @(
    'run', '--quiet', '--manifest-path', $EngineManifest, '--', 'map', '--root', $ExampleRoot
) $RepositoryRoot

Write-Host 'WORKFLOW PASS: init -> intent/decision -> work package -> Rust tests -> ledger -> Assay evidence/map'
