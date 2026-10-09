# R-03 — 证明 S-03 红得了：让引擎无论什么情况都 exit 0
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-03' `
    -Find 'std::process::exit(code);' `
    -Replace 'std::process::exit(0);' `
    -CheckScript 'S-03-unknown-subcommand-is-red.ps1'
