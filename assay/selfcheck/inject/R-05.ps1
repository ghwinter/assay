# R-05 — 证明 S-05 红得了：让没有出生证明的检查也能进棘轮
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-05' `
    -Find 'if b != Birth::Missing' `
    -Replace 'if true' `
    -CheckScript 'S-05-missing-birth-certificate.ps1'
