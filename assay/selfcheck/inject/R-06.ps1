# R-06 — 证明 S-06 红得了：拿掉棘轮的强度削弱判定
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-06' `
    -Find 'if c.strength < e.strength {' `
    -Replace 'if false {' `
    -CheckScript 'S-06-ratchet-cannot-be-weakened.ps1'
