# R-01 — 证明 S-01 红得了：往协议里写一个后端工具名
. "$PSScriptRoot\..\_inject.ps1"
Invoke-DocInjection -Name 'S-01' `
    -File 'PROTOCOL.md' `
    -Append "`n<!-- 注入：举例时说「例如 Verus 可以证明 Rust 函数」（这一行就是违约） -->" `
    -CheckScript 'S-01-protocol-has-no-tool-names.ps1'
