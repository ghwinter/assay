#!/usr/bin/env python3
"""崩溃证据自动采集（O-022，D-011）

用法: python tools/debug/crash_evidence.py <exe> [args...]
用 cdb 运行目标进程；目标 abort/断言失败/访问违例时，cdb 在二次机会异常处停住，
脚本捕获异常类型与符号化 backtrace，写入 stdout（可重定向入账 ledger/evidence/）。
cdb 路径单点 tools/config.toml（D-021）。
"""
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def cdb():
    cfg = tomllib.load(open(ROOT / "tools" / "config.toml", "rb"))
    return cfg["tools"]["cdb"]


def main():
    if len(sys.argv) < 2:
        print("usage: crash_evidence.py <exe> [args...]")
        return 2
    exe = sys.argv[1]
    # g: 跳过初始断点；崩溃（二次机会异常）时 cdb 自动停住 → k 12 抓 backtrace → qd 退出
    out = subprocess.run(
        [cdb(), "-c", "g; k 12; qd", exe] + sys.argv[2:],
        capture_output=True, text=True, timeout=600,
    )
    lines = out.stdout.splitlines()
    # 证据 = 从最后一次异常停住处开始（初始断点也有异常字样，故取最后一次）
    start = 0
    for i, ln in enumerate(lines):
        low = ln.lower()
        if ("exception" in low and "hit" in low) or "second chance" in low:
            start = i
    print(f"=== crash evidence: {Path(exe).name} ===")
    print("\n".join(lines[start:start + 20]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
