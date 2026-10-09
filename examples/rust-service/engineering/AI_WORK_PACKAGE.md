# AI work package: D-001

Implement only the accepted behavior in `ledger/decisions/D-001.yaml` and `engineering/ARCHITECTURE.md`.

- Preserve the error distinctions and trimming behavior in the accepted decision.
- Do not add dependencies, persistence, networking, concurrency claims, or public behavior not covered
  by the decision.
- Run `cargo test --manifest-path Cargo.toml` from this project root and
  `python tools/ledger_check.py O-001`.
- If behavior, assumptions, or scope need to change, stop and propose a ledger update first.
