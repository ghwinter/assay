#!/usr/bin/env python3
"""Create a bounded AI implementation packet from an accepted Spec Ledger decision."""

import argparse
import json
import re
import sys
from pathlib import Path

import yaml


ID_PATTERN = re.compile(r"^(?:D|G|O)-\d{3,}$")


def load_record(path: Path) -> dict:
    if not path.is_file():
        raise ValueError(f"record not found: {path}")
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"record is not a YAML mapping: {path}")
    return value


def ids(value) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    return [str(item) for item in value]


def section(lines: list[str], title: str, values: list[str], empty: str) -> None:
    lines.extend([f"## {title}", ""])
    lines.extend(values if values else [empty])
    lines.append("")


def render_rationale(records) -> list[str]:
    lines = []
    for item in records or []:
        if isinstance(item, dict):
            text = str(
                item.get("text")
                or item.get("because")
                or item.get("reason")
                or item.get("desc")
                or ""
            ).strip()
            alternative = str(item.get("alternative") or item.get("alt") or "").strip()
            anchor = str(item.get("anchor", "")).strip()
            prefix = f"**{alternative}:** " if alternative else ""
            lines.append(f"- {prefix}{text}" + (f"  _(anchor: {anchor})_" if anchor else ""))
        else:
            lines.append(f"- {item}")
    return lines


def latest_assay_evidence(root: Path, obligation_id: str) -> dict | None:
    evidence_dir = root / "assay" / "evidence"
    if not evidence_dir.is_dir():
        return None

    latest = None
    for path in sorted(evidence_dir.glob("*.jsonl")):
        with path.open(encoding="utf-8") as stream:
            for line_number, line in enumerate(stream, start=1):
                if not line.strip():
                    continue
                try:
                    record = json.loads(line)
                except json.JSONDecodeError as exc:
                    raise ValueError(f"invalid Assay evidence at {path}:{line_number}: {exc}") from exc
                if not isinstance(record, dict):
                    raise ValueError(f"Assay evidence is not an object at {path}:{line_number}")
                if record.get("schema_version") != "assay-evidence/v1":
                    continue
                lineage = record.get("lineage")
                if not isinstance(lineage, dict):
                    raise ValueError(f"Assay evidence has no lineage object at {path}:{line_number}")
                obligation_ids = lineage.get("obligation_ids")
                if not isinstance(obligation_ids, list) or not all(isinstance(item, str) for item in obligation_ids):
                    raise ValueError(f"Assay evidence has invalid obligation_ids at {path}:{line_number}")
                if obligation_id not in obligation_ids:
                    continue
                result = record.get("result")
                if not isinstance(result, dict):
                    raise ValueError(f"Assay evidence has no result object at {path}:{line_number}")
                candidate = (str(result.get("started_at", "")), str(record.get("run_id", "")), record)
                if latest is None or candidate[:2] > latest[:2]:
                    latest = candidate

    return latest[2] if latest else None


def render_assumptions(records) -> list[str]:
    lines = []
    for item in records or []:
        if isinstance(item, dict):
            text = str(item.get("text", "")).strip()
            invalidation = str(item.get("invalidation", "")).strip()
            lines.append(f"- **Assumption:** {text}")
            if invalidation:
                lines.append(f"  - **Invalidated when:** {invalidation}")
        else:
            lines.append(f"- {item}")
    return lines


def render_obligations(records, root: Path) -> list[str]:
    lines = []
    for item in records or []:
        if not isinstance(item, dict):
            lines.append(f"- {item}")
            continue
        oid = str(item.get("id", "unknown"))
        desc = str(item.get("desc", "")).strip()
        lines.append(f"### {oid} — {desc}" if desc else f"### {oid}")
        lines.append("")
        for key, label in (("satisfies", "Trigger"), ("origin", "Origin")):
            if item.get(key):
                lines.append(f"- **{label}:** {item[key]}")
        checker = item.get("checker") or {}
        if checker:
            lines.append(f"- **Checker type:** {checker.get('type', 'unspecified')}")
            for key in ("cmd", "name", "target", "ref", "scope", "bound"):
                if checker.get(key):
                    lines.append(f"- **{key}:** `{checker[key]}`")
        else:
            lines.append("- **Checker:** UNASSIGNED — do not claim this obligation is verified.")
        assay_evidence = latest_assay_evidence(root, oid)
        if assay_evidence:
            result = assay_evidence.get("result") or {}
            checker = assay_evidence.get("checker") or {}
            lines.append(
                f"- **Latest Assay evidence:** {result.get('verdict', 'unknown')} "
                f"({result.get('started_at', 'undated')}, check {checker.get('id', 'unknown')}); "
                "historical only—rerun after this change."
            )
        else:
            evidence_path = root / "ledger" / "evidence" / f"{oid}.yaml"
            if evidence_path.is_file():
                evidence = load_record(evidence_path)
                verdict = evidence.get("status", "unknown")
                date = evidence.get("date", "undated")
                lines.append(
                    f"- **Legacy ledger evidence:** {verdict} ({date}); "
                    "historical only—rerun after this change."
                )
            else:
                lines.append("- **Last recorded evidence:** none; establish a baseline.")
        lines.append("")
    return lines


