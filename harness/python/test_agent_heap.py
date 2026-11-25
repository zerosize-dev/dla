"""Tests of the retention capture (`DLA_CAPTURE=heap`) on a large heap: the objects of the lines that
grew are found first, the scan has a share of the time bound of its own, the walks always happen, and
the record counts what was walked and what was not.

Run with the interpreter that runs the targets, from any directory:

    python3 -m unittest discover -s harness/python -p 'test_agent_*.py'

Each test starts a fresh interpreter with the agent first on PYTHONPATH, as `dla harness run` does.
The time bound is driven by a clock the test owns (every reading of `time.monotonic` is one
millisecond later than the one before), so what the scan reads before the bound stops it does not
depend on the speed of the machine.
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
    BIG = []
    GROW = {}


    def fill(n):
        for i in range(n):
            BIG.append([i])


    def grow(n):
        for i in range(n):
            GROW[i] = [i, i]  # GROWS


    def make(i):
        return [i]


    def copies(n):
        import copy
        out = []
        for i in range(n):
            out.append(copy.copy([i, i]))  # COPIES
        return out
    """
)
# Lines of MODULE: 12 is the line that grows (`GROW[i] = [i, i]`), 23 is `out.append(copy.copy(...))`.

DRIVER = textwrap.dedent(
    """\
    import gc, json, sys, time
    import big
    import sitecustomize as agent

    class Clock:
        now = 0.0

        @staticmethod
        def monotonic():
            Clock.now += {tick}
            return Clock.now

        def __getattr__(self, name):
            return getattr(time, name)

    big.fill({fill})
    gc.collect()
    first = agent._count_blocks()[0]
    big.grow({grow})
    gc.collect()
    counted = agent._count_blocks()[0]
    objs = gc.get_objects()
    {patch}
    agent.time = Clock()
    doc = agent._heap_build(objs, first, counted, [])
    agent.time = time
    print(json.dumps(doc))
    """
)


def run_heap(fill=200000, grow=3000, secs="4", tick=0.001, patch="pass", frames="1"):
    """Builds a heap with `fill` small objects kept by a module list and `grow` objects made at one
    line and kept by a module dict, and returns the document `_heap_build` writes."""
    with tempfile.TemporaryDirectory() as tmp:
        project = os.path.realpath(os.path.join(tmp, "project"))
        os.makedirs(project)
        with open(os.path.join(project, "big.py"), "w", encoding="utf-8") as fh:
            fh.write(MODULE)
        script = os.path.join(project, "drive.py")
        with open(script, "w", encoding="utf-8") as fh:
            fh.write(DRIVER.format(fill=fill, grow=grow, patch=patch, tick=tick))
        env = {k: v for k, v in os.environ.items() if not k.startswith("DLA_")}
        env.update(
            PYTHONPATH=HERE + os.pathsep + project,
            PYTHONDONTWRITEBYTECODE="1",
            DLA_HARNESS_OUT=os.path.join(tmp, "out.jsonl"),
            DLA_CAPTURE="heap",
            DLA_CAPTURE_DIR=os.path.join(tmp, "cap"),
            DLA_PROJECT=project,
            DLA_HEAP_SECS=secs,
            DLA_TRACEMALLOC_FRAMES=frames,
        )
        done = subprocess.run(
            [sys.executable, script], cwd=project, env=env, capture_output=True, text=True, timeout=300
        )
        if done.returncode != 0:
            raise AssertionError(f"driver failed: {done.stderr}")
        return json.loads(done.stdout.strip().splitlines()[-1])


def site(doc, line):
    rows = [s for s in doc["sites"] if s["site"] == f"big.py:{line}"]
    assert rows, [s["site"] for s in doc["sites"]]
    return rows[0]


