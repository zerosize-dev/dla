"""Tests of the agent's line counts (DLA_LINES) and of whose a holder is.

Run with the interpreter that runs the targets, from any directory:

    python3 -m unittest discover -s harness/python -p 'test_agent_*.py'

Each test starts a fresh interpreter with the agent first on PYTHONPATH, as `dla harness run`
does, so that nothing of the test runner is in the process that is counted.
"""

import json
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))

MODULE = textwrap.dedent(
    """\
    REG = []


    def add(x, take):
        if take:
            REG.append(x)
        return 1


    def one():
        REG.append(0)
    """
)
# Lines of MODULE: 5 is `if take:`, 6 is `REG.append(x)`, 7 is `return 1`, 11 is `REG.append(0)`.


def run_agent(driver, lines=None, mode=None, extra_files=None):
    """Runs `driver` (source text of a script) in a project directory that holds `m.py`, with the
    agent loaded; returns the JSON the script printed on its last line."""
    with tempfile.TemporaryDirectory() as tmp:
        project = os.path.realpath(os.path.join(tmp, "project"))
        os.makedirs(project)
        for name, text in {"m.py": MODULE, **(extra_files or {})}.items():
            path = os.path.join(project, name)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as fh:
                fh.write(text)
        script = os.path.join(project, "drive.py")
        with open(script, "w", encoding="utf-8") as fh:
            fh.write(driver)
        env = {k: v for k, v in os.environ.items() if not k.startswith("DLA_")}
        env.update(
            PYTHONPATH=HERE + os.pathsep + project,
            PYTHONDONTWRITEBYTECODE="1",
            DLA_HARNESS_OUT=os.path.join(tmp, "out.jsonl"),
            DLA_PROJECT=project,
        )
        if lines is not None:
            env["DLA_LINES"] = lines
        if mode is not None:
            env["DLA_LINES_MODE"] = mode
        done = subprocess.run(
            [sys.executable, script], cwd=project, env=env, capture_output=True, text=True, timeout=120
        )
        if done.returncode != 0:
            raise AssertionError(f"driver failed: {done.stderr}")
        return json.loads(done.stdout.strip().splitlines()[-1])


COUNT = textwrap.dedent(
    """\
    import json, sys
    import m
    {body}
    import sitecustomize as agent
    rec = agent._build("checkpoint", 1, True)
    print(json.dumps({{"counts": rec.get("line_counts"), "mode": agent._lines["mode"],
                      "limitations": rec["limitations"]}}))
    """
)


def counts_of(body, lines, mode=None):
    return run_agent(COUNT.format(body=body), lines, mode)


class LineCounts(unittest.TestCase):
    TARGETS = "m.py:5,m.py:6,m.py:7,m.py:11"

    def check_branch_counts(self, mode):
        got = counts_of(
            "m.add(1, True)\nm.add(2, False)\nm.add(3, False)",
            self.TARGETS,
            mode,
        )
        # The line of the branch ran once (taken once); the lines around it ran for every call.
        self.assertEqual(got["counts"]["m.py:6"], 1, got)
        self.assertEqual(got["counts"]["m.py:5"], 3, got)
        self.assertEqual(got["counts"]["m.py:7"], 3, got)
        # A line of another function that was never called is 0, not absent.
        self.assertEqual(got["counts"]["m.py:11"], 0, got)
        return got

    def test_a_target_line_on_a_branch_counts_only_when_the_branch_is_taken(self):
        got = self.check_branch_counts(None)
        self.assertEqual(got["mode"], "monitoring" if hasattr(sys, "monitoring") else "settrace")

    def test_a_branch_never_taken_counts_zero_while_its_function_ran(self):
        got = counts_of("for i in range(5):\n    m.add(i, False)", "m.py:6,m.py:7")
        self.assertEqual(got["counts"], {"m.py:6": 0, "m.py:7": 5}, got)

    def test_a_target_line_in_a_function_called_ten_times_counts_ten(self):
        got = counts_of("for i in range(10):\n    m.one()", "m.py:11")
        self.assertEqual(got["counts"], {"m.py:11": 10}, got)

    def test_the_forced_fallback_gives_the_same_counts(self):
        got = self.check_branch_counts("settrace")
        self.assertEqual(got["mode"], "settrace")
        ten = counts_of("for i in range(10):\n    m.one()", "m.py:11", "settrace")
        self.assertEqual(ten["counts"], {"m.py:11": 10}, ten)
        self.assertEqual(ten["mode"], "settrace")

    def test_a_harness_with_no_target_writes_no_counts_and_installs_no_line_events(self):
        probe = textwrap.dedent(
            """\
            import json, sys
            import m
            m.add(1, True)
            import sitecustomize as agent
            rec = agent._build("checkpoint", 1, True)
            used = []
            mon = getattr(sys, "monitoring", None)
            if mon is not None:
                used = [i for i in range(6) if mon.get_tool(i) == "dla-lines"]
            print(json.dumps({"has_key": "line_counts" in rec, "on": agent._lines["on"],
                              "trace": sys.gettrace() is not None, "tools": used}))
            """
        )
        for lines in (None, ""):
            got = run_agent(probe, lines)
            self.assertEqual(got, {"has_key": False, "on": False, "trace": False, "tools": []}, got)

    def test_a_line_of_another_file_or_of_the_agent_is_not_counted(self):
        got = counts_of("m.add(1, True)", "other.py:3,m.py:6")
        self.assertEqual(got["counts"], {"other.py:3": 0, "m.py:6": 1}, got)


class WhoseHolder(unittest.TestCase):
    """`_holder_of`: "project", "tests" or None for the module that holds a root."""

    FILES = {
        "reg.py": "X = []\n",
        "reg_test.py": "X = []\n",
        "test_helpers.py": "X = []\n",
        "pkg/__init__.py": "",
        "pkg/store.py": "X = []\n",
        "pkg/conftest.py": "X = []\n",
        "tests/__init__.py": "",
        "tests/shared.py": "X = []\n",
    }
    MODULES = ["reg", "reg_test", "test_helpers", "pkg.store", "pkg.conftest", "tests.shared"]

    def test_project_files_test_files_and_other_modules_are_told_apart(self):
        body = textwrap.dedent(
            """\
            import json, sys, importlib
            import sitecustomize as agent
            names = {names}
            for n in names:
                importlib.import_module(n)
            print(json.dumps({{n: agent._holder_of(n) for n in names + ["__main__", "unittest", "json"]}}))
            """
        ).format(names=json.dumps(self.MODULES))
        got = run_agent(body, None, None, extra_files=self.FILES)
        self.assertEqual(got["reg"], "project", got)
        self.assertEqual(got["pkg.store"], "project", got)
        for name in ("reg_test", "test_helpers", "pkg.conftest", "tests.shared", "__main__", "unittest"):
            self.assertEqual(got[name], "tests", got)
        # The standard library is neither the project's nor the tests'.
        self.assertIsNone(got["json"], got)


if __name__ == "__main__":
    unittest.main()