def build_packet(root: Path, did: str) -> str:
    if not ID_PATTERN.fullmatch(did) or not did.startswith("D-"):
        raise ValueError("decision id must look like D-001")

    decision = load_record(root / "ledger" / "decisions" / f"{did}.yaml")
    if decision.get("id") != did:
        raise ValueError(f"decision file id does not match {did}")
    if decision.get("status") != "accepted":
        raise ValueError(f"{did} is not accepted; current status: {decision.get('status', 'missing')}")
    if decision.get("superseded_by"):
        raise ValueError(f"{did} was superseded by {decision['superseded_by']}")

    goal_records = []
    for gid in ids(decision.get("goal")):
        if not ID_PATTERN.fullmatch(gid) or not gid.startswith("G-"):
            raise ValueError(f"invalid goal id: {gid}")
        record = load_record(root / "ledger" / "intent" / f"{gid}.yaml")
        if record.get("id") != gid:
            raise ValueError(f"goal file id does not match {gid}")
        goal_records.append(record)

    lines = [
        f"# AI Work Package — {did}",
        "",
        f"> Generated from the accepted decision `{did}`. This package is a task view, not evidence of correctness.",
        "> Historical evidence below must be rerun for the current change.",
        "",
        "## Decision",
        "",
        f"- **Question:** {decision.get('question', '')}",
        f"- **Accepted choice:** {decision.get('choice', '')}",
        f"- **Alternatives:** {', '.join(ids(decision.get('alternatives')))}",
        "",
        "## Dispatch authorization",
        "",
        "- **Baseline commit:** fill before dispatch.",
        "- **Allowed paths:** fill before dispatch; the decision's artifact list is expected output, not blanket edit permission.",
        "- **Human approver:** fill before dispatch.",
        "",
        "## Goals",
        "",
    ]
    if goal_records:
        for goal in goal_records:
            lines.extend([
                f"### {goal.get('id', '')} — {goal.get('title', '')}",
                "",
                str(goal.get("description", "")).strip(),
                "",
                f"- **Acceptance obligations:** {', '.join(ids(goal.get('verification'))) or 'not specified'}",
                "",
            ])
            questions = goal.get("open_questions") or []
            if questions:
                lines.extend(["**Open questions:**", ""])
                lines.extend(f"- {item}" for item in questions)
                lines.append("")
    else:
        lines.extend(["No linked goals are recorded.", ""])

    section(lines, "Decision rationale", render_rationale(decision.get("rationale")), "No rationale recorded.")
    section(lines, "Rejected alternatives", render_rationale(decision.get("rejected_alternatives")), "No rejected alternatives recorded.")
    section(lines, "Assumptions", render_assumptions(decision.get("assumptions")), "No assumptions recorded.")
    section(lines, "Introduced triggers", [f"- `{item}`" for item in ids(decision.get("introduces"))], "No triggers recorded.")
    section(lines, "Open questions", [f"- {item}" for item in decision.get("open_questions", [])], "None recorded.")
    section(lines, "Required obligations", render_obligations(decision.get("obligations"), root), "No obligations recorded.")
    section(lines, "Waivers", render_rationale(decision.get("waivers")), "No waivers recorded.")
    section(lines, "Expected artifacts", [f"- `{item}`" for item in ids(decision.get("artifacts"))], "No artifact paths recorded.")

    lines.extend([
        "## Agent instructions",
        "",
        "1. Keep the accepted intent and choice unchanged. If they are insufficient or contradictory, stop and propose a ledger change.",
        "2. Implement only the approved scope. Do not weaken, delete, or silently bypass a required checker.",
        "3. Treat prior evidence as historical. Rerun the listed checkers after changing relevant artifacts.",
        "4. Report commands actually run, verdicts, scope limits, failed/unknown checks, and any new assumptions.",
        "5. A passing checker supports only its stated claim, range, and assumptions; do not summarize it as proof of the whole product.",
        "",
    ])
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate an AI work package from an accepted Spec Ledger decision")
    parser.add_argument("decision", help="accepted decision id, such as D-001")
    parser.add_argument("--root", default=".", help="project root containing ledger/")
    parser.add_argument("--output", help="write the package to this path; otherwise print to stdout")
    parser.add_argument("--force", action="store_true", help="overwrite an existing output file")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    try:
        packet = build_packet(root, args.decision)
        if args.output:
            output = Path(args.output)
            if not output.is_absolute():
                output = root / output
            if output.exists() and not args.force:
                raise ValueError(f"output exists (use --force to replace): {output}")
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(packet, encoding="utf-8")
            print(f"Wrote {output}")
        else:
            print(packet)
        return 0
    except (OSError, ValueError, yaml.YAMLError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
