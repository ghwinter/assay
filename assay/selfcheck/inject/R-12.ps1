# R-12 — 证明 S-12 红得了：让 git_visible_files 退化回"不看 .gitignore 的全量 glob"
# （这正是修好之前的行为：作用域由文件系统决定，于是被忽略的 .venv 也进来了）
. "$PSScriptRoot\..\_inject.ps1"
Invoke-ToolInjection -Name 'S-12' `
    -Source 'tools/ledger_lint.py' `
    -Find '    files = set()
    for line in r.stdout.splitlines():' `
    -Replace '    files = set()
    for p in root.glob("**/*"):
        if p.is_file():
            files.add(p.relative_to(root).as_posix())
    for line in r.stdout.splitlines():' `
    -CheckScript 'S-12-l1-scope-follows-git.ps1'
