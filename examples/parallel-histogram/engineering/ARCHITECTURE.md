# Architecture

`histogram_sequential` is the simple in-memory oracle. `histogram_parallel` partitions an immutable
slice into disjoint chunks. The streaming pipeline reads a generic `Read` in fixed-size chunks,
sends them through a bounded `sync_channel`, and has scoped workers build private histograms. After
EOF the sender closes and workers drain queued chunks before returning reports for deterministic
reduction. Read errors close the channel and still join workers; the caller receives an error rather
than partial counts. Checked accounting compares produced and processed byte/chunk totals.

The `histogram` binary exposes this pipeline for files and prints JSON counts. Configuration keeps
worker count, chunk size, and queue capacity explicit. Blocking reads are not cancellable in this
version. The TLA+ model checks an abstract finite queue/worker protocol, not the Rust channel or OS.

```text
file -> reader -> bounded queue -> worker-local histograms -> join/reduce -> JSON
                              |                                  |
                              +------- sequential oracle --------+-> equality tests
                         abstract protocol -> TLC
```

The computational dependency graph is intentionally small: chunk counting is parallel, while
reduction depends on all local results. The tests verify observable equivalence and count
conservation; they do not assert that every input size benefits from parallelism.
