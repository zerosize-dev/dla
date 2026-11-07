import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import grade  # noqa: E402

SAMPLE = HERE / "testdata" / "py-sample"


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------


def leak_toml(lid, site, holder="static/global", cause="L-COLLECT", confirmable=True,
              lines=None, alts=None, share=None):
    s = f'\n[[leak]]\nid = "{lid}"\nsite = "{site}"\nholder = "{holder}"\n'
    s += f'region = "heap"\ncause = "{cause}"\n'
    if confirmable is not None:
        s += f'confirmable = {"true" if confirmable else "false"}\n'
    if lines:
        s += f"lines = {list(lines)}\n"
    if alts:
        s += "alt_sites = " + json.dumps(alts) + "\n"
    if share is not None:
        s += f"share = {share}\n"
    return s


def fixture_toml(fid, category, body="", pair=None):
    s = f'schema_version = 1\nid = "{fid}"\nruntime = "jvm"\ncategory = "{category}"\n'
    if pair:
        s += f'pair = "{pair}"\n'
    s += 'dynamic_confirmation = true\n\n[task]\nprompt = "p"\n'
    return s + body


def added(fid, site, holder=None, arch=None, parent=None):
    f = {"id": fid, "site": site}
    if holder is not None:
        f["holder"] = holder
    if arch is not None:
        f["archetype"] = arch
    if parent is not None:
        f["parent"] = parent
    return json.dumps({"type": "finding_added", "finding": f})


def trans(fid, to):
    return json.dumps({"type": "transition", "finding": fid, "to": to})


def ledger(*lines):
    return "\n".join(lines) + "\n"


def jf(site, holder=None, verdict="leak-confirmed", cause=None, disc=None):
    d = {"site": site, "verdict": verdict}
    if holder is not None:
        d["holder"] = holder
    if cause is not None:
        d["cause"] = cause
    if disc is not None:
        d["discriminator"] = disc
    return d


