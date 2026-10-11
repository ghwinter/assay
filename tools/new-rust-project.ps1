[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Target,
    [Parameter(Mandatory = $true)][string]$Name,
    [string]$SpecLedgerRoot = (Join-Path $PSScriptRoot '..'),
    # 显式指定解释器时走 ASSAY_PYTHON；留空则由唯一入口解析出真正能跑的那一个。
    # 原先默认 'python' 正是那次事故的形态：PATH 上的 python 可能是执行即失败的占位符。
    [string]$Python = ''
)

$ErrorActionPreference = 'Stop'

$specRoot = (Resolve-Path -LiteralPath $SpecLedgerRoot -ErrorAction Stop).Path
$specTool = Join-Path $specRoot 'tools\spec.py'
$cli = Join-Path $PSScriptRoot 'assay-cli.ps1'
$templateRoot = Join-Path $PSScriptRoot '..\templates\rust-app'
$engineeringTemplate = Join-Path $templateRoot 'engineering'
$manifestTemplate = Join-Path $templateRoot 'assay\manifest.toml'
$agentTemplate = Join-Path $templateRoot 'AGENTS.md'
$runbookTemplate = Join-Path $templateRoot 'RUNBOOK.md'
$workPackageTool = Join-Path $PSScriptRoot 'prepare_work_package.py'
$targetPath = [System.IO.Path]::GetFullPath($Target)

if (-not (Test-Path -LiteralPath $specTool -PathType Leaf)) {
    throw "Spec workflow CLI not found: $specTool"
}
if (-not (Test-Path -LiteralPath $cli -PathType Leaf)) {
    throw "Entry point not found: $cli"
}
if (-not (Test-Path -LiteralPath $engineeringTemplate -PathType Container)) {
    throw "Rust project templates not found: $engineeringTemplate"
}
if (-not (Test-Path -LiteralPath $manifestTemplate -PathType Leaf)) {
    throw "Assay manifest template not found: $manifestTemplate"
}
if (-not (Test-Path -LiteralPath $agentTemplate -PathType Leaf)) {
    throw "Rust project agent template not found: $agentTemplate"
}
if (-not (Test-Path -LiteralPath $runbookTemplate -PathType Leaf)) {
    throw "Rust project runbook template not found: $runbookTemplate"
}
if (-not (Test-Path -LiteralPath $workPackageTool -PathType Leaf)) {
    throw "Work package generator not found: $workPackageTool"
}

$hadOverride = $env:ASSAY_PYTHON
if ($Python) { $env:ASSAY_PYTHON = $Python }
try {
    & pwsh -NoProfile -ExecutionPolicy Bypass -File $cli py $specTool init $targetPath $Name
    if ($LASTEXITCODE -ne 0) {
        throw "Spec Ledger initialization failed with exit code $LASTEXITCODE"
    }
} finally {
    if ($hadOverride) { $env:ASSAY_PYTHON = $hadOverride } else { Remove-Item Env:\ASSAY_PYTHON -ErrorAction SilentlyContinue }
}

$engineeringTarget = Join-Path $targetPath 'engineering'
New-Item -ItemType Directory -Path $engineeringTarget -Force | Out-Null
Copy-Item -Path (Join-Path $engineeringTemplate '*') -Destination $engineeringTarget -Recurse
Copy-Item -LiteralPath $agentTemplate -Destination (Join-Path $targetPath 'AGENTS.md') -Force
Copy-Item -LiteralPath $runbookTemplate -Destination (Join-Path $targetPath 'RUNBOOK.md') -Force

$assayTarget = Join-Path $targetPath 'assay'
New-Item -ItemType Directory -Path $assayTarget -Force | Out-Null
Copy-Item -LiteralPath $manifestTemplate -Destination (Join-Path $assayTarget 'manifest.toml')
Copy-Item -LiteralPath $workPackageTool -Destination (Join-Path $targetPath 'tools\prepare_work_package.py')

Write-Host "Initialized assurance workspace: $targetPath"
Write-Host 'Next: complete engineering/INTENT.md, approve the first goal, then draft one vertical-slice decision.'
