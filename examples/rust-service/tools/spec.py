#!/usr/bin/env python3
"""spec — Spec Ledger 工具链 CLI v0（D-024/D-025）

把"工程纪律"编译为命令的片段：
  init    按 WORKFLOW 第 0 步生成新项目五件套（账本空置——禁止 BDUF，不预写任何决策）
  expand  展开决策草稿的触发器义务存根（只读，输出到 stdout，由人/AI 并入 D 草案）
  check   统一检查器调用：复用 ledger_check 分发器，输出归一化证据记录
          （spec-evidence/v0：oid/did/checker/verdict[pass|fail|skip]/started/detail）
          ——任何验证器只要可被 exec 协议调用即插即用（叶子），证据是数据

用法:
  python tools/spec.py init <目标目录> <项目名>
  python tools/spec.py expand D-021 [--root PATH]
  python tools/spec.py check O-029 [--root PATH]
  python tools/spec.py check --type manual [--root PATH]
"""
import argparse
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

import yaml

TEMPLATE = Path(__file__).resolve().parent.parent

COPY_FILES = [
    "CONSTITUTION.md", "AGENTS.md", "WORKFLOW.md", ".gitignore",
    "experiments/protocol.md",
]
COPY_TOOL_PY = ["ledger_lint.py", "ledger_check.py", "bridge_check.py", "digest.py",
                "trace.py", "gate.py", "spec.py", "config.toml"]
COPY_DIRS = [
    ("thinking", ["*.md"]),
    ("ledger/triggers", ["*.yaml", "*.md"]),
]
COPY_TREES = ["thinking/methodology"]
MKDIRS = ["ledger/intent", "ledger/decisions", "ledger/evidence", "ledger/triggers",
          "tools/tests", "thinking", "experiments", ".githooks", ".github/workflows"]


def _sh(cmd, cwd):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if r.returncode != 0:
        print(f"ERROR {' '.join(cmd)}\n{r.stdout}{r.stderr}", file=sys.stderr)
        sys.exit(1)


def cmd_init(args):
    target = Path(args.dir).resolve()
    if target.exists() and any(target.iterdir()):
        print(f"ERROR {target} 已存在且非空", file=sys.stderr)
        return 1
    target.mkdir(parents=True)
    for d in MKDIRS:
        (target / d).mkdir(parents=True, exist_ok=True)
    for f in COPY_FILES:
        shutil.copy(TEMPLATE / f, target / f)
    for f in COPY_TOOL_PY:
        shutil.copy(TEMPLATE / "tools" / f, target / "tools" / f)
    for sub, pats in COPY_DIRS:
        for pat in pats:
            for p in (TEMPLATE / sub).glob(pat):
                shutil.copy(p, target / sub / p.name)
    for tree in COPY_TREES:
        shutil.copytree(TEMPLATE / tree, target / tree, dirs_exist_ok=True)
    shutil.copy(TEMPLATE / "tools" / "tests" / "test_ledger_lint.py",
                target / "tools" / "tests" / "test_ledger_lint.py")
    shutil.copy(TEMPLATE / ".githooks" / "pre-commit", target / ".githooks" / "pre-commit")
    shutil.copy(TEMPLATE / ".github" / "workflows" / "ci.yml",
                target / ".github" / "workflows" / "ci.yml")

    # config.toml：python 用当前解释器（可移植）；重型验证器路径留待按需配置
    cfg = (TEMPLATE / "tools" / "config.toml").read_text(encoding="utf-8")
    lines = []
    for ln in cfg.splitlines():
        if ln.startswith("python = "):
            exe = str(sys.executable).replace("\\", "/")
            lines.append(f'python = "{exe}"')
        else:
            lines.append(ln)
    (target / "tools" / "config.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")

    # 预生成世界模型单页（空账本也有 O(1) 入口）；桥未建，gate 自动跳过桥门
    _sh([sys.executable, "tools/digest.py"], target)
    _sh(["git", "init", "-q"], target)
    _sh(["git", "config", "core.hooksPath", ".githooks"], target)

    print(f"已生成 {args.name} 于 {target}")
    print("五件套就位，账本空置（禁止 BDUF）。接下来按 WORKFLOW：")
    print("  1. 改 AGENTS.md——主语言、本机命令段；核对 tools/config.toml 机器路径")
    print("  2. 你写一段话（引擎要什么/给谁用/什么数字算成功）→ AI 起草 G-*.yaml（过五问）")
    print("  3. AI 起草 D-001 认领工具链（artifacts = 全部继承文件）→ 你裁决 accepted")
    print("  4. 首提：chore: 项目初始化（四门将首次实战）")
    return 0


