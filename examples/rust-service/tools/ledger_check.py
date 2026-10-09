#!/usr/bin/env python3
"""ledger-check — 检查器运行器 (D-015/A3)

按 evidence 声明的 checker 档位实际复跑检查器，让 L2 的 PASS 从手填变成可再生。
检查器阶梯（宪法原文）：lint/约定 → 架构测试 → 类型/契约 → 压力/性质测试 → 模型检查 → 机器证明。

用法:
  python tools/ledger_check.py --list          # 全部义务 → 检查器映射（冒烟）
  python tools/ledger_check.py O-011           # 复跑某义务的检查器
  python tools/ledger_check.py --type proof    # 复跑某一档全部义务
"""
import argparse
import subprocess
import sys
import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
LED = ROOT / "ledger"


def load_config():
    with open(ROOT / "tools" / "config.toml", "rb") as f:
        return tomllib.load(f)


def as_list(x):
    return x if isinstance(x, list) else ([] if x is None else [x])


def collect():
    """返回 {oid: {desc, checker, did, status}}，仅当前约束中的义务。

    排除：父决策退场（superseded/rejected）与义务级历史化
    （superseded_by 被取代 / frozen 归档冻结，D-023）。
    """
    out = {}
    for p in sorted((LED / "decisions").glob("D-*.yaml")):
        d = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        if d.get("status") not in ("accepted", "draft"):
            continue
        for o in as_list(d.get("obligations")):
            if o.get("superseded_by") or o.get("frozen"):
                continue
            out[str(o.get("id"))] = {
                "desc": o.get("desc", ""), "checker": o.get("checker") or {},
                "did": d.get("id"), "status": d.get("status"),
            }
    return out


def exec_checker(oid, item, cfg):
    """按 checker.type 分发执行。返回 (ok, 输出摘要)。"""
    ck = item["checker"]
    typ = ck.get("type")
    cargo_dir = ROOT / cfg["pilot-rs"]["dir"]

    def run(cmd, cwd=ROOT):
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                           shell=True, encoding="utf-8", errors="replace")
        return r.returncode == 0, (r.stdout + r.stderr).strip()[-1200:]

    if typ == "test":
        if ck.get("cmd"):
            # checker.cmd 支持（D-021）：{PY} 占位符替换为 config 单点路径
            return run(str(ck["cmd"]).replace("{PY}", cfg["tools"]["python"]))
        name = ck.get("name")
        cmd = f'cargo test {name}' if name else "cargo test"
        return run(cmd, cwd=cargo_dir)
    if typ == "bench":
        name = ck.get("name", "--ignored")
        return run(f"cargo test --release -- --ignored {name}".strip(), cwd=cargo_dir)
    if typ == "lint":
        # 内置规则档：账目 lint 自身即 lint 档检查器；带 name 分发到专项对账工具
        if ck.get("name") == "bridge":
            return run(f'{cfg["tools"]["python"]} tools/bridge_check.py')
        if ck.get("name") == "digest":
            return run(f'{cfg["tools"]["python"]} tools/digest.py --check')
        return run(f'{cfg["tools"]["python"]} tools/ledger_lint.py')
    if typ == "model":
        jar = ROOT / cfg["tools"]["tla2tools"]
        # TLC 按文件名解析模块，必须在 tla/ 目录内运行
        return run(f'{cfg["tools"]["java"]} -cp {jar} tlc2.TLC -cleanup queue.tla',
                   cwd=ROOT / "tla")
    if typ == "proof":
        target = ck.get("target", "proofs/queue_core.rs")
        vcfg = cfg["tools"].get("verus_cfg", "verus_verify")
        return run(f'{cfg["tools"]["verus"]} --cfg {vcfg} {target}')
    if typ == "manual":
        return None, "manual 档不可机器复跑——评审签名在 evidence.notes / checker.ref"
    return None, f"未知 checker.type {typ!r}"


def main():
    ap = argparse.ArgumentParser(description="Spec Ledger 检查器运行器")
    ap.add_argument("oid", nargs="?", help="义务 id，如 O-011")
    ap.add_argument("--type", dest="typ", help="复跑某一档全部义务")
    ap.add_argument("--list", action="store_true", help="冒烟：打印义务→检查器映射")
    args = ap.parse_args()

    items = collect()
    cfg = load_config()

    if args.list:
        print(f"== 检查器映射（{len(items)} 条在约束义务）==")
        for oid in sorted(items):
            it = items[oid]
            ck = it["checker"]
            print(f"{oid} [{ck.get('type', '—'):6}] ({it['did']}) {it['desc'][:48]}")
        return 0

    if args.oid:
        if args.oid not in items:
            print(f"未找到在约束义务 {args.oid}（可能已随父决策历史化）")
            return 1
        targets = {args.oid: items[args.oid]}
    elif args.typ:
        targets = {o: it for o, it in items.items() if it["checker"].get("type") == args.typ}
        if not targets:
            print(f"该档无在约束义务: {args.typ}")
            return 1
    else:
        ap.print_usage()
        return 1

    n_fail = 0
    for oid, it in sorted(targets.items()):
        ok, out = exec_checker(oid, it, cfg)
        if ok is None:
            print(f"[SKIP] {oid} — {out}")
        elif ok:
            print(f"[PASS] {oid} — {out.splitlines()[-1] if out else ''}")
        else:
            n_fail += 1
            print(f"[FAIL] {oid}\n{out}")
    return 1 if n_fail else 0


if __name__ == "__main__":
    sys.exit(main())