class HeapWalkTests(unittest.TestCase):
    def test_a_heap_of_200000_objects_names_the_holder_of_the_growing_line_when_the_bound_stops_the_scan(self):
        # The clock reads one millisecond per look: the bound of 4 s lets the scan 2.4 s, of which
        # the first look at every object takes about 0.84 s, so the pass that reads the holders is
        # stopped well before it has read the 215,000 objects.
        doc = run_heap()
        walk = doc["walk"]
        self.assertLess(walk["scanned"], walk["of"], walk)
        self.assertTrue(any("stopped the scan after" in n for n in doc["limitations"]), doc["limitations"])
        grows = site(doc, 12)
        self.assertEqual(grows["found_tracked"], 3000)
        roots = [(c["count"], c["root"]["kind"], c["root"].get("key")) for c in grows["chains"]]
        self.assertEqual(roots, [(40, "module-global", "GROW")], grows["chains"])
        # Every kept object of the line was walked and the record counts them.
        self.assertEqual((grows["walked"], grows["not_walked"]), (grows["kept"], 0))
        self.assertEqual(walk["walked"] + walk["not_walked"], walk["kept"])
        self.assertEqual(walk["not_walked"], 0, walk)

    def test_a_scan_that_finishes_leaves_every_walk_to_the_holders_it_read(self):
        doc = run_heap(fill=2000, grow=200, secs="60")
        walk = doc["walk"]
        self.assertEqual(walk["scanned"], walk["of"] + 1 if walk["scanned"] > walk["of"] else walk["of"])
        self.assertFalse(any("stopped the scan" in n for n in doc["limitations"]), doc["limitations"])
        grows = site(doc, 12)
        self.assertEqual(grows["chains"][0]["root"].get("key"), "GROW")
        self.assertEqual(grows["not_walked"], 0)

    def test_objects_that_were_not_walked_are_counted_and_named_in_the_record(self):
        # A search that may visit one object ends at once: nothing is walked, and the record says
        # how many kept objects that left without a chain.
        doc = run_heap(fill=2000, grow=200, secs="60", patch="agent._HEAP_WALK_NODES = 1")
        grows = site(doc, 12)
        self.assertEqual(grows["chains"], [])
        self.assertEqual((grows["walked"], grows["not_walked"]), (0, grows["kept"]))
        self.assertGreater(grows["kept"], 0)
        walk = doc["walk"]
        self.assertGreaterEqual(walk["not_walked"], grows["kept"])
        self.assertEqual(walk["walked"] + walk["not_walked"], walk["kept"])
        self.assertEqual(walk["not_walked"], sum(s["not_walked"] for s in doc["sites"]))
        self.assertTrue(any("kept objects have no chain" in n for n in doc["limitations"]), doc["limitations"])

    def test_the_site_of_an_object_is_the_one_the_block_rule_gives_with_ten_frames(self):
        script = textwrap.dedent(
            """\
            import gc, json, tracemalloc
            import big
            import sitecustomize as agent
            keep = big.copies(50) + [big.make(i) for i in range(50)]
            find = agent._site_finder()
            rows = []
            for obj in keep:
                tb = tracemalloc.get_object_traceback(obj)
                rows.append([find(obj), None if tb is None else agent._block_site(tb)])
            print(json.dumps(rows))
            """
        )
        with tempfile.TemporaryDirectory() as tmp:
            project = os.path.realpath(os.path.join(tmp, "project"))
            os.makedirs(project)
            for name, text in (("big.py", MODULE), ("drive.py", script)):
                with open(os.path.join(project, name), "w", encoding="utf-8") as fh:
                    fh.write(text)
            env = {k: v for k, v in os.environ.items() if not k.startswith("DLA_")}
            env.update(
                PYTHONPATH=HERE + os.pathsep + project,
                PYTHONDONTWRITEBYTECODE="1",
                DLA_HARNESS_OUT=os.path.join(tmp, "out.jsonl"),
                DLA_CAPTURE="alloc",
                DLA_CAPTURE_DIR=os.path.join(tmp, "cap"),
                DLA_PROJECT=project,
                DLA_TRACEMALLOC_FRAMES="10",
            )
            done = subprocess.run(
                [sys.executable, os.path.join(project, "drive.py")],
                cwd=project,
                env=env,
                capture_output=True,
                text=True,
                timeout=120,
            )
        self.assertEqual(done.returncode, 0, done.stderr)
        rows = json.loads(done.stdout.strip().splitlines()[-1])
        self.assertTrue(all(a == b for a, b in rows), rows[:3])
        # The copies are made inside the standard library, called from line 23 of the project.
        self.assertEqual({a for a, _ in rows[:50]}, {"big.py:23"}, rows[:3])
        self.assertEqual({a for a, _ in rows[50:]} - {None}, {"big.py:16"}, rows[50:53])


if __name__ == "__main__":
    unittest.main()
