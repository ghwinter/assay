# Project AI working rules

Before changing this project, read `CONSTITUTION.md`, `AGENTS.md`, and `LEDGER.md`. The ledger is the
project's current model; consult linked decisions, assumptions, checkers, and evidence before acting.

1. Do not change source behavior without an accepted decision that covers the change.
2. Draft decisions first, expand relevant triggers, and leave unresolved product choices for a human.
3. Every obligation needs a checker. Record signed deferred evidence when a checker cannot yet run.
4. Run `tools/gate.py` after ledger changes and the checkers linked to affected obligations after code
   changes. Refresh `LEDGER.md` after modifying the ledger.
5. Keep implementation scope within the accepted decision. If the intent or model is insufficient,
   stop and propose a ledger change rather than silently choosing semantics.
6. Report the changed goal, decision, obligation, evidence, commands, results, and known limits.

Use this project's `WORKFLOW.md` and `tools/config.toml` for its actual commands and tool paths. Do not
assume a particular application architecture, deployment target, database, or verification backend.
