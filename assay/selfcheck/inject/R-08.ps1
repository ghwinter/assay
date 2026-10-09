# R-08 — 证明 S-08 红得了：让范围规则失效（有界检查可以兑现无界声明）
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-08' `
    -Find 'Err(format!("声明是无界的，检查范围只有 {cscope}"))' `
    -Replace 'Ok(())' `
    -CheckScript 'S-08-scope-rule-holds.ps1'
