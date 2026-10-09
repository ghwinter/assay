#!/usr/bin/env python3
"""close/关停路径断点快照（O-023，D-011）

用法: python tools/debug/close_snapshot.py <exe 或测试目标名>
在 BoundedQueue*::close 断点处抓取命中记录与符号化调用链。
注意：cdb 泛型符号不能用尖括号直接 bp；行断点因泛型实例化歧义会报错；
必须用 module!bounded_queue::BoundedQueue*::close* 收窄通配符（D-011 留档的坑）。
参数为无后缀目标名（如 queue）时按 mtime 解析 target/debug/deps/ 下最新
<name>-*.exe——构建产物 hash 不再入账（BS-013，D-021）。cdb 路径单点 config.toml。
"""
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def cdb():
    cfg = tomllib.load(open(ROOT / "tools" / "config.toml", "rb"))
    return cfg["tools"]["cdb"]


def resolve_exe(arg: str) -> Path:
    p = Path(arg)
    if p.suffix == ".exe" and p.exists():
        return p
    base = ROOT / "pilot-rs" / "target" / "debug" / "deps"
    cands = sorted(base.glob(f"{arg}-*.exe"), key=lambda q: q.stat().st_mtime)
    if not cands:
        raise SystemExit(f"未找到测试二进制 {arg}-*.exe（先 cargo test --no-run 构建）")
    return cands[-1]


def main():
    if len(sys.argv) != 2:
        print("usage: close_snapshot.py <exe 或测试目标名>")
        return 2
    exe = resolve_exe(sys.argv[1])
    module = exe.stem.replace("-", "_")
    pat = f"{module}!bounded_queue::BoundedQueue*::close*"
    out = subprocess.run(
        [cdb(), "-c", f"bm {pat}; g; k 8; qd", str(exe)],
        capture_output=True, text=True, timeout=600,
    )
    text = out.stdout
    ok = "Breakpoint" in text and "bounded_queue::BoundedQueue" in text
    idx = text.find("Breakpoint")
    print(f"=== close snapshot: {exe.name} (ok={ok}) ===")
    print(text[idx:idx + 1200] if idx >= 0 else text[-1500:])
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
