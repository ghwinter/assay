#!/usr/bin/env python3
"""gate — 提交门四合一（O-032，D-021）

pre-commit 薄壳与 CI ledger job 共用的同一检查器集：账目 lint / 桥对账 /
digest 新鲜 / 守门人自测。任一失败即非零退出（红灯）。
强制层本地化：本仓库无 remote，CI 档由本门在提交时兑现（BS-013 判例的硬化）。
"""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GATES = [
    ("账目平衡（L1/L2/L3）", ["tools/ledger_lint.py"]),
    # 新项目尚未建桥（bridge.yaml 不存在）时跳过——桥随第一个模型检查器引入
    ("模型↔代码桥对账", ["tools/bridge_check.py"]
     if (ROOT / "ledger" / "bridge.yaml").exists() else None),
    ("LEDGER.md 新鲜度", ["tools/digest.py", "--check"]),
    ("守门人自测（黄金用例）", ["-m", "unittest", "discover", "-s", "tools/tests"]),
]


def main():
    failed = []
    for gate in GATES:
        if gate is None or gate[1] is None:
            continue
        name, args = gate
        r = subprocess.run([sys.executable] + args, cwd=ROOT)
        if r.returncode != 0:
            failed.append(name)
    if failed:
        print(f"GATE FAIL: {'; '.join(failed)}")
        return 1
    print("GATE PASS: lint / bridge / digest / self-test")
    return 0


if __name__ == "__main__":
    sys.exit(main())
