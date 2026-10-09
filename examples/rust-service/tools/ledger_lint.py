#!/usr/bin/env python3
"""ledger-lint — Spec Ledger 账目平衡检查 (v1)

规则（见 CONSTITUTION.md）:
  L1  ∀ artifact   ∃ decision        没有无主的代码
  L2  ∀ obligation ∃ checker ∧ PASS  没有裸奔的义务（deferred 需 reason+owner → WARN）
  L3  ∀ goal       ∃ evidence        没有无人验收的目标
外加：schema 校验、引用完整性、触发器覆盖（或带签名的豁免）。

用法:
  python tools/ledger_lint.py [--root PATH]
退出码: 0 = PASS，1 = 存在 error

v1（D-015）：校验逻辑拆为可导入的 run(root)，供 tools/tests/ 黄金用例自测；
L1 代码图案扩展到 tools/**/*.py、tools/*.toml、.githooks/*（守门人自身入账）。
v1.1（D-022/D-023）：rationale.anchor 路径形态存在性检查；义务出身 origin
（tool|trigger，与 satisfies 皆缺才 WARN——消灭设计性永久告警）；义务级
superseded_by / frozen 生命周期（历史义务随父决策、后继义务或归档冻结退场）；
G 账 status 必填（active|withdrawn|superseded）。
"""
import argparse
import re
import sys
from pathlib import Path

import yaml

OID = re.compile(r"^O-\d{3}$")
GID = re.compile(r"^G-\d{3}$")
DID = re.compile(r"^D-\d{3}$")
D_STATUS = {"draft", "accepted", "superseded", "rejected"}
CHECKER_TYPES = {"test", "bench", "lint", "manual", "model", "proof"}
ORIGINS = {"tool", "trigger"}
G_STATUS = {"active", "withdrawn", "superseded"}
_ANCHOR_PATH = re.compile(r"[A-Za-z0-9_./\\-]+")
BS_ID = re.compile(r"^BS-\d{3}$")


def blind_spot_ids(root: Path):
    """判例库里已升库的 BS 编号集合。库不存在时返回 None（无声明则不需要库）。

    作用域自证（BS-C1「空作用域通过」）：库存在却解析出零条判例时，调用方须报错，
    不得把"零条"当作"全部通过"。
    """
    p = root / "thinking" / "blind_spots.md"
    if not p.exists():
        return None
    return {m.group(1) for m in re.finditer(r"^###\s+(BS-\d{3})", p.read_text(encoding="utf-8"), re.M)}


def _check_anchor(root: Path, did: str, anchor: str, rep) -> None:
    """anchor 的路径形态必须真实存在（D-022：出处指针可机械核对）。纯词/URL 跳过。"""
    m = _ANCHOR_PATH.match(anchor)
    if not m:
        return
    cand = m.group(0).rstrip(".")
    if ("/" not in cand and "\\" not in cand and "." not in cand) \
            or cand.startswith(("http://", "https://")):
        return
    if not (root / cand.replace("\\", "/")).exists():
        rep.warn(f"{did}: anchor 路径不存在（仓库外绝对路径或笔误）— {cand}")
CODE_PATTERNS = [
    "pilot-rs/src/**/*.rs",
    "pilot-rs/tests/**/*.rs",
    "pilot-rs/examples/**/*.rs",
    "pilot-rs/*.toml",
    "pilot/src/**/*.hpp",
    "pilot/src/**/*.cpp",
    "pilot/tests/**/*.cpp",
    "pilot/*.sh",
    "tla/*.tla",
    "proofs/**/*.rs",
    "tools/*.py",
    "tools/**/*.py",
    "tools/*.toml",
    ".githooks/*",
]


class Rep:
    def __init__(self):
        self.errors = []
        self.warns = []

    def error(self, msg):
        self.errors.append(msg)

    def warn(self, msg):
        self.warns.append(msg)


