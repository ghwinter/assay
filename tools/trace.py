#!/usr/bin/env python3
"""trace — Spec Ledger 影响分析 (v0)

用法:
  python tools/trace.py                      账本总览
  python tools/trace.py G-002                目标 ← 支撑它的决策/义务/证据
  python tools/trace.py D-003                决策 ← 上游目标 / → 义务/证据/产物
  python tools/trace.py O-006                义务 ← 所属决策、目标、证据状态
  python tools/trace.py backpressure_policy  触发器键 → 覆盖它的决策/义务
  python tools/trace.py pilot/src/queue.hpp  文件 → 拥有它的决策及其目标
"""
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
LED = ROOT / "ledger"


def load(d, pat):
    out = {}
    if not d.exists():
        return out
    for p in sorted(d.glob(pat)):
        try:
            out[p.stem] = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        except Exception as e:  # noqa: BLE001
            out[p.stem] = {"_parse_error": str(e)}
    return out


def as_list(x):
    return x if isinstance(x, list) else ([] if x is None else [x])


def main():
    goals = load(LED / "intent", "G-*.yaml")
    decisions = load(LED / "decisions", "D-*.yaml")
    triggers = load(LED / "triggers", "*.yaml")
    evidence = load(LED / "evidence", "O-*.yaml")

    obl = {}
    for did, d in decisions.items():
        for o in as_list(d.get("obligations")):
            oid = str(o.get("id", ""))
            obl[oid] = {"id": oid, "did": did, "desc": o.get("desc", ""),
                        "satisfies": o.get("satisfies"), "checker": o.get("checker") or {}}
    ev = {str(e.get("obligation")): e for e in evidence.values() if e.get("obligation")}
    file_of = {}
    for did, d in decisions.items():
        for a in as_list(d.get("artifacts")):
            file_of.setdefault(str(a), []).append(did)
    rule_of = {}
    for t in triggers.values():
        for r in as_list(t.get("obligations")):
            rule_of[r.get("key")] = (t.get("trigger"), r.get("desc", ""))

    def ev_line(oid):
        st = ev.get(oid, {}).get("status")
        return {"pass": "PASS", "deferred": "DEFER", "fail": "FAIL"}.get(st, "----")

    if len(sys.argv) < 2:
        print("== Spec Ledger 总览 ==")
        print(f"目标 {len(goals)} | 决策 {len(decisions)} | 义务 {len(obl)} | 证据 {len(ev)}")
        for gid in sorted(goals):
            print(f"\n[{gid}] {goals[gid].get('title', '')}")
            print(f"  验收: {', '.join(str(x) for x in as_list(goals[gid].get('verification')))}")
        print("\n查询: trace.py <G-xxx|D-xxx|O-xxx|触发器键|文件路径>")
        return 0

    q = sys.argv[1]

    if q in goals:
        g = goals[q]
        print(f"[{q}] {g.get('title', '')}")
        print(g.get("description", "").strip())
        print(f"\n验收义务: {', '.join(str(x) for x in as_list(g.get('verification')))}")
        for did in sorted(decisions):
            d = decisions[did]
            if q in [str(x) for x in as_list(d.get("goal"))]:
                print(f"\n← {did} [{d.get('status')}] {d.get('question', '')}")
                print(f"   choice: {d.get('choice', '')}")
                for o in as_list(d.get("obligations")):
                    oid = str(o.get("id"))
                    print(f"   → {oid} [{ev_line(oid)}] ({o.get('satisfies')}) {o.get('desc', '')}")
        return 0

    if q in decisions:
        d = decisions[q]
        print(f"[{q}] status={d.get('status')}")
        print(f"question: {d.get('question', '')}")
        print(f"choice:   {d.get('choice', '')}")
        print(f"alternatives: {', '.join(str(x) for x in as_list(d.get('alternatives')))}")
        for r in as_list(d.get("rationale")):
            if isinstance(r, dict):
                print(f"rationale: {r.get('text', '')}  (anchor: {r.get('anchor', '—')})")
        print(f"\n上游目标: {', '.join(str(x) for x in as_list(d.get('goal')))}")
        print(f"引入触发器: {', '.join(str(x) for x in as_list(d.get('introduces'))) or '—'}")
        for o in as_list(d.get("obligations")):
            oid = str(o.get("id"))
            ck = o.get("checker") or {}
            print(f"\n→ {oid} [{ev_line(oid)}] ({o.get('satisfies')}) {o.get('desc', '')}")
            print(f"   checker: {ck.get('type', '—')} | {ck.get('ref', '—')}")
        print(f"\n产物: {', '.join(str(x) for x in as_list(d.get('artifacts')))}")
        if d.get("superseded_by"):
            print(f"被取代: {d['superseded_by']}")
        return 0

    if q in obl:
        o = obl[q]
        d = decisions[o["did"]]
        print(f"[{q}] ({o['satisfies']}) {o['desc']}")
        print(f"所属决策: {o['did']} [{d.get('status')}] {d.get('question', '')}")
        print(f"上游目标: {', '.join(str(x) for x in as_list(d.get('goal')))}")
        ck = o["checker"]
        print(f"checker:  {ck.get('type', '—')} | {ck.get('ref', '—')}")
        e = ev.get(q, {})
        print(f"证据:     [{ev_line(q)}] {e.get('command', '')}"
              + (f" | {e.get('notes')}" if e.get("notes") else ""))
        if o["satisfies"] in rule_of:
            tk, td = rule_of[o["satisfies"]]
            print(f"触发器:   {tk} — {td}")
        return 0

    if q in rule_of:
        tk, td = rule_of[q]
        print(f"[触发器义务 {q}] {tk} — {td}")
        for did in sorted(decisions):
            d = decisions[did]
            hit = [str(o.get("id")) for o in as_list(d.get("obligations"))
                   if str(o.get("satisfies")) == q]
            if hit:
                print(f"  覆盖: {did} [{d.get('status')}] → {', '.join(hit)}")
            for w in as_list(d.get("waivers")):
                if w.get("id") == q:
                    print(f"  豁免: {did} — {w.get('reason', '')} (owner: {w.get('owner', '—')})")
        return 0

    norm = Path(q).as_posix()
    if norm in file_of:
        print(f"[产物] {norm}")
        for did in file_of[norm]:
            d = decisions[did]
            print(f"\n← 拥有者 {did} [{d.get('status')}] {d.get('question', '')}")
            print(f"   服务目标: {', '.join(str(x) for x in as_list(d.get('goal')))}")
            for o in as_list(d.get("obligations")):
                oid = str(o.get("id"))
                print(f"   义务 {oid} [{ev_line(oid)}] ({o.get('satisfies')}) {o.get('desc', '')}")
        return 0

    print(f"未找到: {q!r}")
    print(f"可用: {' '.join(sorted(goals))} / {' '.join(sorted(decisions))} / "
          f"{' '.join(sorted(obl))} / 触发器键 / 文件路径")
    return 1


if __name__ == "__main__":
    sys.exit(main())