def cmd_expand(args):
    root = Path(args.root).resolve()
    dpath = root / "ledger" / "decisions" / f"{args.did}.yaml"
    if not dpath.exists():
        print(f"ERROR 未找到 {dpath}", file=sys.stderr)
        return 1
    d = yaml.safe_load(dpath.read_text(encoding="utf-8")) or {}
    satisfied = {str(o.get("satisfies")) for o in d.get("obligations") or []}
    waived = {str(w.get("id")) for w in d.get("waivers") or []}
    introduced = [str(k) for k in d.get("introduces") or []]
    if not introduced:
        print(f"{args.did} 无 introduces——无触发器义务需展开")
        return 0
    stubs, covered, waived_out = [], [], []
    # 按触发器键分组输出
    trig_of = {}
    for p in sorted((root / "ledger" / "triggers").glob("*.yaml")):
        t = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        for r in t.get("obligations") or []:
            trig_of[str(r.get("key"))] = (t.get("trigger"), r)
    for key in introduced:
        for rk, (tk, r) in sorted(trig_of.items()):
            if tk != key:
                continue
            if rk in satisfied:
                covered.append(rk)
            elif rk in waived:
                waived_out.append(rk)
            else:
                stubs.append(r)
    print(f"# {args.did} 触发器展开（只读输出，并入 obligations 前需逐条指定 checker 档位）")
    for rk in covered:
        print(f"#   已覆盖: {rk}")
    for rk in waived_out:
        print(f"#   已豁免: {rk}")
    for r in stubs:
        print(f"""  - id: O-???
    desc: {r.get('desc', '')}
    satisfies: {r.get('key')}
    checker:
      type: manual
      ref: TODO——按检查器阶梯选起步档（宪法：从最低档开始，允许升级）""")
    if not stubs:
        print("#   无未覆盖义务——该决策的触发器义务已全部落实或豁免")
    return 0


def cmd_check(args):
    root = Path(args.root).resolve()
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import ledger_check as lc  # noqa: E402 —— 同目录工具，ROOT 随其文件定位

    items = lc.collect()
    cfg = lc.load_config()
    if args.oid:
        if args.oid not in items:
            print(f"ERROR 未找到在约束义务 {args.oid}", file=sys.stderr)
            return 1
        targets = {args.oid: items[args.oid]}
    elif args.typ:
        targets = {o: it for o, it in items.items()
                   if it["checker"].get("type") == args.typ}
        if not targets:
            print(f"ERROR 该档无在约束义务: {args.type}", file=sys.stderr)
            return 1
    else:
        print("ERROR 需要 oid 或 --type", file=sys.stderr)
        return 1
    results = []
    for oid, it in sorted(targets.items()):
        ok, out = lc.exec_checker(oid, it, cfg)
        results.append({
            "oid": oid, "did": it["did"], "checker": it["checker"],
            "verdict": "skip" if ok is None else ("pass" if ok else "fail"),
            "started": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "detail": (out or "")[-400:],
        })
    print(json.dumps({"schema": "spec-evidence/v0", "results": results},
                     ensure_ascii=False, indent=2))
    return 0 if all(r["verdict"] != "fail" for r in results) else 1


def main():
    ap = argparse.ArgumentParser(description="Spec Ledger 工具链 CLI（D-024/D-025）")
    sub = ap.add_subparsers(dest="cmd", required=True)
    p_init = sub.add_parser("init", help="生成新项目五件套（WORKFLOW 第 0 步）")
    p_init.add_argument("dir")
    p_init.add_argument("name")
    p_exp = sub.add_parser("expand", help="展开决策的触发器义务存根（只读）")
    p_exp.add_argument("did", help="决策 id，如 D-021")
    p_exp.add_argument("--root", default=".", help="账本仓库根（默认当前目录）")
    p_chk = sub.add_parser("check", help="统一检查器调用（spec-evidence/v0 输出）")
    p_chk.add_argument("oid", nargs="?", help="义务 id，如 O-029")
    p_chk.add_argument("--type", dest="typ", help="批量：某一检查器档（test/lint/proof/…）")
    p_chk.add_argument("--root", default=".", help="账本仓库根（默认当前目录）")
    args = ap.parse_args()
    return {"init": cmd_init, "expand": cmd_expand, "check": cmd_check}[args.cmd](args)


if __name__ == "__main__":
    sys.exit(main())
