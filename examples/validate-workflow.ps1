param(
    [string]$RepositoryRoot = (Join-Path $PSScriptRoot '..')
)

$ErrorActionPreference = 'Stop'
$RepositoryRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
$ExampleRoot = Join-Path $PSScriptRoot 'rust-service'
$ParallelExampleRoot = Join-Path $PSScriptRoot 'parallel-histogram'

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

$RustWorkPackageProbe = Join-Path $env:TEMP ('assay-work-package-' + [guid]::NewGuid().ToString('N') + '.md')
Invoke-Checked 'Generate AI work package' 'python' @(
    (Join-Path $RepositoryRoot 'tools/prepare_work_package.py'), 'D-001',
    '--root', $ExampleRoot, '--output', $RustWorkPackageProbe
) $RepositoryRoot
Remove-Item -LiteralPath $RustWorkPackageProbe
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

$ParallelWorkPackageProbe = Join-Path $env:TEMP ('assay-work-package-' + [guid]::NewGuid().ToString('N') + '.md')
Invoke-Checked 'Generate parallel histogram work package' 'python' @(
    (Join-Path $RepositoryRoot 'tools/prepare_work_package.py'), 'D-032',
    '--root', $RepositoryRoot, '--output', $ParallelWorkPackageProbe
) $RepositoryRoot
Remove-Item -LiteralPath $ParallelWorkPackageProbe
Invoke-Checked 'Expand streaming pipeline decision' 'python' @('tools/spec.py', 'expand', 'D-032') $RepositoryRoot
Invoke-Checked 'Parallel histogram behavior tests' 'cargo' @('test', '--locked', '--manifest-path', 'Cargo.toml') $ParallelExampleRoot
Invoke-Checked 'Bounded pipeline protocol model' 'python' @('check.py') (Join-Path $ParallelExampleRoot 'tla')
Invoke-Checked 'Generate parallel histogram map' 'cargo' @(
    'run', '--locked', '--quiet', '--manifest-path', $EngineManifest, '--', 'map', '--root', $ParallelExampleRoot
) $RepositoryRoot
Invoke-Checked 'Parallel histogram Assay check' 'cargo' @(
    'run', '--locked', '--quiet', '--manifest-path', $EngineManifest, '--', 'check', '--root', $ParallelExampleRoot
) $RepositoryRoot
Invoke-Checked 'Parallel histogram Assay evidence' 'cargo' @(
    'run', '--locked', '--quiet', '--manifest-path', $EngineManifest, '--',
    'run', '--root', $ParallelExampleRoot, '--stage', 'pre-commit'
) $RepositoryRoot
$StreamingWorkPackageProbe = Join-Path $env:TEMP ('assay-work-package-' + [guid]::NewGuid().ToString('N') + '.md')
Invoke-Checked 'Generate streaming pipeline work package' 'python' @(
    (Join-Path $RepositoryRoot 'tools/prepare_work_package.py'), 'D-032',
    '--root', $RepositoryRoot, '--output', $StreamingWorkPackageProbe
) $RepositoryRoot
Remove-Item -LiteralPath $StreamingWorkPackageProbe

Write-Host 'WORKFLOW PASS: init -> intent/decision -> work packages -> Rust tests -> TLC -> ledger -> Assay evidence/maps'
