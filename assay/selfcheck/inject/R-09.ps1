# R-09 — 证明 S-09 红得了：让分歧检测失效
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-09' `
    -Find 'if !p.is_empty() && !f.is_empty() {' `
    -Replace 'if false {' `
    -CheckScript 'S-09-contested-is-reported.ps1'
