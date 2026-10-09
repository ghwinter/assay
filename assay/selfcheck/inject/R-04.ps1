# R-04 — 证明 S-04 红得了：拿掉档 4 禁令
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-04' `
    -Find 'strength4.is_empty()' `
    -Replace 'true' `
    -CheckScript 'S-04-strength4-is-rejected.ps1'
