# R-02 — 证明 S-02 红得了：往引擎源码里写一个后端工具名
. "$PSScriptRoot\..\_inject.ps1"
Invoke-DocInjection -Name 'S-02' `
    -File 'engine\src\main.rs' `
    -Append "`n// 注入：对齐 Verus 的输出格式（这一行就是违约）" `
    -CheckScript 'S-02-engine-knows-no-tools.ps1'
