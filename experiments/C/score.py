#!/usr/bin/env python3
"""实验 C 评分：agent 分类 vs 人工金标准

用法: python experiments/C/score.py [agent_labels.txt gold_labels.txt]
缺省用本目录下的 agent_labels.txt 与 gold_labels.txt。
输出: 总体 accuracy + 每类 precision/recall/F1（重点类：invalidated 与 newly-required）。
"""
import sys
from pathlib import Path

VALID = {"still-valid", "invalidated", "newly-required", "unaffected", "unknown"}
HERE = Path(__file__).resolve().parent


def load(p: Path):
    d = {}
    for line in p.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) >= 2 and parts[1] in VALID:
            d[parts[0]] = parts[1]
    return d


def prf(gold, pred, label):
    tp = sum(1 for k in gold if gold[k] == label and pred.get(k) == label)
    fp = sum(1 for k in pred if pred[k] == label and gold.get(k) != label)
    fn = sum(1 for k in gold if gold[k] == label and pred.get(k) != label)
    p = tp / (tp + fp) if tp + fp else 0.0
    r = tp / (tp + fn) if tp + fn else 0.0
    f = 2 * p * r / (p + r) if p + r else 0.0
    return p, r, f


def main():
    a = load(HERE / "agent_labels.txt")
    g = load(HERE / "gold_labels.txt")
    if not g:
        print("gold_labels.txt 不存在或全为 TODO——请先按 gold_labels_TEMPLATE.txt 人工标注")
        return 1
    common = sorted(set(a) & set(g))
    only_a = sorted(set(a) - set(g))
    agree = sum(1 for k in common if a[k] == g[k])
    print(f"比对实体数: {len(common)}（agent 独有: {len(only_a)} {only_a}）")
    print(f"总体 accuracy: {agree}/{len(common)} = {agree/len(common):.3f}")
    print(f"{'label':<16}{'precision':>10}{'recall':>10}{'F1':>8}")
    for label in ("invalidated", "newly-required", "still-valid", "unaffected", "unknown"):
        p, r, f = prf(g, a, label)
        print(f"{label:<16}{p:>10.3f}{r:>10.3f}{f:>8.3f}")
    wrong = [(k, a[k], g[k]) for k in common if a[k] != g[k]]
    if wrong:
        print("\n分歧明细（id, agent, gold）:")
        for k, pa, pg in wrong:
            print(f"  {k}: agent={pa}  gold={pg}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
