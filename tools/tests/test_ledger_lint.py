"""ledger_lint 黄金用例（O-026，D-015）。

每例构造最小账本夹具，断言 lint 报出的 error/warn 符合预期——
守门人先自证，其余改动才敢动。
"""
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))

import ledger_lint as ll  # noqa: E402


def write(root, rel, text):
    p = root / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8")


def good_ledger(root):
    """最小但合法的账本：1 目标 1 决策 1 义务 1 证据，零义务触发器。"""
    write(root, "ledger/intent/G-001.yaml",
          "id: G-001\nstatus: active\ntitle: t\ndescription: d\nverification: [O-001]\n")
    write(root, "ledger/decisions/D-001.yaml",
          "id: D-001\nstatus: accepted\nquestion: q\nchoice: c\n"
          "alternatives: [a, b]\n"
          "rationale:\n  - text: r\n    anchor: x\n"
          "goal: [G-001]\nobligations:\n"
          "  - id: O-001\n    desc: d\n    bs_guard: []\n"
          "    bs_note: 夹具义务：不针对已编目判例（D-027）\n"
          "    checker:\n      type: test\n      ref: r\n"
          "waivers: []\nartifacts: []\nsuperseded_by: null\n")
    write(root, "ledger/evidence/O-001.yaml",
          "obligation: O-001\nstatus: pass\ncommand: c\n")


