#!/usr/bin/env python3
"""digest — 生成 LEDGER.md：账本的世界模型单页 (D-018/B4+C1)

给人与 AI 的单一入口：三个账本的紧凑视图 + 活跃决策拓扑（mermaid 机械生成）。
AI 会话开头读本文件而非逐个扫描 ledger/（上下文成本 O(1)）。
账本变更后必须重跑本脚本——--check 模式下漂移即红灯（D-021 起钩子强制，CI 同款）。
义务级历史化（superseded_by/frozen，D-023）与父决策退场同显示为"历史"。

用法:
  python tools/digest.py            # 重新生成 LEDGER.md
  python tools/digest.py --check    # 校验已提交的 LEDGER.md 是否新鲜
"""
import argparse
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


def one_line(s, n=60):
    s = " ".join(str(s or "").split())
    return s if len(s) <= n else s[: n - 1] + "…"


def build() -> str:
    goals = load(LED / "intent", "G-*.yaml")
    decisions = load(LED / "decisions", "D-*.yaml")
    triggers = load(LED / "triggers", "*.yaml")
    evidence = load(LED / "evidence", "O-*.yaml")

    obl = {}
    for did, d in decisions.items():
        for o in as_list(d.get("obligations")):
            oid = str(o.get("id"))
            obl[oid] = {"id": oid, "did": did, "desc": o.get("desc", ""),
                        "checker": o.get("checker") or {},
                        "superseded_by": o.get("superseded_by"),
                        "frozen": bool(o.get("frozen"))}
    ev = {str(e.get("obligation")): e.get("status", "—") for e in evidence.values()}

    active = {k: d for k, d in decisions.items() if d.get("status") in ("accepted", "draft")}
    retired = {k: d for k, d in decisions.items() if d.get("status") not in ("accepted", "draft")}

    lines = []
    ap = lines.append
    ap("# LEDGER — 账本世界模型（机械生成，勿手改）")
    ap("")
    ap(f"> 由 `tools/digest.py` 生成于账本状态 {len(goals)}G/{len(decisions)}D/"
       f"{len(obl)}O/{len(evidence)}E。变更账本后必须重跑；`--check` 漂移即 CI 红灯。")
    ap("> AI 会话入口：本文件 + `AGENTS.md` + `CONSTITUTION.md` + `thinking/`。")
    ap("")

    # ---------- 目标 ----------
    ap("## 目标账")
    ap("")
    ap("| 目标 | 内容 | 验收 |")
    ap("|---|---|---|")
    for gid, g in sorted(goals.items()):
        ver = ", ".join(str(x) for x in as_list(g.get("verification")))
        ap(f"| {gid} | {one_line(g.get('title'))} | {ver} |")
    ap("")

    # ---------- 活跃决策 ----------
    ap("## 活跃决策")
    ap("")
    ap("| 决策 | 状态 | 问题 → 选择 | 服务 |")
    ap("|---|---|---|---|")
    for did, d in sorted(active.items()):
        q = one_line(d.get("question"), 40)
        c = one_line(d.get("choice"), 46)
        gl = ", ".join(str(x) for x in as_list(d.get("goal")))
        ap(f"| {did} | {d.get('status')} | {q} → {c} | {gl} |")
    ap("")

    if retired:
        ap("## 历史决策")
        ap("")
        ap("| 决策 | 状态 | 去向 |")
        ap("|---|---|---|")
        for did, d in sorted(retired.items()):
            sup = d.get("superseded_by") or "—"
            ap(f"| {did} | {d.get('status')} | superseded_by={sup} |")
        ap("")

    # ---------- 义务 ----------
    ap("## 义务与证据")
    ap("")
    ap("| 义务 | 描述 | 检查器档 | 证据 | 所属 |")
    ap("|---|---|---|---|---|")
    hist = {"superseded", "rejected"}
    for oid, o in sorted(obl.items()):
        did = o["did"]
        parent = decisions.get(did, {})
        if parent.get("status") in hist or o.get("superseded_by") or o.get("frozen"):
            evs = "历史"
        else:
            evs = ev.get(oid, "缺失")
        ap(f"| {oid} | {one_line(o['desc'], 44)} | {o['checker'].get('type', '—')} | {evs} | {did} |")
    ap("")

    # ---------- 触发器 ----------
    ap("## 触发器库")
    ap("")
    for tk, t in sorted(triggers.items()):
        rules = ", ".join(str(r.get("key")) for r in as_list(t.get("obligations")))
        ap(f"- `{t.get('trigger')}`：{len(as_list(t.get('obligations')))} 条规则（{rules}）")
    ap("")

    # ---------- 桥 ----------
    bridge_p = LED / "bridge.yaml"
    if bridge_p.exists():
        b = yaml.safe_load(bridge_p.read_text(encoding="utf-8")) or {}
        sps = as_list(b.get("semantic_points"))
        nv = sum(1 for s in sps if s.get("verus"))
        ap("## 模型↔代码桥")
        ap("")
        ap(f"- 模型常量 {len(as_list(b.get('model_parameters')))}（合成道具）| "
           f"不变式/性质锚点 {sum(len(as_list(b.get(k))) for k in ('invariants', 'properties'))} | "
           f"语义点 {len(sps)}（verus 覆盖 {nv}/{len(sps)}）")
        ap("- 对账命令：`python tools/bridge_check.py`（义务 O-025）")
        ap("")

    # ---------- 元认知 ----------
    bs = len([1 for l in (ROOT / "thinking" / "blind_spots.md").read_text(encoding="utf-8").splitlines()
              if l.startswith("### BS-")])
    ms = len([1 for l in (ROOT / "thinking" / "methods.md").read_text(encoding="utf-8").splitlines()
              if l.startswith("### M-")])
    ap(f"## 元认知库")
    ap("")
    ap(f"- 盲区判例 {bs} 条（`thinking/blind_spots.md`）| 方法 {ms} 条（`thinking/methods.md`）")
    ap("")

    # ---------- mermaid 拓扑 ----------
    ap("## 活跃拓扑")
    ap("")
    ap("```mermaid")
    ap("flowchart LR")
    for gid, g in sorted(goals.items()):
        ap(f"    {gid}(('{gid}'))")
    for did, d in sorted(active.items()):
        ap(f"    {did.replace('-', '_')}['{did}']")
        for gid in as_list(d.get("goal")):
            ap(f"    {gid} --> {did.replace('-', '_')}")
        for o in as_list(d.get("obligations")):
            oid = str(o.get("id")).replace("-", "_")
            ap(f"    {did.replace('-', '_')} --> {oid}")
    for oid in sorted(obl):
        st = ev.get(oid, "缺失")
        o_rec = obl[oid]
        parent = decisions.get(o_rec["did"], {})
        cls = "hist" if (parent.get("status") in hist or o_rec.get("superseded_by")
                         or o_rec.get("frozen")) else {"pass": "ok", "deferred": "def"}.get(st, "miss")
        ap(f"    {oid.replace('-', '_')}[{oid}]")
        ap(f"    class {oid.replace('-', '_')} {cls}")
    ap("    classDef ok fill:#d4edda,stroke:#3c763d")
    ap("    classDef def fill:#fff3cd,stroke:#856404")
    ap("    classDef miss fill:#f8d7da,stroke:#a94442")
    ap("    classDef hist fill:#e2e3e5,stroke:#6c757d,stroke-dasharray: 5 5")
    ap("```")
    ap("")
    return "\n".join(lines)


def main():
    ap_ = argparse.ArgumentParser(description="生成/校验 LEDGER.md")
    ap_.add_argument("--check", action="store_true", help="校验已存在文件的新鲜度")
    args = ap_.parse_args()

    content = build()
    target = ROOT / "LEDGER.md"
    if args.check:
        current = target.read_text(encoding="utf-8") if target.exists() else ""
        if current != content:
            print("ERROR LEDGER.md 与账本当前状态不一致——请重跑 python tools/digest.py 并提交")
            return 1
        print("RESULT: PASS —— LEDGER.md 新鲜")
        return 0
    target.write_text(content, encoding="utf-8", newline="\n")
    print(f"已生成 {target}（{len(content.splitlines())} 行）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