class Env:
    """A temporary results + fixtures tree."""

    def __init__(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.fixtures = self.root / "fixtures"
        self.results = self.root / "results"
        self.fixtures.mkdir()
        self.results.mkdir()

    def close(self):
        self._tmp.cleanup()

    def fixture(self, fid, category, body="", pair=None):
        d = self.fixtures / fid
        d.mkdir(exist_ok=True)
        (d / "expected.toml").write_text(fixture_toml(fid, category, body, pair))

    def run(self, fid, n=1, abl="no-skill", arch="on", ledger_text=None, findings=None,
            raw_findings=None, coverage=None, run_json=None, write_run_json=True):
        d = self.results / "runs" / abl / f"archetypes-{arch}" / fid / str(n)
        d.mkdir(parents=True, exist_ok=True)
        if write_run_json:
            rj = {"exit_code": 0, "timed_out": False, "wall_seconds": 1.0, "cost_usd": 0.5,
                  "usage": {"input_tokens": 10, "output_tokens": 5,
                            "cache_read_input_tokens": 100, "cache_creation_input_tokens": 7}}
            if run_json is not None:
                rj.update(run_json)
            (d / "run.json").write_text(json.dumps(rj))
        if ledger_text is not None:
            (d / "ledger.jsonl").write_text(ledger_text)
        if findings is not None:
            (d / "findings.json").write_text(json.dumps({"findings": findings}))
        if raw_findings is not None:
            (d / "findings.json").write_text(raw_findings)
        if coverage is not None:
            (d / "coverage.json").write_text(coverage if isinstance(coverage, str) else json.dumps(coverage))
        return d

    def grade(self):
        return grade.grade(self.results, self.fixtures)

    def group(self, abl="no-skill", arch="on"):
        for g in self.grade()["groups"]:
            if g["ablation"] == abl and g["archetypes"] == arch:
                return g
        raise AssertionError("no such group")


class EnvCase(unittest.TestCase):
    def setUp(self):
        self.env = Env()
        self.addCleanup(self.env.close)


# ---------------------------------------------------------------------------
# pure functions
# ---------------------------------------------------------------------------


class Round6Test(unittest.TestCase):
    def test_tie_one_128th_rounds_half_to_even(self):
        # 1/128 = 0.0078125 exactly; half-to-even gives ...812, half-up would give ...813
        self.assertEqual(grade.round6(1 / 128), 0.007812)

    def test_tie_three_128th_rounds_up_to_even(self):
        self.assertEqual(grade.round6(3 / 128), 0.023438)

    def test_plain(self):
        self.assertEqual(grade.round6(2 / 3), 0.666667)
        self.assertEqual(grade.round6(1.0), 1.0)

    def test_ratio_entry_null_on_zero_den(self):
        self.assertIsNone(grade.ratio_entry(0, 0)["value"])
        self.assertEqual(grade.ratio_entry(1, 128)["value"], 0.007812)


class SiteParsingTest(unittest.TestCase):
    def test_basic(self):
        self.assertEqual(grade.parse_site("src/a.py:12"), ("src/a.py", 12))

    def test_column_is_ignored(self):
        self.assertEqual(grade.parse_site("src/a.py:12:7"), ("src/a.py", 12))

    def test_backslashes_and_leading_dot_slash(self):
        self.assertEqual(grade.parse_site(".\\src\\a.py:3"), ("src/a.py", 3))
        self.assertEqual(grade.parse_site("././src/a.py:3"), ("src/a.py", 3))

    def test_no_location(self):
        for s in ("src/a.py", "src/a.py:", "src/a.py:x", None, 5, ":"):
            self.assertIsNone(grade.parse_site(s), s)

    def test_dot_slash_only_removed_at_start(self):
        self.assertEqual(grade.parse_site("a/./b.py:1"), ("a/./b.py", 1))

    def test_windows_drive_letter(self):
        self.assertEqual(grade.parse_site("C:\\w\\src\\a.py:9"), ("C:/w/src/a.py", 9))


def match(site, leak_site, lines=None, alts=None):
    leak = {"site": leak_site}
    if lines:
        leak["lines"] = lines
    if alts:
        leak["alt_sites"] = alts
    f = grade.new_finding("confirmed", site, None, None, False, False)
    return grade.finding_matches(f, grade.leak_targets(leak))


class MatchTest(unittest.TestCase):
    def test_exact_and_default_range(self):
        self.assertTrue(match("a/b.py:10", "a/b.py:10"))
        self.assertFalse(match("a/b.py:11", "a/b.py:10"))
        self.assertFalse(match("a/b.py:9", "a/b.py:10"))

    def test_lines_range_inclusive(self):
        for ln, ok in ((54, False), (55, True), (58, True), (61, True), (62, False)):
            self.assertEqual(match(f"a.py:{ln}", "a.py:57", lines=[55, 61]), ok, ln)

    def test_suffix_path(self):
        self.assertTrue(match("/tmp/x/proj/src/a.py:5", "src/a.py:5"))
        self.assertTrue(match("proj/src/a.py:5", "src/a.py:5"))

    def test_suffix_requires_slash_boundary(self):
        self.assertFalse(match("mysrc/a.py:5", "src/a.py:5"))
        self.assertFalse(match("src/a.pyc:5", "src/a.py:5"))

    def test_finding_path_shorter_than_leak_path_does_not_match(self):
        self.assertFalse(match("a.py:5", "src/a.py:5"))

    def test_backslash_and_dot_slash(self):
        self.assertTrue(match(".\\src\\a.py:5", "src/a.py:5"))
        self.assertTrue(match("./src/a.py:5:2", "src/a.py:5"))

    def test_alt_site_single_line(self):
        self.assertTrue(match("b.py:7", "a.py:1", alts=["b.py:7"]))
        self.assertFalse(match("b.py:8", "a.py:1", alts=["b.py:7"]))

    def test_alt_site_range(self):
        for ln, ok in ((21, False), (22, True), (23, True), (24, True), (25, False)):
            self.assertEqual(match(f"x/hub.go:{ln}", "a.go:1", alts=["hub.go:22-24"]), ok, ln)

    def test_alt_site_does_not_borrow_main_path(self):
        self.assertFalse(match("a.py:7", "a.py:1", alts=["b.py:7"]))

    def test_no_location_never_matches(self):
        self.assertFalse(match("a.py", "a.py:1"))
        self.assertFalse(match(None, "a.py:1"))

    def test_malformed_alt_ignored(self):
        self.assertTrue(match("a.py:1", "a.py:1", alts=["nonsense", 5]))


# ---------------------------------------------------------------------------
# normalisation
# ---------------------------------------------------------------------------


class LedgerParseTest(EnvCase):
    def parse(self, text):
        p = self.env.root / "l.jsonl"
        p.write_text(text)
        return grade.parse_ledger(p)

    def test_basic_and_state_mapping(self):
        cases = {"confirmed": "confirmed", "strong": "strong", "refuted": "refuted",
                 "suspected_static_only": "suspected", "unresolved": "unresolved",
                 "whatever": "open", "": "open"}
        for to, want in cases.items():
            saw, fs = self.parse(ledger(added("f", "a:1"), trans("f", to)))
            self.assertTrue(saw)
            self.assertEqual(fs[0]["state"], want, to)

    def test_fields(self):
        _, fs = self.parse(ledger(added("f", "a.py:1", "static/global", "L-X")))
        f = fs[0]
        self.assertEqual((f["state"], f["site"], f["holder"], f["label"], f["hypothesis"]),
                         ("open", "a.py:1", "static/global", "L-X", False))

    def test_missing_or_nonstring_members_are_null(self):
        line = json.dumps({"type": "finding_added",
                           "finding": {"id": "f", "site": 5, "holder": None, "archetype": 3}})
        _, fs = self.parse(line + "\n")
        self.assertEqual((fs[0]["site"], fs[0]["holder"], fs[0]["label"]), (None, None, None))

    def test_hypothesis_is_nonempty_string_parent(self):
        saw, fs = self.parse(ledger(
            added("a", "a:1", parent="x"), added("b", "a:1", parent=""),
            json.dumps({"type": "finding_added", "finding": {"id": "c", "parent": 4}}),
            json.dumps({"type": "finding_added", "finding": {"id": "d", "parent": None}})))
        self.assertEqual([f["hypothesis"] for f in fs], [True, False, False, False])

    def test_discriminator_true_iff_refuted(self):
        _, fs = self.parse(ledger(added("a", "a:1"), trans("a", "refuted"),
                                  added("b", "a:1"), trans("b", "confirmed"),
                                  added("c", "a:1")))
        self.assertEqual([f["discriminator"] for f in fs], [True, False, False])

    def test_malformed_lines_skipped(self):
        text = "\n".join(["garbage", "", "[1, 2]", "42", '"str"', "null", '{"no": "type"}',
                           '{"type": 5}', '{"type": "finding_added"',
                           added("f", "a:1"), trans("f", "confirmed")]) + "\n"
        saw, fs = self.parse(text)
        self.assertTrue(saw)
        self.assertEqual([(f["site"], f["state"]) for f in fs], [("a:1", "confirmed")])

    def test_invalid_utf8_line_skipped(self):
        p = self.env.root / "l.jsonl"
        p.write_bytes(b"\xff\xfe\n" + added("f", "a:1").encode() + b"\n")
        saw, fs = grade.parse_ledger(p)
        self.assertEqual(len(fs), 1)

    def test_crlf_lines(self):
        saw, fs = self.parse(ledger(added("f", "a:1"), trans("f", "strong")).replace("\n", "\r\n"))
        self.assertEqual(fs[0]["state"], "strong")

    def test_other_record_types_ignored(self):
        saw, fs = self.parse(ledger(json.dumps({"type": "evidence", "id": "e1"}),
                                    json.dumps({"type": "run-config"})))
        self.assertFalse(saw)
        self.assertEqual(fs, [])

    def test_transition_for_unknown_finding_ignored(self):
        saw, fs = self.parse(ledger(trans("ghost", "confirmed"), added("f", "a:1")))
        self.assertEqual(fs[0]["state"], "open")

    def test_transition_before_add_does_not_stick(self):
        _, fs = self.parse(ledger(trans("f", "confirmed"), added("f", "a:1")))
        self.assertEqual(fs[0]["state"], "open")

    def test_repeated_id_replaces_and_resets_state(self):
        _, fs = self.parse(ledger(added("f", "a:1"), trans("f", "confirmed"),
                                  added("g", "b:2"), added("f", "c:3")))
        self.assertEqual([(f["site"], f["state"]) for f in fs], [("b:2", "open"), ("c:3", "open")])

    def test_last_transition_wins(self):
        _, fs = self.parse(ledger(added("f", "a:1"), trans("f", "confirmed"), trans("f", "refuted")))
        self.assertEqual(fs[0]["state"], "refuted")

    def test_transition_non_string_to_is_open(self):
        _, fs = self.parse(ledger(added("f", "a:1"), trans("f", "confirmed"),
                                  json.dumps({"type": "transition", "finding": "f", "to": 5})))
        self.assertEqual(fs[0]["state"], "open")


class FindingsJsonParseTest(EnvCase):
    def parse(self, doc):
        p = self.env.root / "f.json"
        p.write_text(doc if isinstance(doc, str) else json.dumps(doc))
        return grade.parse_findings_json(p)

    def test_verdicts(self):
        fs = self.parse({"findings": [
            {"verdict": "leak-confirmed"}, {"verdict": "leak-suspected"},
            {"verdict": "not-a-leak"}, {"verdict": "maybe"}, {}, {"verdict": 3}]})
        self.assertEqual([f["state"] for f in fs],
                         ["confirmed", "suspected", "refuted", "open", "open", "open"])
        self.assertFalse(any(f["hypothesis"] for f in fs))

    def test_label_regex(self):
        good = ["L-WORKER", "N-HWM", "UNCLASSIFIED", "L-A"]
        bad = ["l-worker", "L-", "L-A1", "L-WORKER ", "free text", "UNCLASSIFIED2", "", "X-ABC", 5, None]
        fs = self.parse({"findings": [{"cause": c} for c in good + bad]})
        self.assertEqual([f["label"] for f in fs], good + [None] * len(bad))

    def test_label_rejects_trailing_newline(self):
        fs = self.parse({"findings": [{"cause": "L-A\n"}]})
        self.assertIsNone(fs[0]["label"])

    def test_discriminator_needs_not_a_leak_and_nonblank(self):
        fs = self.parse({"findings": [
            {"verdict": "not-a-leak", "discriminator": "plateau"},
            {"verdict": "not-a-leak", "discriminator": " \t\n "},
            {"verdict": "not-a-leak", "discriminator": ""},
            {"verdict": "not-a-leak", "discriminator": 5},
            {"verdict": "not-a-leak"},
            {"verdict": "leak-confirmed", "discriminator": "plateau"}]})
        self.assertEqual([f["discriminator"] for f in fs], [True, False, False, False, False, False])

    def test_non_object_elements_skipped_and_non_string_members_null(self):
        fs = self.parse({"findings": ["x", 3, None, [1], {"site": 4, "holder": ["h"]}]})
        self.assertEqual(len(fs), 1)
        self.assertEqual((fs[0]["site"], fs[0]["holder"]), (None, None))

    def test_unusable_files(self):
        for doc in ("{broken", "[]", "42", '{"findings": {}}', '{"findings": null}', '{"other": []}'):
            self.assertIsNone(self.parse(doc), doc)


class SourceSelectionTest(EnvCase):
    def test_ledger_with_finding_added_wins(self):
        d = self.env.run("x", ledger_text=ledger(added("f", "a:1")),
                         findings=[jf("b:1"), jf("c:1")])
        src, fs = grade.load_findings(d)
        self.assertEqual((src, len(fs)), ("ledger", 1))

    def test_ledger_without_finding_added_falls_back(self):
        d = self.env.run("x", ledger_text=ledger(json.dumps({"type": "note"})), findings=[jf("b:1")])
        self.assertEqual(grade.load_findings(d)[0], "findings")

    def test_ledger_with_only_invalid_finding_object_still_counts(self):
        d = self.env.run("x", ledger_text=ledger(json.dumps({"type": "finding_added", "finding": 5})),
                         findings=[jf("b:1")])
        src, fs = grade.load_findings(d)
        self.assertEqual((src, fs), ("ledger", []))

    def test_none_when_nothing_usable(self):
        d = self.env.run("x")
        self.assertEqual(grade.load_findings(d), ("none", []))
        d = self.env.run("x", n=2, raw_findings="{oops")
        self.assertEqual(grade.load_findings(d), ("none", []))

    def test_empty_findings_array_is_findings_source(self):
        d = self.env.run("x", findings=[])
        self.assertEqual(grade.load_findings(d), ("findings", []))


# ---------------------------------------------------------------------------
# metrics
# ---------------------------------------------------------------------------


class ChosenFindingTest(EnvCase):
    def setUp(self):
        super().setUp()
        self.env.fixture("fx", "archetype-leak", leak_toml("L1", "a.py:5", "static/global", "L-A"))

    def metrics(self, **kw):
        self.env.run("fx", **kw)
        return self.env.group()["metrics"]

    def test_strong_preferred_over_earlier_confirmed(self):
        m = self.metrics(ledger_text=ledger(
            added("c", "a.py:5", "container-field", "L-B"), trans("c", "confirmed"),
            added("s", "a.py:5", "static/global", "L-A"), trans("s", "strong")))
        self.assertEqual((m["grounded_attribution"]["num"], m["grounded_attribution"]["den"]), (1, 1))
        self.assertEqual(m["label_accuracy"]["num"], 1)

    def test_first_of_several_strong(self):
        m = self.metrics(ledger_text=ledger(
            added("s1", "a.py:5", "container-field", "L-B"), trans("s1", "strong"),
            added("s2", "a.py:5", "static/global", "L-A"), trans("s2", "strong")))
        self.assertEqual(m["grounded_attribution"]["num"], 0)
        self.assertEqual(m["label_accuracy"]["num"], 0)
        self.assertEqual(m["label_accuracy"]["den"], 1)

    def test_first_confirmed_when_no_strong(self):
        m = self.metrics(ledger_text=ledger(
            added("c1", "a.py:5", "static/global", "L-A"), trans("c1", "confirmed"),
            added("c2", "a.py:5", "container-field", "L-B"), trans("c2", "confirmed")))
        self.assertEqual(m["grounded_attribution"]["num"], 1)

    def test_hypothesis_never_found_or_flagged(self):
        m = self.metrics(ledger_text=ledger(
            added("h", "a.py:5", "static/global", "L-A", parent="p"), trans("h", "confirmed")))
        self.assertEqual(m["leak_recall"]["num"], 0)
        self.assertEqual(m["flagged_recall"]["num"], 0)

    def test_flagged_states(self):
        for to, flagged in (("confirmed", 1), ("strong", 1), ("suspected_static_only", 1),
                            ("unresolved", 1), ("bogus", 1), ("refuted", 0)):
            env = Env()
            self.addCleanup(env.close)
            env.fixture("fx", "archetype-leak", leak_toml("L1", "a.py:5"))
            env.run("fx", ledger_text=ledger(added("f", "a.py:5"), trans("f", to)))
            m = env.group()["metrics"]
            self.assertEqual(m["flagged_recall"]["num"], flagged, to)
        # still open (no transition)
        env = Env()
        self.addCleanup(env.close)
        env.fixture("fx", "archetype-leak", leak_toml("L1", "a.py:5"))
        env.run("fx", ledger_text=ledger(added("f", "a.py:5")))
        self.assertEqual(env.group()["metrics"]["flagged_recall"]["num"], 1)

    def test_found_requires_confirmed_or_strong(self):
        m = self.metrics(ledger_text=ledger(added("f", "a.py:5"), trans("f", "suspected_static_only")))
        self.assertEqual(m["leak_recall"]["num"], 0)
        self.assertEqual(m["flagged_recall"]["num"], 1)

    def test_null_chosen_holder_is_not_grounded(self):
        m = self.metrics(ledger_text=ledger(added("f", "a.py:5"), trans("f", "confirmed")))
        self.assertEqual((m["grounded_attribution"]["num"], m["grounded_attribution"]["den"]), (0, 1))

    def test_null_label_excluded_from_label_metrics(self):
        m = self.metrics(ledger_text=ledger(added("f", "a.py:5", "static/global"), trans("f", "confirmed")))
        self.assertEqual(m["label_accuracy"]["den"], 0)
        self.assertEqual(m["forced_fit_rate"]["den"], 0)


class RecallMetricsTest(EnvCase):
    def test_leak_set_excludes_unconfirmable_and_other_categories(self):
        body = leak_toml("L1", "a.py:1") + leak_toml("L2", "a.py:2", confirmable=False)
        self.env.fixture("t", "archetype-leak", body)
        self.env.fixture("imp", "impossible", leak_toml("L1", "z.py:1", confirmable=False))
        self.env.fixture("d", "decoy-control", '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n')
        self.env.run("t", findings=[jf("a.py:1", "static/global"), jf("a.py:2", "static/global")])
        self.env.run("imp", findings=[jf("z.py:1", "static/global")])
        m = self.env.group()["metrics"]
        self.assertEqual((m["leak_recall"]["num"], m["leak_recall"]["den"]), (1, 1))
        self.assertEqual((m["flagged_recall"]["num"], m["flagged_recall"]["den"]), (1, 1))

    def test_archetype_free_recall(self):
        body = (leak_toml("L1", "a.py:1", cause="UNCLASSIFIED")
                + leak_toml("L2", "a.py:2", cause="L-COLLECT"))
        self.env.fixture("m", "multi-leak", body)
        self.env.run("m", findings=[jf("a.py:1", "static/global")])
        m = self.env.group()["metrics"]
        self.assertEqual((m["archetype_free_recall"]["num"], m["archetype_free_recall"]["den"]), (1, 1))
        self.assertEqual((m["leak_recall"]["num"], m["leak_recall"]["den"]), (1, 2))

    def test_second_leak_arithmetic(self):
        self.env.fixture("m3", "multi-leak",
                         leak_toml("L1", "a.py:1") + leak_toml("L2", "a.py:2") + leak_toml("L3", "a.py:3"))
        # 3 leaks, 2 found -> den 2, num 1
        self.env.run("m3", n=1, findings=[jf("a.py:1", "h"), jf("a.py:3", "h")])
        # 3 leaks, 1 found -> den 2, num 0
        self.env.run("m3", n=2, findings=[jf("a.py:2", "h")])
        # 3 leaks, 3 found -> den 2, num 2
        self.env.run("m3", n=3, findings=[jf("a.py:1", "h"), jf("a.py:2", "h"), jf("a.py:3", "h")])
        # nothing found -> contributes nothing
        self.env.run("m3", n=4, findings=[])
        s = self.env.group()["metrics"]["second_leak_recall"]
        self.assertEqual((s["num"], s["den"]), (3, 6))
        self.assertEqual([(p["num"], p["den"]) for p in s["per_run"]], [(1, 2), (0, 2), (2, 2), (0, 0)])
        self.assertIsNone(s["per_run"][3]["value"])

    def test_second_leak_only_for_multi_leak(self):
        self.env.fixture("t", "archetype-leak", leak_toml("L1", "a.py:1") + leak_toml("L2", "a.py:2"))
        self.env.run("t", findings=[jf("a.py:1", "h")])
        self.assertEqual(self.env.group()["metrics"]["second_leak_recall"]["den"], 0)

    def test_second_leak_single_confirmable_leak_adds_zero(self):
        self.env.fixture("m1", "multi-leak", leak_toml("L1", "a.py:1") + leak_toml("L2", "a.py:2", confirmable=False))
        self.env.run("m1", findings=[jf("a.py:1", "h")])
        s = self.env.group()["metrics"]["second_leak_recall"]
        self.assertEqual((s["num"], s["den"]), (0, 0))

    def test_same_finding_can_match_two_leaks(self):
        self.env.fixture("m", "multi-leak",
                         leak_toml("L1", "a.py:5") + leak_toml("L2", "a.py:7", lines=[5, 9]))
        self.env.run("m", findings=[jf("a.py:5", "static/global")])
        m = self.env.group()["metrics"]
        self.assertEqual(m["leak_recall"]["num"], 2)

    def test_label_accuracy_and_forced_fit(self):
        body = (leak_toml("L1", "a.py:1", cause="L-A") + leak_toml("L2", "a.py:2", cause="L-B")
                + leak_toml("L3", "a.py:3", cause="UNCLASSIFIED") + leak_toml("L4", "a.py:4", cause="UNCLASSIFIED")
                + leak_toml("L5", "a.py:5", cause="L-C"))
        self.env.fixture("m", "multi-leak", body)
        self.env.run("m", findings=[
            jf("a.py:1", "h", cause="L-A"),            # accurate
            jf("a.py:2", "h", cause="L-A"),            # wrong label
            jf("a.py:3", "h", cause="L-A"),            # forced fit
            jf("a.py:4", "h", cause="UNCLASSIFIED"),   # fine
            jf("a.py:5", "h", cause="free text")])     # null label -> excluded
        m = self.env.group()["metrics"]
        self.assertEqual((m["label_accuracy"]["num"], m["label_accuracy"]["den"]), (1, 2))
        self.assertEqual((m["forced_fit_rate"]["num"], m["forced_fit_rate"]["den"]), (1, 2))


class ClaimMetricsTest(EnvCase):
    def setUp(self):
        super().setUp()
        nl = '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n'
        self.env.fixture("imp", "impossible", leak_toml("L1", "a.py:1", confirmable=False))
        self.env.fixture("dec", "decoy-control", nl, pair="x")
        self.env.fixture("non", "non-leak", nl)

    def test_overclaim(self):
        self.env.run("imp", n=1, findings=[jf("a.py:1", "h", "leak-confirmed")])
        self.env.run("imp", n=2, findings=[jf("a.py:1", "h", "leak-suspected")])
        self.env.run("imp", n=3, ledger_text=ledger(added("f", "a.py:1"), trans("f", "strong")))
        self.env.run("imp", n=4, ledger_text=ledger(added("f", "a.py:1", parent="p"), trans("f", "confirmed")))
        m = self.env.group()["metrics"]["overclaim_rate"]
        self.assertEqual((m["num"], m["den"]), (2, 4))

    def test_false_confirmation_and_false_alarm(self):
        self.env.run("dec", n=1, findings=[jf("q.py:1", "h", "leak-confirmed")])
        self.env.run("dec", n=2, findings=[jf("q.py:1", "h", "not-a-leak", disc="plateau")])
        self.env.run("non", n=1, findings=[jf("q.py:1", "h", "leak-confirmed")])
        self.env.run("non", n=2)
        m = self.env.group()["metrics"]
        self.assertEqual((m["false_confirmation_rate"]["num"], m["false_confirmation_rate"]["den"]), (2, 4))
        self.assertEqual((m["trap_false_alarm_rate"]["num"], m["trap_false_alarm_rate"]["den"]), (1, 2))

    def test_non_leak_precision(self):
        self.env.run("non", n=1, findings=[jf("q:1", "h", "not-a-leak", disc="plateau")])   # ok
        self.env.run("non", n=2, findings=[jf("q:1", "h", "not-a-leak", disc="plateau"),
                                           jf("q:2", "h", "leak-confirmed")])                  # claim
        self.env.run("non", n=3, findings=[jf("q:1", "h", "not-a-leak")])                      # no discriminator
        self.env.run("non", n=4, findings=[])                                                   # no rule-out
        self.env.run("non", n=5, ledger_text=ledger(
            added("h", "q:1", parent="p"), trans("h", "refuted")))                              # hypothesis rule-out counts
        self.env.run("non", n=6, ledger_text=ledger(
            added("h", "q:1", parent="p"), trans("h", "confirmed"),
            added("g", "q:2"), trans("g", "refuted")))                                          # hypothesis is no claim
        m = self.env.group()["metrics"]["non_leak_precision"]
        self.assertEqual((m["num"], m["den"]), (3, 6))

    def test_trap_miss_rate_requires_pair_and_all_confirmable_found(self):
        self.env.fixture("pa", "archetype-leak", leak_toml("L1", "a.py:1") + leak_toml("L2", "a.py:2")
                         + leak_toml("L3", "a.py:3", confirmable=False), pair="pb")
        self.env.fixture("nopair", "archetype-leak", leak_toml("L1", "a.py:1"))
        self.env.fixture("free", "archetype-free", leak_toml("L1", "a.py:1", cause="UNCLASSIFIED"), pair="z")
        self.env.run("pa", n=1, findings=[jf("a.py:1", "h"), jf("a.py:2", "h")])
        self.env.run("pa", n=2, findings=[jf("a.py:1", "h")])
        self.env.run("nopair", n=1, findings=[])
        self.env.run("free", n=1, findings=[])
        m = self.env.group()["metrics"]["trap_miss_rate"]
        self.assertEqual((m["num"], m["den"]), (1, 2))

    def test_fixtures_of_other_categories_do_not_feed_claim_metrics(self):
        self.env.fixture("t", "archetype-leak", leak_toml("L1", "a.py:1"))
        self.env.run("t", findings=[jf("zzz.py:1", "h", "leak-confirmed")])
        m = self.env.group()["metrics"]
        for name in ("overclaim_rate", "false_confirmation_rate", "non_leak_precision", "trap_false_alarm_rate"):
            self.assertEqual(m[name]["den"], 0, name)


class StatisticsTest(EnvCase):
    def setUp(self):
        super().setUp()
        self.env.fixture("t", "archetype-leak", leak_toml("L1", "a.py:1"))

    def test_per_run_mean_stdev(self):
        self.env.fixture("u", "archetype-leak", leak_toml("L1", "b.py:1"))
        self.env.run("t", n=1, findings=[jf("a.py:1", "h")])
        self.env.run("u", n=1, findings=[jf("b.py:1", "h")])
        self.env.run("t", n=2, findings=[])
        self.env.run("u", n=2, findings=[jf("b.py:1", "h")])
        self.env.run("t", n=3, findings=[])
        self.env.run("u", n=3, findings=[])
        m = self.env.group()["metrics"]["leak_recall"]
        self.assertEqual((m["num"], m["den"], m["value"]), (3, 6, 0.5))
        self.assertEqual([(p["run"], p["num"], p["den"], p["value"]) for p in m["per_run"]],
                         [(1, 2, 2, 1.0), (2, 1, 2, 0.5), (3, 0, 2, 0.0)])
        self.assertEqual(m["mean"], 0.5)
        # squared diffs 0.25 + 0 + 0.25 = 0.5 ; sqrt(0.5/3) = 0.408248
        self.assertEqual(m["stdev"], 0.408248)

    def test_null_per_run_values_are_skipped_in_mean_and_stdev(self):
        self.env.fixture("d", "decoy-control", '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n')
        self.env.run("t", n=1, findings=[jf("a.py:1", "h")])
        self.env.run("d", n=2, findings=[])
        self.env.run("t", n=3, findings=[])
        m = self.env.group()["metrics"]["leak_recall"]
        self.assertEqual([p["value"] for p in m["per_run"]], [1.0, None, 0.0])
        self.assertEqual(m["mean"], 0.5)
        self.assertEqual(m["stdev"], 0.5)

    def test_per_run_covers_every_index_up_to_largest(self):
        self.env.run("t", n=4, findings=[jf("a.py:1", "h")])
        m = self.env.group()["metrics"]["leak_recall"]
        self.assertEqual([p["run"] for p in m["per_run"]], [1, 2, 3, 4])
        self.assertEqual([p["den"] for p in m["per_run"]], [0, 0, 0, 1])
        self.assertEqual((m["mean"], m["stdev"]), (1.0, 0.0))

    def test_zero_denominator_everything_null(self):
        self.env.run("t", findings=[])
        m = self.env.group()["metrics"]["overclaim_rate"]
        self.assertEqual((m["num"], m["den"], m["value"], m["mean"], m["stdev"]), (0, 0, None, None, None))
        self.assertEqual(m["per_run"], [{"run": 1, "num": 0, "den": 0, "value": None}])

    def test_all_twelve_metrics_present(self):
        self.env.run("t", findings=[])
        m = self.env.group()["metrics"]
        self.assertEqual(sorted(m), sorted(grade.METRIC_NAMES))
        self.assertEqual(len(m), 12)
        for v in m.values():
            self.assertEqual(sorted(v), ["den", "mean", "num", "per_run", "stdev", "value"])

    def test_value_rounding_tie(self):
        self.env.fixture("big", "archetype-leak", "".join(leak_toml(f"L{i}", f"a.py:{i}") for i in range(1, 129)))
        self.env.run("big", findings=[jf("a.py:1", "h")])
        m = self.env.group()["metrics"]["leak_recall"]
        self.assertEqual((m["num"], m["den"]), (1, 128))
        self.assertEqual(m["value"], 0.007812)

    def test_mean_uses_unrounded_per_run_ratios(self):
        self.env.fixture("m3", "multi-leak",
                         leak_toml("L1", "m.py:1") + leak_toml("L2", "m.py:2") + leak_toml("L3", "m.py:3"))
        self.env.run("m3", n=1, findings=[jf("m.py:1", "h"), jf("m.py:2", "h"), jf("m.py:3", "h")])
        self.env.run("m3", n=2, findings=[jf("m.py:1", "h")])
        m = self.env.group()["metrics"]["leak_recall"]
        self.assertEqual([p["value"] for p in m["per_run"]], [1.0, 0.333333])
        vals = [1.0, 1 / 3]
        s = sum(vals)
        self.assertEqual(m["mean"], grade.round6(s / 2))
        q = sum((v - s / 2) ** 2 for v in vals)
        self.assertEqual(m["stdev"], grade.round6((q / 2) ** 0.5))
        # 2/3 = 0.666667 exactly from unrounded ratios; stdev = 1/3 -> 0.333333
        self.assertEqual((m["mean"], m["stdev"]), (0.666667, 0.333333))

    def test_mean_where_rounded_and_unrounded_differ(self):
        # per-run ratios 1/3, 1/3, 1/3 -> mean 1/3; rounded inputs would give the same,
        # so use 2/3 and 1/3 with a third run: 1/3 stays distinct from 0.333333
        self.env.fixture("m3", "multi-leak",
                         leak_toml("L1", "m.py:1") + leak_toml("L2", "m.py:2") + leak_toml("L3", "m.py:3"))
        for n in (1, 2, 3):
            self.env.run("m3", n=n, findings=[jf("m.py:1", "h")])
        m = self.env.group()["metrics"]["leak_recall"]
        vals = [1 / 3] * 3
        self.assertEqual(m["mean"], grade.round6(sum(vals) / 3))
        self.assertEqual(m["stdev"], 0.0)


class UsageTest(EnvCase):
    def setUp(self):
        super().setUp()
        self.env.fixture("t", "non-leak", '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n')

    def test_sums_means_and_errors(self):
        self.env.run("t", n=1, ledger_text=ledger(added("f", "q:1")),
                     run_json={"wall_seconds": 10.0, "cost_usd": 1.0,
                               "usage": {"input_tokens": 100, "output_tokens": 10,
                                         "cache_read_input_tokens": 1000, "cache_creation_input_tokens": 5}})
        self.env.run("t", n=2, findings=[], run_json={"wall_seconds": 20.5, "cost_usd": 2.0, "exit_code": 1,
                                                      "usage": {"input_tokens": 201}})
        self.env.run("t", n=3, run_json={"wall_seconds": 5, "cost_usd": None, "timed_out": True, "usage": {}})
        u = self.env.group()["usage"]
        self.assertEqual(u["runs"], 3)
        self.assertEqual(u["errored"], 2)
        self.assertEqual(u["sources"], {"ledger": 1, "findings": 1, "none": 1})
        self.assertEqual(u["input_tokens"], {"sum": 301, "mean": 100.333333})
        self.assertEqual(u["output_tokens"], {"sum": 10, "mean": 3.333333})
        self.assertEqual(u["cache_read_input_tokens"], {"sum": 1000, "mean": 333.333333})
        self.assertEqual(u["cache_creation_input_tokens"], {"sum": 5, "mean": 1.666667})
        self.assertEqual(u["wall_seconds"], {"sum": 35.5, "mean": 11.833333, "max": 20.5})
        self.assertEqual(u["cost_usd"], {"sum": 3.0, "mean": 1.5})

    def test_no_numeric_cost(self):
        self.env.run("t", run_json={"cost_usd": None})
        self.assertEqual(self.env.group()["usage"]["cost_usd"], {"sum": 0.0, "mean": None})

    def test_missing_usage_keys_count_as_zero(self):
        d = self.env.run("t")
        (d / "run.json").write_text(json.dumps({"exit_code": 0, "timed_out": False, "wall_seconds": 1}))
        u = self.env.group()["usage"]
        self.assertEqual(u["input_tokens"], {"sum": 0, "mean": 0.0})
        self.assertEqual(u["errored"], 0)

    def test_nonzero_exit_without_timeout_is_errored(self):
        self.env.run("t", run_json={"exit_code": 2})
        self.assertEqual(self.env.group()["usage"]["errored"], 1)

    def test_timeout_with_zero_exit_is_errored(self):
        self.env.run("t", run_json={"timed_out": True})
        self.assertEqual(self.env.group()["usage"]["errored"], 1)


class UnexplainedGrowthTest(EnvCase):
    def setUp(self):
        super().setUp()
        self.env.fixture("t", "non-leak", '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n')

    def share(self):
        return self.env.group()["unexplained_growth_share"]

    def test_none(self):
        self.env.run("t")
        self.assertEqual(self.share(), {"value": None, "runs": 0})

    def test_sum_over_runs_and_rows(self):
        self.env.run("t", n=1, coverage={"residual": [{"residual": 1, "total_slope": 10},
                                                      {"residual": 2, "total_slope": 20}]})
        self.env.run("t", n=2, coverage={"residual": [{"residual": 3.5, "total_slope": 70}]})
        self.env.run("t", n=3, coverage={"residual": []})
        self.env.run("t", n=4, coverage={"other": 1})
        self.env.run("t", n=5, coverage="{broken")
        self.assertEqual(self.share(), {"value": 0.065, "runs": 2})   # 6.5 / 100

    def test_zero_total_is_null_but_counts_runs(self):
        self.env.run("t", coverage={"residual": [{"residual": 0, "total_slope": 0}]})
        self.assertEqual(self.share(), {"value": None, "runs": 1})


class FixtureBlockTest(EnvCase):
    def test_blocks(self):
        self.env.fixture("m", "multi-leak", leak_toml("L1", "a.py:1") + leak_toml("L2", "a.py:2", confirmable=False))
        self.env.fixture("n", "non-leak", '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n')
        self.env.run("m", n=1, findings=[jf("a.py:1", "static/global", "leak-confirmed"),
                                         jf("a.py:2", "other", "leak-suspected")])
        self.env.run("m", n=2, findings=[jf("a.py:1", "wrong", "leak-confirmed"),
                                         jf("a.py:2", "static/global", "leak-confirmed")])
        self.env.run("m", n=3, findings=[jf("a.py:1", "static/global", "not-a-leak", disc="d")])
        self.env.run("n", n=1, findings=[jf("q:1", None, "not-a-leak", disc="d")])
        fx = self.env.group()["fixtures"]
        self.assertEqual(fx["m"], {
            "category": "multi-leak", "runs": 3, "claim_runs": 2, "rule_out_runs": 1,
            "leaks": {"L1": {"found": 2, "flagged": 2, "grounded": 1},
                      "L2": {"found": 1, "flagged": 2, "grounded": 1}}})
        self.assertEqual(fx["n"], {"category": "non-leak", "runs": 1, "claim_runs": 0,
                                   "rule_out_runs": 1, "leaks": {}})


class GroupingAndErrorsTest(EnvCase):
    def setUp(self):
        super().setUp()
        self.env.fixture("t", "non-leak", '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n')

    def test_groups_sorted_by_ablation_then_archetypes(self):
        for abl, arch in (("prose", "on"), ("full", "on"), ("no-skill", "off"), ("full", "off"), ("no-skill", "on")):
            self.env.run("t", abl=abl, arch=arch)
        got = [(g["ablation"], g["archetypes"]) for g in self.env.grade()["groups"]]
        self.assertEqual(got, [("full", "off"), ("full", "on"), ("no-skill", "off"), ("no-skill", "on"),
                               ("prose", "on")])

    def test_missing_expected_toml_is_an_error(self):
        self.env.run("ghost")
        with self.assertRaises(grade.GradeError):
            self.env.grade()

    def test_run_dir_without_run_json_is_skipped(self):
        self.env.run("t", n=1)
        self.env.run("t", n=2, write_run_json=False, ledger_text=ledger(added("f", "q:1")))
        g = self.env.group()
        self.assertEqual(g["usage"]["runs"], 1)
        self.assertEqual(g["usage"]["incomplete"], 1)
        self.assertEqual(len(g["metrics"]["leak_recall"]["per_run"]), 1)

    def test_run_dir_without_run_json_does_not_need_fixture(self):
        self.env.run("ghost", write_run_json=False)
        self.env.run("t")
        self.assertEqual(self.env.group()["usage"]["runs"], 1)

    def test_unparsable_run_json_is_an_error(self):
        d = self.env.run("t")
        (d / "run.json").write_text("{nope")
        with self.assertRaises(grade.GradeError):
            self.env.grade()

    def test_nonnumeric_dirs_and_foreign_dirs_ignored(self):
        self.env.run("t")
        base = self.env.results / "runs" / "no-skill" / "archetypes-on" / "t"
        (base / "notes").mkdir()
        (base / "0").mkdir()
        (self.env.results / "runs" / "no-skill" / "scratch").mkdir()
        (self.env.results / "runs" / "stray.txt").write_text("x")
        self.assertEqual(self.env.group()["usage"]["runs"], 1)

    def test_empty_results_has_no_groups(self):
        (self.env.results / "runs").mkdir()
        self.assertEqual(self.env.grade(), {"schema_version": 1, "groups": []})

    def test_missing_results_dir_is_an_error(self):
        with self.assertRaises(grade.GradeError):
            grade.grade(self.env.root / "nope", self.env.fixtures)

    def test_repeated_id_replacement_end_to_end(self):
        self.env.fixture("t2", "archetype-leak", leak_toml("L1", "a.py:1"))
        self.env.run("t2", ledger_text=ledger(added("f", "a.py:1"), trans("f", "confirmed"),
                                              added("f", "zzz.py:9")))
        m = self.env.group()["metrics"]
        self.assertEqual((m["leak_recall"]["num"], m["flagged_recall"]["num"]), (0, 0))


class ClarifiedRulesTest(EnvCase):
    """Rules added when the grading contract was clarified."""

    def fx(self, body=None):
        self.env.fixture("t", "archetype-leak", body or leak_toml("L1", "a.py:5"))

    def metrics(self, **kw):
        self.env.run("t", **kw)
        return self.env.group()["metrics"]

    # directory rules
    def test_only_exact_archetype_dirs_are_groups(self):
        self.fx()
        self.env.run("t")
        for name in ("archetypes-", "archetypes-maybe", "archetypes-On", "extra"):
            d = self.env.results / "runs" / "no-skill" / name / "t" / "1"
            d.mkdir(parents=True)
            (d / "run.json").write_text("{}")
        self.assertEqual([(g["ablation"], g["archetypes"]) for g in self.env.grade()["groups"]],
                         [("no-skill", "on")])

    def test_run_dir_names_must_be_positive_integers_without_leading_zeros(self):
        self.fx()
        self.env.run("t", n=1)
        base = self.env.results / "runs" / "no-skill" / "archetypes-on" / "t"
        for name in ("01", "001", "0", "00", "-1", "+1", "1.0", "1a", "\u0663"):
            (base / name).mkdir()
            (base / name / "run.json").write_text("{}")
        self.env.run("t", n=10)
        g = self.env.group()
        self.assertEqual(g["usage"]["runs"], 2)
        self.assertEqual(g["usage"]["incomplete"], 0)
        self.assertEqual(len(g["metrics"]["leak_recall"]["per_run"]), 10)

    def test_incomplete_counts_skipped_dirs_only_in_their_group(self):
        self.fx()
        self.env.run("t", n=1)
        self.env.run("t", n=2, write_run_json=False)
        self.env.run("t", n=3, write_run_json=False)
        self.env.run("t", n=1, abl="prose")
        self.assertEqual(self.env.group()["usage"]["incomplete"], 2)
        self.assertEqual(self.env.group("prose")["usage"]["incomplete"], 0)

    def test_run_json_not_an_object_is_an_error(self):
        self.fx()
        for text in ("[]", "3", "null", '"x"'):
            env = Env()
            self.addCleanup(env.close)
            env.fixture("t", "archetype-leak", leak_toml("L1", "a.py:5"))
            d = env.run("t")
            (d / "run.json").write_text(text)
            with self.assertRaises(grade.GradeError):
                env.grade()

    # run.json fields
    def test_run_json_field_coercions(self):
        self.fx()
        self.env.run("t", run_json={"usage": {"input_tokens": "9", "output_tokens": 2.5,
                                              "cache_read_input_tokens": None,
                                              "cache_creation_input_tokens": True},
                                    "wall_seconds": "fast", "cost_usd": "free"})
        u = self.env.group()["usage"]
        for k in grade.TOKEN_FIELDS:
            self.assertEqual(u[k], {"sum": 0, "mean": 0.0})
        self.assertEqual(u["wall_seconds"], {"sum": 0.0, "mean": 0.0, "max": 0.0})
        self.assertEqual(u["cost_usd"], {"sum": 0.0, "mean": None})

    def test_errored_when_exit_code_missing_or_not_integer(self):
        self.fx()
        self.env.run("t", n=1, run_json={"exit_code": None})
        self.env.run("t", n=2, run_json={"exit_code": "0"})
        self.env.run("t", n=3, run_json={"exit_code": 0.0})
        self.env.run("t", n=4, run_json={"exit_code": True})
        self.env.run("t", n=5)
        d = self.env.run("t", n=6)
        rj = json.loads((d / "run.json").read_text())
        del rj["exit_code"]
        (d / "run.json").write_text(json.dumps(rj))
        self.assertEqual(self.env.group()["usage"]["errored"], 5)

    # fixture fields
    def test_confirmable_defaults_to_true(self):
        self.fx(leak_toml("L1", "a.py:5", confirmable=None) + leak_toml("L2", "a.py:6", confirmable=False))
        m = self.metrics(findings=[jf("a.py:5", "h")])
        self.assertEqual((m["leak_recall"]["num"], m["leak_recall"]["den"]), (1, 1))
        self.assertEqual(m["flagged_recall"]["den"], 1)

    def test_default_confirmable_counts_in_trap_miss(self):
        self.env.fixture("t", "archetype-leak", leak_toml("L1", "a.py:5", confirmable=None), pair="u")
        m = self.metrics(findings=[])
        self.assertEqual((m["trap_miss_rate"]["num"], m["trap_miss_rate"]["den"]), (1, 1))

    def test_unconfirmable_leaks_still_listed_per_fixture(self):
        self.fx(leak_toml("L1", "a.py:5") + leak_toml("L2", "a.py:6", confirmable=False))
        self.env.run("t", findings=[jf("a.py:5", "static/global"), jf("a.py:6", "static/global")])
        leaks = self.env.group()["fixtures"]["t"]["leaks"]
        self.assertEqual(leaks["L1"], {"found": 1, "flagged": 1, "grounded": 1})
        self.assertEqual(leaks["L2"], {"found": 1, "flagged": 1, "grounded": 1})

    def test_expected_site_path_used_as_written(self):
        self.fx(leak_toml("L1", "./src/a.py:5"))
        m = self.metrics(findings=[jf("src/a.py:5", "h")])
        self.assertEqual(m["leak_recall"]["num"], 0)
        self.fx(leak_toml("L1", "src\\\\a.py:5"))
        self.env.run("t", n=2, findings=[jf("src/a.py:5", "h")])
        self.assertEqual(self.env.group()["metrics"]["leak_recall"]["per_run"][1]["num"], 0)

    def test_alt_sites_paths_used_as_written(self):
        self.fx(leak_toml("L1", "z.py:1", alts=["./b/c.py:7", "d/e.py:8"]))
        self.env.run("t", n=1, findings=[jf("b/c.py:7", "h")])
        self.env.run("t", n=2, findings=[jf("./b/c.py:7", "h")])
        self.env.run("t", n=3, findings=[jf("d/e.py:8", "h")])
        per = self.env.group()["metrics"]["leak_recall"]["per_run"]
        self.assertEqual([p["num"] for p in per], [0, 0, 1])

    def test_malformed_alt_sites_ignored(self):
        alts = ["c.py", "c.py:", "c.py:x", "c.py:1-", "c.py:-3", "c.py:1-x", "c.py:1,3", ""]
        self.fx(leak_toml("L1", "z.py:1", alts=alts))
        for i, site in enumerate(["c.py:1", "c.py:3", "c.py", "c.py:2", "c.py:-3"], start=1):
            self.env.run("t", n=i, findings=[jf(site, "h")])
        m = self.env.group()["metrics"]["leak_recall"]
        self.assertEqual(m["num"], 0)
        # a valid entry beside the malformed ones still works
        self.fx(leak_toml("L1", "z.py:1", alts=alts + ["c.py:2-4"]))
        self.env.run("t", n=6, findings=[jf("c.py:3", "h")])
        self.assertEqual(self.env.group()["metrics"]["leak_recall"]["per_run"][5]["num"], 1)

    # float sums in ascending fixture, run order
    def test_float_sums_follow_fixture_then_run_order(self):
        for fid in ("b", "a"):
            self.env.fixture(fid, "archetype-leak", leak_toml("L1", "a.py:5"))
        wall = {("a", 1): 0.1, ("a", 2): 0.2, ("b", 1): 0.3, ("b", 2): 1e16, ("b", 3): -1e16}
        for (fid, n), w in wall.items():
            self.env.run(fid, n=n, run_json={"wall_seconds": w, "cost_usd": w})
        expect = 0.0
        for k in sorted(wall):
            expect += wall[k]
        u = self.env.group()["usage"]
        self.assertEqual(u["wall_seconds"]["sum"], grade.round6(expect))
        self.assertEqual(u["cost_usd"]["sum"], grade.round6(expect))

    # findings: repeated id and invalid finding_added
    def test_repeated_id_moves_to_later_position(self):
        # f1 re-added after f2: order becomes f2, f1(new); first match of state
        # strong picks f2's holder.
        self.fx()
        led = ledger(added("f1", "a.py:5", "old"), trans("f1", "strong"),
                     added("f2", "a.py:5", "mid"), trans("f2", "strong"),
                     added("f1", "a.py:5", "new"), trans("f1", "strong"))
        m = self.metrics(ledger_text=led)
        self.assertEqual(m["grounded_attribution"]["num"], 0)
        res = self.env.grade()
        self.assertEqual(res["groups"][0]["fixtures"]["t"]["leaks"]["L1"]["found"], 1)

    def test_repeated_id_position_decides_chosen_holder(self):
        self.fx(leak_toml("L1", "a.py:5", holder="static/global"))
        led = ledger(added("f1", "a.py:5", "wrong"), trans("f1", "strong"),
                     added("f2", "a.py:5", "static/global"), trans("f2", "strong"),
                     added("f1", "a.py:5", "wrong"), trans("f1", "strong"))
        m = self.metrics(ledger_text=led)
        # order is f2, f1: chosen = f2 (matching holder). Without the move it would be f1.
        self.assertEqual((m["grounded_attribution"]["num"], m["grounded_attribution"]["den"]), (1, 1))

    def test_invalid_finding_added_still_selects_ledger_source(self):
        self.fx()
        bad = [json.dumps({"type": "finding_added", "finding": "x"}),
               json.dumps({"type": "finding_added", "finding": {"id": 7, "site": "a.py:5"}}),
               json.dumps({"type": "finding_added"})]
        for i, line in enumerate(bad, start=1):
            self.env.run("t", n=i, ledger_text=ledger(line),
                         findings=[jf("a.py:5", "h")])
        g = self.env.group()
        self.assertEqual(g["usage"]["sources"], {"ledger": 3, "findings": 0, "none": 0})
        self.assertEqual(g["metrics"]["leak_recall"]["num"], 0)

    def test_other_ledger_records_do_not_select_ledger_source(self):
        self.fx()
        self.env.run("t", ledger_text=ledger(trans("f", "confirmed")), findings=[jf("a.py:5", "h")])
        self.assertEqual(self.env.group()["usage"]["sources"]["findings"], 1)

    # whole-string site match
    def test_site_with_trailing_newline_has_no_location(self):
        self.fx()
        self.assertIsNone(grade.parse_site("a.py:5\n"))
        self.assertEqual(grade.parse_site("a.py:5"), ("a.py", 5))
        m = self.metrics(findings=[jf("a.py:5\n", "h")])
        self.assertEqual(m["leak_recall"]["num"], 0)
        self.assertEqual(m["flagged_recall"]["num"], 0)

    # chosen finding ignores hypotheses
    def test_hypothesis_never_chosen(self):
        self.fx(leak_toml("L1", "a.py:5", holder="static/global"))
        led = ledger(added("h1", "a.py:5", "wrong", parent="root"), trans("h1", "strong"),
                     added("f1", "a.py:5", "static/global"), trans("f1", "confirmed"))
        m = self.metrics(ledger_text=led)
        self.assertEqual((m["grounded_attribution"]["num"], m["grounded_attribution"]["den"]), (1, 1))

    def test_hypothesis_alone_finds_nothing(self):
        self.fx()
        m = self.metrics(ledger_text=ledger(added("h1", "a.py:5", "x", parent="root"), trans("h1", "strong")))
        self.assertEqual((m["leak_recall"]["num"], m["flagged_recall"]["num"]), (0, 0))

    # second leak recall only counts the leak set
    def test_second_leak_recall_counts_leak_set_only(self):
        body = (leak_toml("L1", "m.py:1") + leak_toml("L2", "m.py:2")
                + leak_toml("L3", "m.py:3", confirmable=False))
        self.env.fixture("t", "multi-leak", body)
        self.env.run("t", n=1, findings=[jf("m.py:1", "h"), jf("m.py:2", "h"), jf("m.py:3", "h")])
        self.env.run("t", n=2, findings=[jf("m.py:3", "h")])  # only the unconfirmable: no contribution
        self.env.run("t", n=3, findings=[jf("m.py:1", "h"), jf("m.py:3", "h")])
        m = self.env.group()["metrics"]["second_leak_recall"]
        got = [(p["num"], p["den"]) for p in m["per_run"]]
        self.assertEqual(got, [(1, 1), (0, 0), (0, 1)])
        self.assertEqual((m["num"], m["den"]), (1, 2))

    # per-run ratios with den 0 are skipped, not zero
    def test_mean_skips_zero_denominator_entries(self):
        self.env.fixture("t", "multi-leak", leak_toml("L1", "m.py:1") + leak_toml("L2", "m.py:2"))
        self.env.run("t", n=1, findings=[jf("m.py:1", "h"), jf("m.py:2", "h")])
        self.env.run("t", n=2, findings=[])
        self.env.run("t", n=3, findings=[jf("m.py:2", "h")])
        m = self.env.group()["metrics"]["second_leak_recall"]
        self.assertEqual([(p["num"], p["den"]) for p in m["per_run"]], [(1, 1), (0, 0), (0, 1)])
        self.assertEqual((m["mean"], m["stdev"]), (0.5, 0.5))

    # coverage.json
    def test_coverage_edge_cases(self):
        self.fx()
        cases = {
            1: {"residual": [{"residual": 2, "total_slope": 4}, "junk", [1],
                             {"residual": "x", "total_slope": 4}, {"total_slope": 2}]},
            2: "{not json",
            3: [1, 2],
            4: {"residual": []},
            5: {"residual": ["junk"]},
            6: {"residual": {"residual": 1, "total_slope": 1}},
        }
        for n, cov in cases.items():
            self.env.run("t", n=n, coverage=cov)
        s = self.env.group()["unexplained_growth_share"]
        # run 1: R = 2, T = 4 + 4 + 2 = 10; run 5: no numeric values, counts as a run with 0, 0
        self.assertEqual(s, {"value": 0.2, "runs": 2})


class ClarifiedRules2Test(EnvCase):
    """Encoding, whitespace, ablation, runs/ and unfinished-group rules."""

    def fx(self):
        self.env.fixture("t", "archetype-leak", leak_toml("L1", "a.py:5"))

    # encoding
    def test_run_json_invalid_utf8_is_an_error(self):
        self.fx()
        d = self.env.run("t")
        (d / "run.json").write_bytes(b'{"exit_code": 0, "x": "\xff"}')
        with self.assertRaises(grade.GradeError):
            self.env.grade()

    def test_expected_toml_invalid_utf8_is_an_error(self):
        self.fx()
        self.env.run("t")
        (self.env.fixtures / "t" / "expected.toml").write_bytes(
            fixture_toml("t", "archetype-leak").encode() + b'\n# \xff\n')
        with self.assertRaises(grade.GradeError):
            self.env.grade()

    def test_findings_json_invalid_utf8_treated_as_absent(self):
        self.fx()
        d = self.env.run("t", findings=[jf("a.py:5")])
        (d / "findings.json").write_bytes(b'{"findings": [{"site": "a.py:5", "verdict": "\xff"}]}')
        g = self.env.group()
        self.assertEqual(g["usage"]["sources"], {"ledger": 0, "findings": 0, "none": 1})

    def test_coverage_json_invalid_utf8_treated_as_absent(self):
        self.fx()
        d = self.env.run("t")
        (d / "coverage.json").write_bytes(b'{"residual": [{"residual": 1, "total_slope": 2, "x": "\xff"}]}')
        self.assertEqual(self.env.group()["unexplained_growth_share"], {"value": None, "runs": 0})

    def test_ledger_bad_line_does_not_poison_other_lines(self):
        p = self.env.root / "l.jsonl"
        p.write_bytes(added("f", "a:1").encode() + b"\n\xff\xfe{\n" + trans("f", "strong").encode() + b"\n")
        saw, fs = grade.parse_ledger(p)
        self.assertEqual([(f["site"], f["state"]) for f in fs], [("a:1", "strong")])

    # whitespace
    def test_ledger_trims_only_ascii_whitespace(self):
        p = self.env.root / "l.jsonl"
        ok = added("f", "a:1")
        lines = [" \t" + ok + "\r", "\u00a0" + added("g", "b:1"), added("h", "c:1") + "\u2003",
                 "\x0b" + added("i", "d:1"), "\x0c" + added("j", "e:1")]
        p.write_bytes("\n".join(lines).encode("utf-8"))
        saw, fs = grade.parse_ledger(p)
        self.assertEqual([f["site"] for f in fs], ["a:1"])

    def test_discriminator_blank_check_is_ascii_only(self):
        for disc, want in ((" \t\r\n", False), ("\u00a0", True), ("\u2003", True), ("\x0b", True), (" x ", True)):
            fs = grade.parse_findings_json(self._write({"findings": [jf("a:1", verdict="not-a-leak", disc=disc)]}))
            self.assertEqual(fs[0]["discriminator"], want, repr(disc))

    def _write(self, doc):
        p = self.env.root / "f.json"
        p.write_text(json.dumps(doc), encoding="utf-8")
        return p

    # directories
    def test_unknown_ablation_dirs_ignored(self):
        self.fx()
        self.env.run("t")
        for name in ("scratch", "No-Skill", "full "):
            d = self.env.results / "runs" / name / "archetypes-on" / "ghost" / "1"
            d.mkdir(parents=True)
            (d / "run.json").write_text("{}")
        self.assertEqual([(g["ablation"], g["archetypes"]) for g in self.env.grade()["groups"]],
                         [("no-skill", "on")])

    def test_all_four_ablations_accepted(self):
        self.fx()
        for abl in ("no-skill", "prose", "gates", "full"):
            self.env.run("t", abl=abl)
        self.assertEqual([g["ablation"] for g in self.env.grade()["groups"]],
                         ["full", "gates", "no-skill", "prose"])

    def test_missing_runs_dir_is_an_error(self):
        with self.assertRaises(grade.GradeError):
            self.env.grade()

    def test_runs_that_is_a_file_is_an_error(self):
        (self.env.results / "runs").write_text("x")
        with self.assertRaises(grade.GradeError):
            self.env.grade()

    def test_cli_missing_runs_dir_exits_1(self):
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            rc = grade.main([str(self.env.results), "--fixtures", str(self.env.fixtures),
                             "--out", str(self.env.root / "m.json")])
        self.assertEqual(rc, 1)
        self.assertFalse((self.env.root / "m.json").exists())

    # unfinished-only groups
    def test_group_with_only_unfinished_runs_is_written_with_zero_runs(self):
        self.env.run("ghost", n=1, write_run_json=False)
        self.env.run("ghost", n=2, write_run_json=False)
        g = self.env.group()
        u = g["usage"]
        self.assertEqual((u["runs"], u["incomplete"], u["errored"]), (0, 2, 0))
        self.assertEqual(u["sources"], {"ledger": 0, "findings": 0, "none": 0})
        for k in grade.TOKEN_FIELDS:
            self.assertEqual(u[k], {"sum": 0, "mean": None})
        self.assertEqual(u["wall_seconds"], {"sum": 0.0, "mean": None, "max": None})
        self.assertEqual(u["cost_usd"], {"sum": 0.0, "mean": None})
        self.assertEqual(g["fixtures"], {})
        self.assertEqual(g["unexplained_growth_share"], {"value": None, "runs": 0})
        for name in grade.METRIC_NAMES:
            m = g["metrics"][name]
            self.assertEqual((m["num"], m["den"], m["value"], m["mean"], m["stdev"], m["per_run"]),
                             (0, 0, None, None, None, []))

    def test_unfinished_only_group_next_to_finished_group(self):
        self.fx()
        self.env.run("t", abl="prose")
        self.env.run("t", abl="full", write_run_json=False)
        got = {(g["ablation"], g["usage"]["runs"], g["usage"]["incomplete"]) for g in self.env.grade()["groups"]}
        self.assertEqual(got, {("prose", 1, 0), ("full", 0, 1)})

    def test_expected_toml_needed_only_for_fixtures_with_a_finished_run(self):
        self.fx()
        self.env.run("t", n=1)
        self.env.run("ghost", n=1, write_run_json=False)
        g = self.env.group()
        self.assertEqual(list(g["fixtures"]), ["t"])
        self.assertEqual(g["usage"]["incomplete"], 1)
        self.env.run("ghost", n=2)
        with self.assertRaises(grade.GradeError):
            self.env.grade()

    def test_empty_group_json_serialises(self):
        self.env.run("ghost", write_run_json=False)
        json.dumps(self.env.grade(), allow_nan=False)


class CliTest(EnvCase):
    def setUp(self):
        super().setUp()
        self.env.fixture("t", "non-leak", '\n[[non_leak]]\nid = "N1"\ncause = "N-WARM"\ndiscriminator = "x"\n')
        self.env.run("t")

    def call(self, *argv):
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            rc = grade.main(list(argv))
        return rc, err.getvalue()

    def test_default_out_and_format(self):
        rc, _ = self.call(str(self.env.results), "--fixtures", str(self.env.fixtures))
        self.assertEqual(rc, 0)
        raw = (self.env.results / "metrics.json").read_bytes()
        self.assertTrue(raw.endswith(b"\n"))
        self.assertFalse(raw.endswith(b"\n\n"))
        self.assertEqual(json.loads(raw)["schema_version"], 1)

    def test_explicit_out(self):
        out = self.env.root / "sub" / "m.json"
        rc, _ = self.call(str(self.env.results), "--fixtures", str(self.env.fixtures), "--out", str(out))
        self.assertEqual(rc, 0)
        self.assertTrue(out.is_file())
        self.assertFalse((self.env.results / "metrics.json").exists())

    def test_error_exit_status(self):
        self.env.run("ghost")
        rc, err = self.call(str(self.env.results), "--fixtures", str(self.env.fixtures))
        self.assertEqual(rc, 1)
        self.assertIn("ghost", err)


class GoldenSampleTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.doc = grade.grade(SAMPLE / "results", SAMPLE / "fixtures")

    def group(self, abl, arch):
        return next(g for g in self.doc["groups"] if (g["ablation"], g["archetypes"]) == (abl, arch))

    def test_matches_committed_golden(self):
        golden = json.loads((SAMPLE / "metrics.json").read_text(encoding="utf-8"))
        self.assertEqual(self.doc, golden)

    def test_golden_file_format(self):
        self.assertTrue((SAMPLE / "metrics.json").read_bytes().endswith(b"\n"))

    def test_group_order(self):
        self.assertEqual([(g["ablation"], g["archetypes"]) for g in self.doc["groups"]],
                         [("full", "off"), ("full", "on"), ("no-skill", "on"), ("prose", "on")])

    # hand-calculated values (the arithmetic is in the task summary)

    def test_hand_leak_recall_no_skill(self):
        m = self.group("no-skill", "on")["metrics"]["leak_recall"]
        # leak set per run index = 1 (A) + 1 (C) + 2 (F) + 1 (G) = 5; found 5, 4, 0
        self.assertEqual((m["num"], m["den"], m["value"]), (9, 15, 0.6))
        self.assertEqual([(p["num"], p["den"]) for p in m["per_run"]], [(5, 5), (4, 5), (0, 5)])
        self.assertEqual((m["mean"], m["stdev"]), (0.6, 0.432049))     # sqrt(0.56 / 3)

    def test_hand_flagged_and_second_leak_no_skill(self):
        mm = self.group("no-skill", "on")["metrics"]
        self.assertEqual((mm["flagged_recall"]["num"], mm["flagged_recall"]["den"]), (11, 15))
        s = mm["second_leak_recall"]
        self.assertEqual((s["num"], s["den"], s["value"]), (1, 2, 0.5))
        self.assertEqual([p["value"] for p in s["per_run"]], [1.0, 0.0, None])

    def test_hand_grounded_and_labels_no_skill(self):
        mm = self.group("no-skill", "on")["metrics"]
        self.assertEqual((mm["grounded_attribution"]["num"], mm["grounded_attribution"]["den"]), (7, 9))
        self.assertEqual((mm["label_accuracy"]["num"], mm["label_accuracy"]["den"]), (4, 6))
        self.assertEqual((mm["forced_fit_rate"]["num"], mm["forced_fit_rate"]["den"]), (1, 3))

    def test_hand_claim_rates_no_skill(self):
        mm = self.group("no-skill", "on")["metrics"]
        want = {"overclaim_rate": (1, 3), "false_confirmation_rate": (2, 6), "non_leak_precision": (1, 3),
                "trap_miss_rate": (1, 3), "trap_false_alarm_rate": (1, 3)}
        for name, nd in want.items():
            self.assertEqual((mm[name]["num"], mm[name]["den"]), nd, name)

    def test_hand_unexplained_growth_full_on(self):
        # (10 + 5 + 2.5 + 0) / (100 + 50 + 20 + 30) = 17.5 / 200 = 0.0875 over 3 runs
        self.assertEqual(self.group("full", "on")["unexplained_growth_share"], {"value": 0.0875, "runs": 3})

    def test_hand_full_on_ledger_rules(self):
        mm = self.group("full", "on")["metrics"]
        self.assertEqual([(p["num"], p["den"]) for p in mm["leak_recall"]["per_run"]], [(5, 5), (3, 5), (2, 5)])
        self.assertEqual([(p["num"], p["den"]) for p in mm["grounded_attribution"]["per_run"]],
                         [(5, 5), (2, 3), (1, 2)])
        self.assertEqual([(p["num"], p["den"]) for p in mm["forced_fit_rate"]["per_run"]],
                         [(0, 1), (0, 0), (1, 1)])

    def test_hand_usage_no_skill(self):
        u = self.group("no-skill", "on")["usage"]
        # input_tokens = 1000 n + 100 i over n = 1..3, i = 0..6 -> 42000 + 6300
        self.assertEqual(u["input_tokens"], {"sum": 48300, "mean": 2300.0})
        self.assertEqual((u["runs"], u["errored"]), (21, 1))
        self.assertEqual(u["sources"], {"ledger": 0, "findings": 19, "none": 2})

    def test_sparse_group_per_run(self):
        m = self.group("full", "off")["metrics"]["leak_recall"]
        # run1: A (alt site 7) and C found -> 2/2; run2: F L2 only (line 41 in [38,41]) -> 1/2; run3: A line 21 -> 1/1
        self.assertEqual([(p["num"], p["den"]) for p in m["per_run"]], [(2, 2), (1, 2), (1, 1)])
        self.assertEqual((m["num"], m["den"]), (4, 5))

    def test_cli_reproduces_golden(self):
        with tempfile.TemporaryDirectory() as t:
            out = Path(t) / "m.json"
            rc = grade.main([str(SAMPLE / "results"), "--fixtures", str(SAMPLE / "fixtures"), "--out", str(out)])
            self.assertEqual(rc, 0)
            self.assertEqual(out.read_bytes(), (SAMPLE / "metrics.json").read_bytes())


if __name__ == "__main__":
    unittest.main()
