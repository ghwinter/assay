# R-11 — 证明 S-11 红得了：让 freshness 恒返回 Current（证据永远"当前"）
. "$PSScriptRoot\..\_inject.ps1"
Invoke-EngineInjection -Name 'S-11' `
    -PatchFile 'readiness.rs' `
    -Find 'fn freshness(c: &super::Check, ev: Option<&EvidenceView>) -> Freshness {
    let Some(ev) = ev else {
        return Freshness::Missing;
    };' `
    -Replace 'fn freshness(c: &super::Check, ev: Option<&EvidenceView>) -> Freshness {
    return Freshness::Current;
    let Some(ev) = ev else {
        return Freshness::Missing;
    };' `
    -CheckScript 'S-11-next-detects-definition-drift.ps1'