class LedgerLintTests(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def rep(self):
        return ll.run(self.root)

    # ---------- 好账 ----------
    def test_good_ledger_passes(self):
        good_ledger(self.root)
        rep, s = self.rep()
        self.assertEqual(rep.errors, [])
        self.assertEqual(s["goals"], 1)
        self.assertEqual(s["decisions"], 1)
        self.assertEqual(s["obligations"], 1)

    # ---------- L1：没有无主的代码 ----------
    def test_l1_orphan_code_file(self):
        good_ledger(self.root)
        write(self.root, "pilot-rs/src/orphan.rs", "fn main() {}\n")
        rep, _ = self.rep()
        self.assertTrue(any("L1" in e and "orphan.rs" in e for e in rep.errors))

    def test_l1_owned_code_file_passes(self):
        good_ledger(self.root)
        write(self.root, "pilot-rs/src/owned.rs", "fn main() {}\n")
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("artifacts: []", "artifacts: [pilot-rs/src/owned.rs]")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertFalse(any("L1" in e for e in rep.errors))

    # ---------- L2：没有裸奔的义务 ----------
    def test_l2_missing_evidence(self):
        good_ledger(self.root)
        (self.root / "ledger/evidence/O-001.yaml").unlink()
        rep, _ = self.rep()
        self.assertTrue(any("L2" in e and "O-001" in e and "裸奔" in e for e in rep.errors))

    def test_l2_deferred_without_signature(self):
        good_ledger(self.root)
        write(self.root, "ledger/evidence/O-001.yaml",
              "obligation: O-001\nstatus: deferred\n")
        rep, _ = self.rep()
        self.assertTrue(any("deferred 证据缺少 reason/owner" in e for e in rep.errors))

    def test_l2_fail_evidence_is_error(self):
        good_ledger(self.root)
        write(self.root, "ledger/evidence/O-001.yaml",
              "obligation: O-001\nstatus: fail\n")
        rep, _ = self.rep()
        self.assertTrue(any("证据 FAIL" in e for e in rep.errors))

    def test_l2_superseded_decision_obligation_is_historical(self):
        good_ledger(self.root)
        write(self.root, "ledger/evidence/O-001.yaml",
              "obligation: O-001\nstatus: pass\ncommand: c\n")
        # 加一个 superseded 决策带义务，无证据也应作历史不报错
        write(self.root, "ledger/decisions/D-002.yaml",
              "id: D-002\nstatus: superseded\nquestion: q\nchoice: c\n"
              "alternatives: [a]\nrationale:\n  - text: r\n    anchor: x\n"
              "goal: [G-001]\nobligations:\n"
              "  - id: O-002\n    desc: d\n    bs_guard: []\n    bs_note: 夹具：历史义务\n"
              "    checker:\n      type: test\n      ref: r\n"
              "waivers: []\nartifacts: []\nsuperseded_by: D-001\n")
        rep, s = self.rep()
        self.assertFalse(any("O-002" in e for e in rep.errors))
        self.assertEqual(s["counts"]["historical"], 1)

    # ---------- L3：没有无人验收的目标 ----------
    def test_l3_goal_without_decision(self):
        write(self.root, "ledger/intent/G-001.yaml",
              "id: G-001\nstatus: active\ntitle: t\ndescription: d\nverification: [O-001]\n")
        write(self.root, "ledger/evidence/O-001.yaml",
              "obligation: O-001\nstatus: pass\ncommand: c\n")
        rep, _ = self.rep()
        self.assertTrue(any("L3" in e and "G-001" in e for e in rep.errors))

    def test_l3_verification_missing_obligation(self):
        write(self.root, "ledger/intent/G-001.yaml",
              "id: G-001\nstatus: active\ntitle: t\ndescription: d\nverification: [O-009]\n")
        good_dec = ("id: D-001\nstatus: accepted\nquestion: q\nchoice: c\n"
                    "alternatives: [a]\nrationale:\n  - text: r\n    anchor: x\n"
                    "goal: [G-001]\nobligations: []\n"
                    "waivers: []\nartifacts: []\nsuperseded_by: null\n")
        write(self.root, "ledger/decisions/D-001.yaml", good_dec)
        rep, _ = self.rep()
        self.assertTrue(any("L3" in e and "O-009" in e for e in rep.errors))

    # ---------- schema 与引用完整性 ----------
    def test_superseded_without_successor(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("status: accepted", "status: superseded")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("缺少 superseded_by" in e for e in rep.errors))

    def test_duplicate_decision_id(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        write(self.root, "ledger/decisions/D-002.yaml", d.replace("D-001", "D-002"))
        # D-002 引用同一义务 O-001 → 决策 id 各自合法，但义务 id 跨决策重复
        rep, _ = self.rep()
        self.assertTrue(any("义务 id 跨决策重复" in e for e in rep.errors))

    def test_illegal_checker_type(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("type: test", "type: oracle")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("checker.type 非法" in e for e in rep.errors))

    def test_artifact_path_missing(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("artifacts: []", "artifacts: [no/such/file.rs]")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("指向不存在的文件" in e for e in rep.errors))

    # ---------- 触发器覆盖 ----------
    TRIGGER = ("trigger: shared_mutable_state\n"
               "obligations:\n"
               "  - key: race_freedom\n    desc: d\n    waive_allowed: false\n"
               "  - key: deadlock_freedom\n    desc: d\n    waive_allowed: true\n")

    def good_decision_with_trigger(self, root):
        write(root, "ledger/triggers/concurrency.yaml", self.TRIGGER)
        write(root, "ledger/decisions/D-001.yaml",
              "id: D-001\nstatus: accepted\nquestion: q\nchoice: c\n"
              "alternatives: [a]\nrationale:\n  - text: r\n    anchor: x\n"
              "goal: [G-001]\nintroduces: [shared_mutable_state]\n"
              "obligations:\n"
              "  - id: O-001\n    desc: d\n    satisfies: race_freedom\n"
              "    checker:\n      type: test\n      ref: r\n"
              "waivers: []\nartifacts: []\nsuperseded_by: null\n")

    def test_trigger_uncovered_rule_is_error(self):
        good_ledger(self.root)
        self.good_decision_with_trigger(self.root)  # deadlock_freedom 未覆盖未豁免
        rep, _ = self.rep()
        self.assertTrue(any("deadlock_freedom" in e and "未覆盖" in e for e in rep.errors))

    def test_trigger_nonwaivable_waiver_is_error(self):
        good_ledger(self.root)
        self.good_decision_with_trigger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        # 唯一义务覆盖可豁免的 deadlock_freedom；race_freedom（不可豁免）仅靠豁免 → 违规
        d = d.replace("satisfies: race_freedom", "satisfies: deadlock_freedom")
        d = d.replace("waivers: []",
                      "waivers:\n  - id: race_freedom\n    reason: r\n    owner: o\n")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("不可豁免" in e for e in rep.errors))

    def test_trigger_waivable_waiver_passes(self):
        good_ledger(self.root)
        self.good_decision_with_trigger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("waivers: []",
                      "waivers:\n  - id: deadlock_freedom\n    reason: r\n    owner: o\n")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertFalse(any("未覆盖" in e for e in rep.errors))

    def test_introduces_unknown_trigger(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("goal: [G-001]\n", "goal: [G-001]\nintroduces: [no_such]\n")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("未知触发器" in e for e in rep.errors))

    # ---------- D-022/D-023：anchor 存在性、origin、义务生命周期、G status ----------
    def test_goal_missing_status_is_error(self):
        good_ledger(self.root)
        g = (self.root / "ledger/intent/G-001.yaml").read_text(encoding="utf-8")
        write(self.root, "ledger/intent/G-001.yaml",
              g.replace("status: active\n", ""))
        rep, _ = self.rep()
        self.assertTrue(any("G-001" in e and "status" in e for e in rep.errors))

    def test_origin_tool_suppresses_satisfies_warn(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("  - id: O-001\n", "  - id: O-001\n    origin: tool\n")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertFalse(any("O-001" in w and "satisfies" in w for w in rep.warns))

    def test_obligation_without_satisfies_and_origin_warns(self):
        good_ledger(self.root)
        rep, _ = self.rep()
        self.assertTrue(any("O-001" in w and "origin" in w for w in rep.warns))

    def test_obligation_superseded_by_is_historical(self):
        good_ledger(self.root)
        # O-002 声明被 O-001 取代：无证据也作历史，不计缺失
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("waivers: []",
                      "  - id: O-002\n    desc: d2\n    bs_guard: []\n"
                      "    bs_note: 夹具：被取代的历史义务\n    superseded_by: O-001\n"
                      "    checker:\n      type: lint\n      ref: r\n"
                      "waivers: []")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, s = self.rep()
        self.assertFalse(any("O-002" in e for e in rep.errors))
        self.assertEqual(s["counts"]["historical"], 1)

    def test_obligation_superseded_by_dangling_is_error(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("  - id: O-001\n", "  - id: O-001\n    superseded_by: O-009\n")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("O-001" in e and "superseded_by" in e for e in rep.errors))

    def test_obligation_frozen_is_historical(self):
        good_ledger(self.root)
        (self.root / "ledger/evidence/O-001.yaml").unlink()  # 冻结义务无证据也不报缺失
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("  - id: O-001\n", "  - id: O-001\n    frozen: true\n")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, s = self.rep()
        self.assertEqual(rep.errors, [])
        self.assertEqual(s["counts"]["historical"], 1)

    def test_anchor_missing_path_warns(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("anchor: x", "anchor: no/such/file.md")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("anchor 路径不存在" in w for w in rep.warns))

    def test_anchor_plain_word_no_path_check(self):
        good_ledger(self.root)
        rep, _ = self.rep()
        self.assertFalse(any("anchor 路径不存在" in w for w in rep.warns))

    def test_assumptions_missing_invalidation_is_error(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("waivers: []",
                      "assumptions:\n  - text: a\nwaivers: []")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertTrue(any("assumptions" in e and "invalidation" in e for e in rep.errors))

    def test_assumptions_wellformed_passes(self):
        good_ledger(self.root)
        d = (self.root / "ledger/decisions/D-001.yaml").read_text(encoding="utf-8")
        d = d.replace("waivers: []",
                      "assumptions:\n"
                      "  - text: a\n    invalidation: 什么证据会推翻我\nwaivers: []")
        write(self.root, "ledger/decisions/D-001.yaml", d)
        rep, _ = self.rep()
        self.assertFalse(any("assumptions" in e for e in rep.errors))

    def test_open_questions_pass(self):
        good_ledger(self.root)
        g = (self.root / "ledger/intent/G-001.yaml").read_text(encoding="utf-8")
        write(self.root, "ledger/intent/G-001.yaml",
              g + "open_questions:\n  - 该目标在什么场景下算成功？\n")
        rep, _ = self.rep()
        self.assertEqual(rep.errors, [])

    def test_open_questions_empty_entry_is_error(self):
        good_ledger(self.root)
        g = (self.root / "ledger/intent/G-001.yaml").read_text(encoding="utf-8")
        write(self.root, "ledger/intent/G-001.yaml",
              g + "open_questions:\n  - \"\"\n")
        rep, _ = self.rep()
        self.assertTrue(any("open_questions" in e for e in rep.errors))

    # ---------- bs_guard：盲区库由清单升为门（D-027） ----------
    def _decision_with_guard(self, guard_lines):
        return ("id: D-001\nstatus: accepted\nquestion: q\nchoice: c\n"
                "alternatives: [a, b]\n"
                "rationale:\n  - text: r\n    anchor: x\n"
                "goal: [G-001]\nobligations:\n"
                "  - id: O-001\n    desc: d\n"
                + guard_lines +
                "    checker:\n      type: test\n      ref: r\n"
                "waivers: []\nartifacts: []\nsuperseded_by: null\n")

    def test_bs_guard_missing_is_error(self):
        good_ledger(self.root)
        write(self.root, "ledger/decisions/D-001.yaml", self._decision_with_guard(""))
        rep, _ = self.rep()
        self.assertTrue(any("缺少 bs_guard" in e for e in rep.errors))

    def test_bs_guard_empty_without_note_is_error(self):
        good_ledger(self.root)
        write(self.root, "ledger/decisions/D-001.yaml",
              self._decision_with_guard("    bs_guard: []\n"))
        rep, _ = self.rep()
        self.assertTrue(any("必须给 bs_note" in e for e in rep.errors))

    def test_bs_guard_empty_with_note_passes(self):
        good_ledger(self.root)
        write(self.root, "ledger/decisions/D-001.yaml",
              self._decision_with_guard("    bs_guard: []\n    bs_note: 设计不变量，非已编目判例\n"))
        rep, _ = self.rep()
        self.assertEqual(rep.errors, [])

    def test_bs_guard_referencing_existing_blind_spot_passes(self):
        good_ledger(self.root)
        write(self.root, "thinking/blind_spots.md",
              "# 盲区库\n\n### BS-012 自审不可见偏差\n- **形式**：x\n")
        write(self.root, "ledger/decisions/D-001.yaml",
              self._decision_with_guard("    bs_guard: [BS-012]\n"))
        rep, _ = self.rep()
        self.assertEqual(rep.errors, [])

    def test_bs_guard_referencing_unknown_blind_spot_is_error(self):
        good_ledger(self.root)
        write(self.root, "thinking/blind_spots.md",
              "# 盲区库\n\n### BS-012 自审不可见偏差\n- **形式**：x\n")
        write(self.root, "ledger/decisions/D-001.yaml",
              self._decision_with_guard("    bs_guard: [BS-999]\n"))
        rep, _ = self.rep()
        self.assertTrue(any("不存在的判例" in e for e in rep.errors))

    def test_bs_guard_with_empty_catalogue_is_error(self):
        """BS-C1「空作用域通过」的自应用：库在，却解析出零条判例，不得静默通过。"""
        good_ledger(self.root)
        write(self.root, "thinking/blind_spots.md", "# 盲区库\n\n（尚无判例）\n")
        write(self.root, "ledger/decisions/D-001.yaml",
              self._decision_with_guard("    bs_guard: [BS-012]\n"))
        rep, _ = self.rep()
        self.assertTrue(any("作用域为空" in e for e in rep.errors))

    def test_bs_guard_of_wrong_type_is_error(self):
        good_ledger(self.root)
        write(self.root, "ledger/decisions/D-001.yaml",
              self._decision_with_guard("    bs_guard: BS-012\n"))
        rep, _ = self.rep()
        self.assertTrue(any("必须是列表" in e for e in rep.errors))


if __name__ == "__main__":
    unittest.main()
