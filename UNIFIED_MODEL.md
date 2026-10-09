# Unified Engineering Model v1

This model brings the intent/decision ledger and the assurance runner into one project workflow. It unifies ownership and traceability; it does not require every project to use every model or checker.

## One project root

An application has one repository and one project identity. Ledger records, models, source code, checks, and evidence are different artifact classes within that project, not separate projects.

```text
my-app/
├── ledger/                 # canonical goals, decisions, assumptions, obligations
├── engineering/
│   ├── architecture/       # human-reviewed structure and interface decisions
│   ├── models/             # selected behavior, contract, and system models
│   └── work-packages/      # generated, bounded AI/human change packets
├── assay/
│   ├── manifest.toml       # formalizations and checker bindings
│   ├── evidence/           # append-only execution records
│   ├── ratchet.toml        # accepted checker-strength commitments
│   └── MAP.md              # generated view
├── src/
└── tests/
```

The framework itself should also become one product. Its target source layout is one workspace with `ledger`, `runner`, `workflow`, and `cli` components; the former Spec Ledger and Assay implementations become internal components. The source repositories are not moved by this model change: history-preserving consolidation is a separate migration step.

## Typed artifact graph

```text
Goal (G) → Decision (D) → Obligation (O) → Assurance claim (C) → Check (K) → Evidence event (E)
                    └── Assumption (A)                  ↑
Change (CH) ────────────────────────────────────────────┘
```

| Type | Meaning | Authority |
|---|---|---|
| `G` | Intended outcome and acceptance direction | Human-approved ledger |
| `D` | A selected choice, alternatives, rationale, and status | Human-approved ledger |
| `A` | A premise and the evidence that would invalidate it | Decision record or a stable assumption reference |
| `O` | A falsifiable engineering obligation that supports a goal or decision | Ledger; its wording is not copied as a second authority |
| `C` | A more precise, judgeable formalization of one or more obligations | Assurance plan; must reference `O` IDs or declare a non-ledger `origin` |
| `K` | An executable checker bound to one or more formalizations | Assurance plan |
| `E` | The append-only result of one checker execution | Runner-generated evidence |
| `CH` | A proposed change and the obligations/checks it touches or adds | Change record, authored before implementation |

An assurance claim is not a second copy of an obligation. It states the narrower proposition a checker can judge (for example, a TLA+ safety property derived from a prose concurrency obligation). Every project claim must identify its source obligation IDs. Internal protocol/self-check claims may instead use an explicit `origin`, such as `assay-protocol`. The two source forms cannot be mixed on one claim.

The binding is directional and typed: `D introduces O`, `C formalizes O`, `K judges C`, and `E records K's result`. One obligation may have multiple formalizations and checks; one check may judge multiple claims. No edge means no inferred coverage. This is declared traceability, not automatic semantic impact analysis.

## Ownership and generation

- The ledger owns goals, decisions, assumptions, and obligations. The owner decides whether they express the intended system behavior.
- The assurance plan owns formalizations and checker execution details. Formalization may add precision but must not silently change the obligation.
- The runner owns generated evidence and the ratchet. It does not edit intent or decide whether a model matches intent.
- `MAP.md` and per-check `*.json` latest snapshots are generated views. The append-only `*.jsonl` event history is canonical.
- A legacy `ledger/evidence/O-*.yaml` status record is not a second live evidence authority. During migration it is historical/imported input; new runs are read from the Assay event stream.
- Reference repositories are research inputs, not authorities. Record reusable findings against the local decision they informed; only the reviewed ledger decision and its obligations define this project's intent.

The root may contain human-readable explanation, executable models, configuration, and code. Each has a different job. Do not manually repeat the same normative statement in multiple authoritative places; use IDs and explicit relations, then generate views where needed.

## Evidence relation

Each evidence event binds together:

- the run and optional change ID;
- the check, claim IDs, source obligation IDs or non-ledger origins, and assumption IDs;
- the checker capability, declared strength/scope, command fingerprint, and checker/engine versions;
- the result, exit code, start time, duration, detail, and birth-certificate status;
- the manifest fingerprint and optional source revision supplied through `ASSAY_SOURCE_REVISION`.

`pass` means only that this checker returned success for this recorded scope and assumptions. It does not certify the obligation, model-to-code correspondence, or product as a whole. Fingerprints identify configuration; they are not cryptographic signatures. A changed input makes old evidence historical; freshness/impact is not inferred unless an explicit binding says so.

Append-only is an engine behavior and review convention, not tamper-proof storage. Projects requiring adversarial audit integrity need separately signed attestations or an immutable external evidence store.

The runner records a bounded tail of checker output but does not understand or redact its contents. Checkers and wrappers must avoid emitting secrets; restrict access to evidence when output can contain sensitive data.

The canonical event schema is [protocol/schemas/evidence-v1.schema.json](protocol/schemas/evidence-v1.schema.json). One JSON object is appended per check run to `assay/evidence/<check-id>.jsonl`. `assay/evidence/<check-id>.json` is only the latest-record projection. Both use the same versioned envelope; only the JSONL stream preserves run history.

## Migration map

| Existing source | Unified role |
|---|---|
| Spec Ledger `G-*.yaml`, `D-*.yaml`, nested assumptions and `O-*` | Canonical intent/decision/obligation records |
| Assay `[[claim]]` | `C`: a formalization, linked to source obligations or an explicit origin |
| Assay `[[check]]` | `K`: a checker and its declared capability, scope, stage, and command |
| Assay `[[change]]` | `CH`: add/touch/remove obligation and evidence requirements |
| Assay `evidence/*.jsonl` | Canonical `E` execution history; append-only after v1 |
| Spec Ledger `ledger/evidence/O-*.yaml` | Legacy status snapshot; not a competing authority for new runs |

Protocol v0.6 adds source links and the v1 evidence envelope while keeping existing `[[claim]]` and `[[check]]` syntax readable. The next migration boundary is a generator that projects accepted ledger obligations and their checker bindings into the runner manifest; until that exists, claim-to-obligation links are explicit and reviewed, not silently inferred.
