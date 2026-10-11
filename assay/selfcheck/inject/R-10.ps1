# R-10 — 证明 S-10 红得了：让 resolve_command_in 永远"解析成功"
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-10' `
    -PatchFile 'readiness.rs' `
    -Find 'let t = token.trim().trim_matches(''"'').trim_matches(''\'''');' `
    -Replace 'return Some(root.join("always-resolves")); let t = token.trim().trim_matches(''"'').trim_matches(''\'''');' `
    -CheckScript 'S-10-unresolvable-command-is-red.ps1'
