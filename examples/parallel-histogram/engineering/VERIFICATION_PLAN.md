# Verification plan

| Claim | Checker | Boundary |
|---|---|---|
| 256 bucket counts match sequential behavior | Unit tests with reference oracle | Covered inputs and worker counts only |
| Work partitioning preserves all input bytes | Oracle equality and count conservation checks | Test suite, not exhaustive proof |
| Invalid worker count is rejected | Explicit zero-worker test | Public API behavior |
| Release performance can be measured | `cargo bench` executable | One machine, compiler, input pattern, and run |
| Assay recognizes the claim/check binding | `assay check` | Manifest structure and declared command only |
| Bounded work drains and stays within queue capacity | TLC over finite TLA+ model | Abstract protocol state space only; not an implementation refinement proof |
| Streaming pipeline accounts for all produced work | Rust tests with capacity-one queue, short reads, and injected I/O failure | Test cases only; blocking reads cannot be cancelled |
| File CLI emits complete machine-readable output | CLI integration tests | Tested invocations only |

No performance threshold is selected because no target workload or hardware was specified. Treat
benchmark output as exploratory evidence and preserve the invocation and environment when comparing
future changes.