def load_all(directory, pattern):
    out = []
    if not directory.exists():
        return out
    for p in sorted(directory.glob(pattern)):
        try:
            data = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        except Exception as e:  # noqa: BLE001
            data = {"_parse_error": str(e)}
        out.append((p, data))
    return out


def as_list(x):
    return x if isinstance(x, list) else ([] if x is None else [x])


def run(root: Path) -> Rep:
    """校验 root 处的账本，返回 Rep（errors/warns）。黄金用例与 CLI 共用。"""
    led = root / "ledger"
    rep = Rep()

    goal_list = load_all(led / "intent", "G-*.yaml")
    dec_list = load_all(led / "decisions", "D-*.yaml")
    trg_list = load_all(led / "triggers", "*.yaml")
    ev_list = load_all(led / "evidence", "O-*.yaml")

    # ---------- 目标账 ----------
    goal_ids = {}
    for p, g in goal_list:
        gid = str(g.get("id", ""))
        if g.get("_parse_error"):
            rep.error(f"{p.name}: YAML 解析失败 — {g['_parse_error']}")
            continue
        if not GID.match(gid):
            rep.error(f"{p.name}: 非法目标 id {gid!r}")
            continue
        if gid in goal_ids:
            rep.error(f"{p.name}: 目标 id 重复 {gid}")
        goal_ids[gid] = g
        for f in ("title", "description"):
            if not g.get(f):
                rep.error(f"{gid}: 缺少 {f}")
        if g.get("status") not in G_STATUS:
            rep.error(f"{gid}: 缺少 status 或非法 {g.get('status')!r}"
                      f"（{sorted(G_STATUS)}，D-023）")
        ver = [str(x) for x in as_list(g.get("verification"))]
        if not ver:
            rep.error(f"{gid}: L3 — verification 为空，目标无人验收")
        for o in ver:
            if not OID.match(o):
                rep.error(f"{gid}: verification 引用非法义务 id {o!r}")
        for q in as_list(g.get("open_questions")):
            if not str(q).strip():
                rep.error(f"{gid}: open_questions 条目为空（D-024）")

    # ---------- 决策账 ----------
    dec_ids = {}
    obligations = []
    seen_oids = set()
    owned_artifacts = set()
    for p, d in dec_list:
        did = str(d.get("id", ""))
        if d.get("_parse_error"):
            rep.error(f"{p.name}: YAML 解析失败 — {d['_parse_error']}")
            continue
        if not DID.match(did):
            rep.error(f"{p.name}: 非法决策 id {did!r}")
            continue
        if did in dec_ids:
            rep.error(f"{p.name}: 决策 id 重复 {did}")
        dec_ids[did] = d
        status = d.get("status")
        if status not in D_STATUS:
            rep.error(f"{did}: 非法 status {status!r}（{sorted(D_STATUS)}）")
            status = None
        for f in ("question", "choice"):
            if not d.get(f):
                rep.error(f"{did}: 缺少 {f}")
        if not as_list(d.get("alternatives")):
            rep.error(f"{did}: alternatives 为空——没有备选的决策不是决策")
        rat = as_list(d.get("rationale"))
        if not rat:
            rep.error(f"{did}: rationale 为空——决策必须有理由")
        for item in rat:
            if isinstance(item, dict):
                if not item.get("text"):
                    rep.error(f"{did}: rationale 条目缺少 text")
                elif not item.get("anchor"):
                    rep.warn(f"{did}: rationale 缺少 anchor（软声明无签名）— {str(item.get('text'))[:32]}")
                else:
                    _check_anchor(root, did, str(item["anchor"]), rep)
        grefs = [str(x) for x in as_list(d.get("goal"))]
        if not grefs:
            rep.error(f"{did}: goal 为空——决策必须服务至少一个目标")
        for gref in grefs:
            if gref not in goal_ids:
                rep.error(f"{did}: goal 引用不存在 {gref!r}")
        sup = d.get("superseded_by")
        if status == "superseded" and not sup:
            rep.error(f"{did}: status=superseded 但缺少 superseded_by")
        if sup and not DID.match(str(sup)):
            rep.error(f"{did}: superseded_by 非法 {sup!r}")
        for a in as_list(d.get("assumptions")):
            if isinstance(a, dict):
                for f in ("text", "invalidation"):
                    if not a.get(f):
                        rep.error(f"{did}: assumptions 条目缺少 {f}（什么证据会推翻我，D-024）")
            elif not str(a).strip():
                rep.error(f"{did}: assumptions 条目为空（D-024）")
        oseen = set()
        for o in as_list(d.get("obligations")):
            oid = str(o.get("id", ""))
            if not OID.match(oid):
                rep.error(f"{did}: 非法义务 id {oid!r}")
                continue
            if oid in oseen:
                rep.error(f"{did}: 义务 id 重复 {oid}")
            oseen.add(oid)
            if oid in seen_oids:
                rep.error(f"{did}: 义务 id 跨决策重复 {oid}")
            seen_oids.add(oid)
            if not o.get("desc"):
                rep.error(f"{oid}: 缺少 desc")
            sup_o = o.get("superseded_by")
            if sup_o is not None and not OID.match(str(sup_o)):
                rep.error(f"{oid}: superseded_by 非法 {sup_o!r}")
            org = o.get("origin")
            if org is not None and org not in ORIGINS:
                rep.error(f"{oid}: origin 非法 {org!r}（{sorted(ORIGINS)}，D-023）")
            # bs_guard / bs_note（D-027：盲区库由清单升为门）
            bg = o.get("bs_guard")
            if bg is None:
                rep.error(f"{oid}: 缺少 bs_guard——新义务必须登记它防的是哪条判例，"
                          f"或给出空列表并写明 bs_note（D-027）")
            elif not isinstance(bg, list):
                rep.error(f"{oid}: bs_guard 必须是列表（如 [BS-012] 或 []，D-027）")
            else:
                guards = [str(x) for x in bg]
                for g in guards:
                    if not BS_ID.match(g):
                        rep.error(f"{oid}: bs_guard 非法值 {g!r}（形如 BS-012，D-027）")
                if not guards:
                    note = o.get("bs_note")
                    if not note or not str(note).strip():
                        rep.error(f"{oid}: bs_guard 为空时必须给 bs_note 说明理由（D-027）")
                else:
                    known = blind_spot_ids(root)
                    if known is None:
                        rep.error(f"{oid}: 声明了判例防护，但判例库 thinking/blind_spots.md "
                                  f"不存在——无法核对（D-027）")
                    elif not known:
                        rep.error(f"{oid}: 判例库存在却解析出零条判例——作用域为空，"
                                  f"拒绝静默通过（BS-C1/D-027）")
                    else:
                        for g in guards:
                            if BS_ID.match(g) and g not in known:
                                rep.error(f"{oid}: bs_guard 指向不存在的判例 {g}"
                                          f"（BS-C* 候选未升库前不得引用，D-027）")
            hist_o = (status in ("superseded", "rejected")
                      or bool(sup_o) or bool(o.get("frozen")))
            if not hist_o and not o.get("satisfies") and org != "tool":
                rep.warn(f"{oid}: 无 satisfies 来源且未声明 origin: tool"
                         "（工具/流程类义务须显式出身，D-023）")
            ck = o.get("checker") or {}
            if not isinstance(ck, dict) or not ck.get("ref"):
                rep.error(f"{oid}: L2 — 缺少 checker.ref")
            elif ck.get("type") not in CHECKER_TYPES:
                rep.error(f"{oid}: checker.type 非法 {ck.get('type')!r}（{sorted(CHECKER_TYPES)}）")
            obligations.append({"id": oid, "did": did, "status": status,
                                "satisfies": o.get("satisfies"), "origin": org,
                                "superseded_by": str(sup_o) if sup_o else None,
                                "frozen": bool(o.get("frozen")),
                                "checker": ck if isinstance(ck, dict) else {}})
        for w in as_list(d.get("waivers")):
            if not w.get("id"):
                rep.error(f"{did}: 豁免缺少 id")
            for f in ("reason", "owner"):
                if not w.get(f):
                    rep.error(f"{did}: 豁免 {w.get('id')} 缺少 {f}（软声明必须签名）")
        arts = [str(a) for a in as_list(d.get("artifacts"))]
        if status in ("accepted", "draft"):
            owned_artifacts.update(arts)
        for a in arts:
            if not (root / a).exists():
                rep.error(f"{did}: artifacts 指向不存在的文件 {a!r}")

    # 义务级 superseded_by 目标校验（D-023：全部决策扫描完后才有完整义务 id 集）
    for o in obligations:
        sb = o["superseded_by"]
        if sb:
            if sb == o["id"]:
                rep.error(f"{o['id']}: superseded_by 自指")
            elif sb not in seen_oids:
                rep.error(f"{o['id']}: superseded_by 指向不存在的义务 {sb!r}")

    # ---------- 触发器库 ----------
    trig_keys = set()
    trig_rules = {}
    for p, t in trg_list:
        key = t.get("trigger")
        if not key:
            rep.error(f"{p.name}: 缺少 trigger 键")
            continue
        if key in trig_keys:
            rep.error(f"{p.name}: 触发器键重复 {key}")
        trig_keys.add(key)
        rules = as_list(t.get("obligations"))
        if not rules:
            rep.error(f"{key}: 触发器没有义务规则")
        for r in rules:
            rk = r.get("key")
            if not rk:
                rep.error(f"{key}: 规则缺少 key")
                continue
            if rk in trig_rules:
                rep.error(f"{key}: 触发器义务键重复 {rk}")
            trig_rules[rk] = {"key": rk, "trigger": key,
                              "waive_allowed": bool(r.get("waive_allowed", True))}

    # ---------- 触发器覆盖检查 ----------
    obl_by_did = {}
    for o in obligations:
        obl_by_did.setdefault(o["did"], []).append(o)
    for did, d in dec_ids.items():
        for key in [str(x) for x in as_list(d.get("introduces"))]:
            if key not in trig_keys:
                rep.error(f"{did}: introduces 引用未知触发器 {key!r}")
                continue
            covered = {o["satisfies"] for o in obl_by_did.get(did, []) if o["satisfies"]}
            waived = {w.get("id") for w in as_list(d.get("waivers"))}
            for r in [r for r in trig_rules.values() if r["trigger"] == key]:
                rk = r["key"]
                if rk in covered:
                    continue
                if rk in waived:
                    if not r["waive_allowed"]:
                        rep.error(f"{did}: 义务 {rk} 不可豁免（waive_allowed: false），必须落实")
                    continue
                rep.error(f"{did}: 触发器 {key} 要求义务 {rk}，未覆盖且未豁免")
    for o in obligations:
        if o["satisfies"] and o["satisfies"] not in trig_rules:
            rep.warn(f"{o['id']}: satisfies 引用未知触发器义务键 {o['satisfies']!r}")

    # ---------- L1：没有无主的代码 ----------
    code_files = set()
    for pat in CODE_PATTERNS:
        for f in root.glob(pat):
            if f.is_file():
                code_files.add(f.relative_to(root).as_posix())
    for f in sorted(code_files):
        if f not in owned_artifacts:
            rep.error(f"L1 — 无主产物: {f}（没有任何 accepted/draft 决策认领）")

    # ---------- L2：没有裸奔的义务 ----------
    ev_by_oid = {}
    for p, e in ev_list:
        if e.get("_parse_error"):
            rep.error(f"{p.name}: YAML 解析失败 — {e['_parse_error']}")
            continue
        oid = str(e.get("obligation", ""))
        if not OID.match(oid):
            rep.error(f"{p.name}: evidence obligation 字段非法 {oid!r}")
            continue
        if oid in ev_by_oid:
            rep.error(f"{p.name}: 义务 {oid} 的证据重复")
        ev_by_oid[oid] = (p, e)

    counts = {"pass": 0, "deferred": 0, "fail": 0, "missing": 0, "historical": 0}
    for o in obligations:
        oid = o["id"]
        if o["status"] in ("superseded", "rejected") or o["superseded_by"] or o["frozen"]:
            # 历史义务：随父决策退场 / 被后继义务取代 / 归档线冻结（D-023），
            # 证据只作历史记录，不再约束当前账目
            counts["historical"] += 1
            continue
        if oid not in ev_by_oid:
            counts["missing"] += 1
            msg = f"L2 — 义务 {oid}（{o['satisfies']}）裸奔：无证据"
            if o["status"] == "draft":
                rep.warn(msg + "（父决策为 draft，暂记 WARN）")
            else:
                rep.error(msg)
            continue
        p, e = ev_by_oid[oid]
        st = e.get("status")
        if st == "pass":
            counts["pass"] += 1
            if not e.get("command") and not e.get("notes"):
                rep.warn(f"{oid}: pass 证据缺少 command/notes 记录")
        elif st == "deferred":
            counts["deferred"] += 1
            if not e.get("reason") or not e.get("owner"):
                rep.error(f"{p.name}: deferred 证据缺少 reason/owner（软声明必须签名）")
            else:
                rep.warn(f"L2 — 义务 {oid}（{o['satisfies']}）证据 deferred — {e.get('reason')}")
        elif st == "fail":
            counts["fail"] += 1
            rep.error(f"L2 — 义务 {oid}（{o['satisfies']}）证据 FAIL")
        else:
            rep.error(f"{p.name}: 证据 status 非法 {st!r}（pass|deferred|fail）")
    for oid, (p, _e) in ev_by_oid.items():
        if not any(o["id"] == oid for o in obligations):
            rep.error(f"{p.name}: 证据指向不存在的义务 {oid}")

    # ---------- L3：没有无人验收的目标 ----------
    obl_ids = {o["id"] for o in obligations}
    for gid, g in goal_ids.items():
        for o in [str(x) for x in as_list(g.get("verification"))]:
            if o not in obl_ids:
                rep.error(f"L3 — {gid}.verification 引用不存在的义务 {o}")
        covered_by = [did for did, d in dec_ids.items()
                      if d.get("status") in ("accepted", "draft")
                      and gid in [str(x) for x in as_list(d.get("goal"))]]
        if not covered_by:
            rep.error(f"L3 — 目标 {gid} 没有任何 accepted/draft 决策服务")

    # ---------- 汇总 ----------
    n_goals = len(goal_ids)
    n_decs = len(dec_ids)
    n_obls = len(obligations)
    return rep, {
        "goals": n_goals, "decisions": n_decs, "obligations": n_obls,
        "trigger_kinds": len(trig_keys), "trigger_rules": len(trig_rules),
        "counts": counts,
    }


def main():
    ap = argparse.ArgumentParser(description="Spec Ledger 账目平衡检查")
    ap.add_argument("--root", default=str(Path(__file__).resolve().parent.parent))
    args = ap.parse_args()
    root = Path(args.root).resolve()
    rep, s = run(root)

    # ---------- 报告 ----------
    counts = s["counts"]
    print("== Spec Ledger 账目 ==")
    print(f"目标 {s['goals']} | 决策 {s['decisions']} | 义务 {s['obligations']} | "
          f"触发器 {s['trigger_kinds']} 类 {s['trigger_rules']} 条规则 | "
          f"证据 pass {counts['pass']} / deferred {counts['deferred']} / "
          f"fail {counts['fail']} / 缺失 {counts['missing']} / 历史 {counts['historical']}")
    for w in rep.warns:
        print(f"WARN  {w}")
    for e in rep.errors:
        print(f"ERROR {e}")
    if rep.errors:
        print(f"RESULT: FAIL（{len(rep.errors)} error, {len(rep.warns)} warn）")
        return 1
    print(f"RESULT: PASS（0 error, {len(rep.warns)} warn）——账目平衡")
    return 0


if __name__ == "__main__":
    sys.exit(main())
