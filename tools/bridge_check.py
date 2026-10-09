#!/usr/bin/env python3
"""bridge-check — 模型↔代码机械对账 (D-016/A1)

BS-010（验证器信任转移）的结构防线：TLA+ 查协议不查代码，Verus 查核心不查并发，
两者的结论要落到实现上，桥必须被机械检查而不是靠评审记忆。

对账三律（见 ledger/bridge.yaml 头注）:
  B1  模型常量全部登记为 model_parameters（或显式声明与实现常量的绑定）
  B2  cfg 检查的每条 INVARIANT/PROPERTY 在桥表中有概念+义务锚点，且义务真实存在
  B3  每个语义点的 tla/rust/verus 符号在各自文件中真实存在

用法: python tools/bridge_check.py [--root PATH]    退出码 0=对账平衡 1=漂移
"""
import argparse
import re
import sys
from pathlib import Path

import yaml

FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")
TLA_DEF = re.compile(r"^([A-Za-z][A-Za-z0-9_]*)\s*(\([^)]*\))?\s*==", re.M)
TLA_CONST = re.compile(r"\b([A-Z][A-Z0-9_]*)\s*=\s*(\S+)")


def symbols_in(path: Path):
    text = path.read_text(encoding="utf-8")
    return text, set(FN.findall(text))


def main():
    ap = argparse.ArgumentParser(description="模型↔代码桥对账")
    ap.add_argument("--root", default=str(Path(__file__).resolve().parent.parent))
    args = ap.parse_args()
    root = Path(args.root).resolve()

    bridge = yaml.safe_load((root / "ledger" / "bridge.yaml").read_text(encoding="utf-8"))
    errors, warns = [], []

    # ---- 1. 模型文件与常量 ----
    tla_path = root / bridge["model"]
    cfg_path = root / bridge["config"]
    if not tla_path.exists():
        errors.append(f"模型文件缺失 {bridge['model']}")
        print("\n".join(f"ERROR {e}" for e in errors) or "OK")
        return 1
    tla_text = tla_path.read_text(encoding="utf-8")
    tla_syms = {m.group(1) for m in TLA_DEF.finditer(tla_text)}
    cfg_text = cfg_path.read_text(encoding="utf-8")
    consts = {m.group(1): m.group(2)
              for m in TLA_CONST.finditer(cfg_text) if m.group(1) != "CONSTANT"}
    declared_params = set(bridge.get("model_parameters") or [])
    for c in sorted(consts):
        if c not in declared_params:
            errors.append(f"B1 — 模型常量 {c} 未登记 model_parameters（合成道具未声明）")

    # ---- 2. 实现侧文件 ----
    rust_syms = {}
    for rel in bridge.get("rust") or []:
        p = root / rel
        if not p.exists():
            errors.append(f"B3 — rust 文件缺失 {rel}")
            continue
        rust_syms[rel] = symbols_in(p)[1]
    verus_syms = {}
    for rel in bridge.get("verus") or []:
        p = root / rel
        if not p.exists():
            errors.append(f"B3 — verus 文件缺失 {rel}")
            continue
        verus_syms[rel] = symbols_in(p)[1]

    # ---- 3. 不变式/性质锚点（B2）----
    checked = ([("invariant", m.group(1)) for m in
                re.finditer(r"^INVARIANT\s+(\w+)", cfg_text, re.M)]
               + [("property", m.group(1)) for m in
                  re.finditer(r"^PROPERTY\s+(\w+)", cfg_text, re.M)])
    mapped = {entry["tla"] for k in ("invariants", "properties")
              for entry in (bridge.get(k) or [])}
    for kind, name in checked:
        if name not in mapped:
            errors.append(f"B2 — cfg 检查的{kind} {name} 未在桥表登记（检查了却无人认领）")

    # 义务存在性
    dec_dir = root / "ledger" / "decisions"
    obl_ids = set()
    for p in dec_dir.glob("D-*.yaml"):
        d = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        for o in d.get("obligations") or []:
            obl_ids.add(str(o.get("id")))
    ev_dir = root / "ledger" / "evidence"
    ev_status = {}
    for p in ev_dir.glob("O-*.yaml"):
        e = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        ev_status[str(e.get("obligation"))] = e.get("status")

    for k in ("invariants", "properties"):
        for entry in bridge.get(k) or []:
            obl = entry.get("obligation")
            if obl and obl not in obl_ids:
                errors.append(f"B2 — {entry['tla']} 锚定的义务 {obl} 不存在于决策账")

    # ---- 4. 语义点符号存在性（B3）----
    for sp in bridge.get("semantic_points") or []:
        spid, concept = sp.get("id"), sp.get("concept", "")
        tla_ref = sp.get("tla")
        for name in ([tla_ref] if isinstance(tla_ref, str) else list(tla_ref or [])):
            if name not in tla_syms:
                errors.append(f"B3 — {spid}({concept}): TLA⁺ 符号 {name} 不在 {bridge['model']}")
        rust_sym = sp.get("rust")
        if rust_sym:
            if not any(rust_sym in syms for syms in rust_syms.values()):
                errors.append(f"B3 — {spid}({concept}): rust 符号 fn {rust_sym} 未找到")
        verus_sym = sp.get("verus")
        if verus_sym:
            if not any(verus_sym in syms for syms in verus_syms.values()):
                errors.append(f"B3 — {spid}({concept}): verus 符号 fn {verus_sym} 未找到")
        obl = sp.get("obligation")
        if obl and obl not in obl_ids:
            errors.append(f"B3 — {spid}({concept}): 义务 {obl} 不存在")
        elif obl and ev_status.get(obl) != "pass":
            warns.append(f"{spid}({concept}): 义务 {obl} 证据未 PASS（{ev_status.get(obl, '缺失')}）")

    # ---- 报告 ----
    n_sp = len(bridge.get("semantic_points") or [])
    print(f"== 桥对账 == 模型常量 {len(consts)} | 检查中的不变式/性质 {len(checked)} | "
          f"语义点 {n_sp} | 语义点 verus 覆盖 "
          f"{sum(1 for s in bridge['semantic_points'] if s.get('verus'))}/{n_sp}")
    for w in warns:
        print(f"WARN  {w}")
    for e in errors:
        print(f"ERROR {e}")
    if errors:
        print(f"RESULT: FAIL（{len(errors)} error, {len(warns)} warn）——模型与实现漂移")
        return 1
    print(f"RESULT: PASS（0 error, {len(warns)} warn）——模型↔代码桥平衡")
    return 0


if __name__ == "__main__":
    sys.exit(main())
