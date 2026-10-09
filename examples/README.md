# Workflow example

`rust-service/` is a deliberately small, self-contained Rust service slice. It accepts a request,
validates it, assigns a stable identifier, and stores it in an in-memory repository. It demonstrates
how intent becomes an architecture note, a decision, a checker-backed obligation, an AI work package,
Rust implementation/tests, and Assay evidence. It does not claim persistence, network, deployment,
security, or formal concurrency properties.

Run the full workflow from the Assay repository root:

```powershell
pwsh -NoProfile -File examples/validate-workflow.ps1
```

The script validates the project initializer in a temporary directory, then runs the committed
example's ledger gate, decision expansion, work-package generation, Rust tests, ledger checker, and
Assay `check`/`run`/`map`. The example is committed without a nested `.git` directory.
