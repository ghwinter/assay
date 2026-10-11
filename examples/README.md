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

For an incremental plan in a project initialized as its own Git repository, run
`assay plan --root <project-root> --base <git-revision> --format json`. The example declares
`src/`, `Cargo.toml`, and `Cargo.lock` as check inputs; a changed path outside all declared scopes
causes Assay to select every check rather than assume the change is irrelevant.

`parallel-histogram/` is the same computation example iterated into a small streaming system (not a
second sample): a file CLI feeds chunks through a bounded queue to worker-local histograms, drains
workers on EOF, checks accounting, and reports failures without partial results. Its sequential
oracle, Rust tests, finite TLA+ protocol model, Assay claims, and CI checks cover different layers;
the model is not a proof of implementation refinement. The exploratory benchmark still has no
hardware-independent speed threshold. Parent decisions `ledger/decisions/D-031.yaml` and
`ledger/decisions/D-032.yaml` govern the sample.
