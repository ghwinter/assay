# R-07 — 证明 S-07 红得了：让地图新鲜度永远通过
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-07' `
    -Find 'if have == want {' `
    -Replace 'if true {' `
    -CheckScript 'S-07-map-drift-is-red.ps1'
