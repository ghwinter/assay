# Parallel histogram example

This is a small, dependency-free Rust example governed by the parent Assay ledger decision D-031.
Keep behavior within that decision; update the parent ledger before changing its contract.

Run tests with `cargo test --locked --manifest-path examples/parallel-histogram/Cargo.toml` from
the repository root. Run the exploratory release benchmark with `cargo bench --locked
--manifest-path examples/parallel-histogram/Cargo.toml`. Its timing output is evidence about only
the current machine and workload, not a portable performance guarantee.
