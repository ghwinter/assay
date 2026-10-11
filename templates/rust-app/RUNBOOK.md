# RUNBOOK — commands for this project

> On-demand, unlike `AGENTS.md`, which is injected into every session.
> Concrete commands, paths, and versions live here, because they rot and the injected file must not.

## The one entry point

```text
assay help          # verb table
assay doctor        # readiness; runs no checks
assay next          # what to do now: an ordered action list
assay check         # manifest self-check (gate; runs no checks)
assay map           # regenerate the generated world-model page from the manifest
assay run [--stage pre-commit|pr|merge|nightly] [--check <id>] [--change <id>] [--certify]
assay plan --base <rev> [--change <id>] [--format json]
assay gate          # the four-part commit gate
assay lint | bridge | digest [--check] | trace <id> | ledger <O-id> | spec <...> | wp <D-id>
assay bootstrap     # prepare/verify the Python side (creates tools/.venv if PyYAML is missing)
```

Interpreter resolution order (`tools/assay-cli.ps1`):

```text
1. ASSAY_PYTHON                     explicit override (machine-local; never committed)
2. tools/config.toml python         bare name, resolved through PATH (portable)
3. a daily venv                     %USERPROFILE%\{envs,venvs,.venvs,.virtualenvs}\*\Scripts\python.exe
4. tools\.venv                      repo-local fallback
5. uv creates tools\.venv
6. uv temporary environment (--with pyyaml)
7. any interpreter that can import yaml
```

Steps 1–4 all require "runs **and** has PyYAML": every ledger tool imports `yaml`. An explicit
`ASSAY_PYTHON` that cannot satisfy that **fails loudly** rather than being silently replaced.
Each ledger invocation prints `[py] <interpreter actually used>`; set `ASSAY_QUIET=1` to silence it.

- Keep `python` a **bare name** in shared config so the repository stays portable; to pin an
  interpreter, set `ASSAY_PYTHON` instead of committing an absolute path.
- Make the pre-commit hook effective: `git config core.hooksPath .githooks`.
  Without that line the hook file is just text nobody executes.

## This project's own commands

<!-- Fill in: build, test, model checking, proofs, benchmarks, and where each tool's path is configured. -->
