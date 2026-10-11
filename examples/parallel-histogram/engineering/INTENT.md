# Intent

## Purpose

Provide a Rust byte-histogram system that counts all byte values both in memory and from a file
stream. Demonstrate how an AI-assisted project makes its goal, concurrency protocol, failure
boundaries, implementation, independent reference, and evidence explicit.

## Success

- Return exactly 256 counts for every accepted input.
- Match a separately implemented sequential reference for all tested inputs and worker counts.
- Exercise disjoint task partitions and deterministic reduction without shared mutable counters.
- Process files in chunks through a bounded queue and multiple workers; drain workers after EOF.
- Reject invalid configuration and report read, worker, or accounting failures without partial output.
- Check the abstract bounded-queue close/drain protocol with a finite TLA+ model.
- Make a reproducible release benchmark available without claiming a machine-independent speedup.
- Run the project through the parent repository's Assay checks and end-to-end workflow.

## Non-goals

This example does not promise a performance target, cancellation of blocking reads, retry,
checkpointing, persistence, arbitrary bin ranges, GPU execution, distributed execution, SIMD-specific
code, service integration, or formal proof that the Rust implementation refines the TLA+ model.
