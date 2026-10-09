# Intent: bounded in-memory request service

## User and scenario

A caller submits a request to create a record and then reads it by identifier in the same process.

## Observable success

- A non-blank identifier and body are trimmed, stored once, and returned by lookup.
- A blank identifier, blank body, or duplicate identifier is rejected with a distinct error.

## Failure consequences

The caller receives an explicit error instead of a silently accepted invalid or duplicate record.
Records are lost when the process exits; persistence is intentionally out of scope.

## Runtime and non-goals

This is a Rust library slice with an in-memory map and no external dependencies. Network protocols,
authentication, persistence, deployment, concurrency, and production availability are not claimed.

## Acceptance

`cargo test --manifest-path examples/rust-service/Cargo.toml` passes, the project ledger gate passes,
and the Assay manifest records a passing run for the linked obligation.
