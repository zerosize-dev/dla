"""DLA pytest loop. The project's own tests, run again and again in one process, one checkpoint
record per round (x = runs of the tests completed). Nothing to fill: run it.

A leak that no static candidate names (a reference a C extension never releases) shows only as an
allocation line that grows under a broad workload, and the broadest workload a project has is its
own tests. One unit is one `pytest.main` of the selected test files; the agent (`sitecustomize`)
counts the live traced blocks per project line at every checkpoint (`py.blocks@<file:line>`).

Rounds: DLA_WARMUP warm-up rounds, then DLA_ROUNDS measured rounds of DLA_STEP units each (the
defaults are baked in at generation time; the step is never below 1). The time limit of the
process is DLA_TIME_LIMIT_SECS (`dla harness run` sets it from DLA_COMMAND_SECS, less the start-up and,
with --capture, the capture's own time): from the first unit the loop estimates the whole series and,
when it would not fit, lowers the step, then cuts the number of rounds (only with a capture, not below
CAPTURE_MIN_ROUNDS: its allocation profiles), then drops the slowest test files (a limitation says which).
The time of the checkpoints so far counts too: a checkpoint of a capture is not free.

A run that pytest ends early is not a unit. From the second run on, when the exit code of
`pytest.main` is neither 0 (all passed) nor 1 (some tests failed), or the run executed no test (an
error while collecting, an error in a plugin or a conftest.py), the run is not counted: a note says
which run, the exit code and the first error line, and the same run is made again, at most
MAX_UNIT_REPEATS times for one unit and MAX_LOOP_REPEATS times in the whole loop. After that the run
counts as a dead unit and the summary line and the limitations say how many units were cut. The
time the repeats took counts for the time fitting. The first run keeps its own rules (below).
`--continue-on-collection-errors` makes one test file that fails to import cost its own tests and
not the whole run; such a file is named once in a note.

Exit codes: 0 the series was written; 2 no test passed in any unit; 3 the loop cannot run (pytest
does not import, no test found, the tests import the project from outside it, one test file alone
does not fit the time limit). The reason is a line that starts with `DLA-PYTEST` on stderr.

Run from the scratch project root with the harness directory first on PYTHONPATH:

    PYTHONPATH=<harness dir>:. python <harness dir>/dla_pytest_loop.py
"""

import os
import sys
import tempfile
import time

import sitecustomize as dla  # the DLA agent; writes the checkpoint records

STEP = max(1, int(os.environ.get("DLA_STEP") or "@@DLA_STEP@@"))
WARMUP = int(os.environ.get("DLA_WARMUP") or "@@DLA_WARMUP@@")
ROUNDS = int(os.environ.get("DLA_ROUNDS") or "@@DLA_ROUNDS@@")
# The test files or directories to run, relative to the working directory.
TESTS = @@DLA_TESTS@@

# Time limit of this process (seconds; `dla harness run` sets DLA_TIME_LIMIT_SECS from the harness
# timeout, 0 or unset = no fitting). Margin: 15 percent and 5 seconds for the final checkpoint and
# the exit, as the generic driver and the JUnit loop keep.
LIMIT_SECS = float(os.environ.get("DLA_TIME_LIMIT_SECS") or "0")
_T0 = time.monotonic()

# A run with --capture takes its allocation profiles at the checkpoints and its retention record at
# the last: at least this many rounds must run for 3 profiles and the record. A run without a
# capture keeps WARMUP plus this many (the series is then short and says so).
CAPTURING = bool((os.environ.get("DLA_CAPTURE") or "").strip())
CAPTURE_MIN_ROUNDS = 3

# Bounds of what a unit may write to the harness log: the first unit, and the first later unit that
# has a failure.
LOG_FIRST_BYTES = 16384
LOG_LATER_BYTES = 4096
FAILURE_CHARS = 200

# Arguments that keep the run in this process and quiet. The project's own addopts may ask for
# parallel workers or a coverage plugin; the cache plugin writes into the project.
PYTEST_ARGS = [
    "-o",
    "addopts=",
    "-p",
    "no:cacheprovider",
    "-p",
    "no:xdist",
    "-q",
    "--tb=short",
    "--continue-on-collection-errors",
]

# A run that pytest ended early (see the module text) is made again: at most this many times for one
# unit, and this many in the whole loop.
MAX_UNIT_REPEATS = 2
MAX_LOOP_REPEATS = 4


class _Stop(Exception):
    """The loop cannot go on; `code` is the exit code and `text` the reason line."""

    def __init__(self, code, text):
        Exception.__init__(self, text)
        self.code = code
        self.text = text


def _say(text):
    sys.stderr.write("DLA-PYTEST " + text + "\n")
    sys.stderr.flush()


def _limit(text, n=FAILURE_CHARS):
    text = " ".join(str(text).split())
    return text if len(text) <= n else text[: n - 3] + "..."


class _Tally:
    """The pytest plugin: per test file, the tests that passed, failed and were skipped, the time
    they took, and the first failure."""

    def __init__(self):
        self.files = {}  # path -> [passed, failed, skipped, seconds]
        self.first = {}  # path -> first failure, one line
        self.ran = 0  # test reports seen: a run that executed no test has none
        self.collect_failed = []  # test files whose collection failed, in the order they did

    def _slot(self, nodeid):
        path = (nodeid or "").split("::", 1)[0].replace("\\", "/") or "<session>"
        slot = self.files.get(path)
        if slot is None:
            slot = self.files[path] = [0, 0, 0, 0.0]
        return path, slot

    def _failure(self, path, report):
        if path in self.first:
            return
        text = None
        try:
            crash = getattr(report.longrepr, "reprcrash", None)
            text = getattr(crash, "message", None)
            if not text:
                text = str(report.longrepr).strip().splitlines()[-1]
        except BaseException:
            text = "failed"
        where = getattr(report, "nodeid", "") or path
        self.first[path] = _limit(f"{where}: {text}")

    def pytest_runtest_logreport(self, report):
        self.ran += 1
        path, slot = self._slot(report.nodeid)
        slot[3] += getattr(report, "duration", 0.0) or 0.0
        outcome = report.outcome
        if report.when == "call":
            slot[{"passed": 0, "failed": 1, "skipped": 2}.get(outcome, 1)] += 1
        elif outcome == "failed":  # an error in a fixture, at setup or at teardown
            slot[1] += 1
        elif report.when == "setup" and outcome == "skipped":
            slot[2] += 1
        if outcome == "failed":
            self._failure(path, report)

    def pytest_collectreport(self, report):
        if getattr(report, "failed", False):
            path, slot = self._slot(report.nodeid)
            slot[1] += 1
            self._failure(path, report)
            if path not in self.collect_failed:
                self.collect_failed.append(path)

    def totals(self):
        passed = sum(s[0] for s in self.files.values())
        failed = sum(s[1] for s in self.files.values())
        skipped = sum(s[2] for s in self.files.values())
        return passed, failed, skipped

    def first_failure(self):
        for path in sorted(self.first):
            return self.first[path]
        return None


def _error_line(tally, head):
    """The error line of a run: the first failure the plugin saw, else the last line of the output
    that names an error (the end of a traceback)."""
    line = tally.first_failure()
    if line:
        return line
    for raw in reversed(head.splitlines()):
        if "rror" in raw or "INTERNALERROR" in raw:
            return _limit(raw.strip())
    return "no error line in the output"


def _holds_python(directory):
    try:
        for name in os.listdir(directory):
            full = os.path.join(directory, name)
            if name.endswith(".py") or os.path.isfile(os.path.join(full, "__init__.py")):
                return True
    except OSError:
        pass
    return False


def _set_up_paths():
    """The working directory first, and `<cwd>/src` when that directory holds a package or a module
    (a src layout). Returns the directories that hold the project's top-level names."""
    cwd = os.getcwd()
    roots = [cwd]
    src = os.path.join(cwd, "src")
    if os.path.isdir(src) and _holds_python(src):
        roots.append(src)
    for path in reversed(roots):
        while path in sys.path:
            sys.path.remove(path)
        sys.path.insert(0, path)
    return roots


def _project_names(roots):
    """The top-level packages and modules the project defines under `roots`."""
    names = []
    for root in roots:
        try:
            entries = sorted(os.listdir(root))
        except OSError:
            continue
        for entry in entries:
            full = os.path.join(root, entry)
            if os.path.isdir(full) and os.path.isfile(os.path.join(full, "__init__.py")):
                name = entry
            elif entry.endswith(".py") and entry not in ("setup.py", "conftest.py"):
                name = entry[:-3]
            else:
                continue
            if name.isidentifier() and name not in names:
                names.append(name)
    return names


def _inside(path, roots):
    real = os.path.realpath(path)
    return any(real == root or real.startswith(root + os.sep) for root in roots)


def _check_origin(roots):
    """After the first unit: every top-level package or module the project defines that the tests
    imported must come from the scratch copy or from the project it was made from. Otherwise the
    loop would measure another copy (an installed one)."""
    own = [os.path.realpath(os.environ.get("DLA_PROJECT") or os.getcwd())]
    source = os.environ.get("DLA_SOURCE_ROOT")
    if source:
        own.append(os.path.realpath(source))
    for name in _project_names(roots):
        mod = sys.modules.get(name)
        if mod is None:
            continue
        where = getattr(mod, "__file__", None)
        if where is None:
            paths = list(getattr(mod, "__path__", None) or [])
            where = paths[0] if paths else None
        if where and not _inside(where, own):
            raise _Stop(
                3,
                f"the tests import {name} from {where}, which is outside the project: the loop would "
                "measure another copy. Build or install the project in place (for example "
                "python3 -m pip install -e . , or python3 setup.py build_ext --inplace for extension "
                f"modules) so that {name} imports from the project, then generate the harness again "
                "(the scratch copy is made when the harness is generated, and it keeps the extension "
                "modules built in place at that time)",
            )


def _run_pytest(files, tally):
    """One pytest run of `files` in this process. Returns `(exit code, seconds, output head,
    output size)`; what pytest and the tests write goes to a temporary file, not to the log."""
    import pytest

    args = PYTEST_ARGS + ["--rootdir", os.getcwd()] + list(files)
    sys.stdout.flush()
    sys.stderr.flush()
    saved = (os.dup(1), os.dup(2))
    code = 3
    t0 = time.monotonic()
    with tempfile.TemporaryFile() as tmp:
        os.dup2(tmp.fileno(), 1)
        os.dup2(tmp.fileno(), 2)
        try:
            try:
                code = int(pytest.main(args, plugins=[tally]))
            except SystemExit as exc:
                code = exc.code if isinstance(exc.code, int) else 1
            except Exception as exc:  # pytest itself failed: not a result of the tests
                sys.stderr.write(f"pytest.main raised {exc!r}\n")
        finally:
            try:
                sys.stdout.flush()
                sys.stderr.flush()
            except BaseException:
                pass
            os.dup2(saved[0], 1)
            os.dup2(saved[1], 2)
            os.close(saved[0])
            os.close(saved[1])
        took = time.monotonic() - t0
        size = tmp.seek(0, 2)
        tmp.seek(0)
        head = tmp.read(LOG_FIRST_BYTES).decode("utf-8", "replace")
    return code, took, head, size


def _record_files(active, dropped):
    """Writes the test files the loop runs (and those it dropped) next to the dump, so the bundle
    of a finding can list them. Never raises."""
    out = os.environ.get("DLA_HARNESS_OUT")
    if not out:
        return
    try:
        import json

        with open(os.path.join(os.path.dirname(out), "pytest-files.json"), "w", encoding="utf-8") as fh:
            json.dump({"files": list(active), "dropped": list(dropped)}, fh)
    except BaseException:
        pass


def _show(unit, head, size, limit):
    text = head[:limit]
    cut = f" (first {len(text)} of {size} bytes)" if size > len(text) else ""
    sys.stderr.write(f"DLA-PYTEST output of unit {unit}{cut}\n{text}\n")
    sys.stderr.flush()


def main(counts):
    dla.mark_phase("the pytest loop: the project's own tests (start-up and imports)")
    try:
        import pytest  # noqa: F401
    except BaseException as exc:
        raise _Stop(3, f"pytest does not import in {sys.executable}: {exc!r}")
    roots = _set_up_paths()
    total = WARMUP + ROUNDS
    active = list(TESTS)  # after the first unit: the test files that had results
    step = STEP
    done = 0
    dropped = []
    first_failure_noted = False
    shown_later = False
    state = {"first": 0.0, "since_secs": 0.0, "since_units": 0, "estimate": 0.0, "ckpt_secs": 0.0, "ckpts": 0}
    file_secs = {}
    noted = set()
    collect_noted = []  # test files named in a note as failing to collect
    cut_runs = []  # the numbers of the runs that ended as a dead unit although pytest ended them early

    def per_unit():
        if state["since_units"] > 0:
            return state["since_secs"] / state["since_units"]
        return state["estimate"] or state["first"]

    def note_once(text):
        if text not in noted:
            noted.add(text)
            dla.note(text)
            _say(text)

    def checkpoint_secs():
        """What one checkpoint takes (the mean of those written so far): a capture's profiles are
        taken there."""
        return state["ckpt_secs"] / state["ckpts"] if state["ckpts"] else 0.0

    def cut_rounds(round_no, in_round, per, ckpt, budget, floor):
        """Cuts `total` to the rounds that fit (not below `floor`) and tells the agent, which reads
        DLA_ROUNDS to place its allocation profiles. Returns True when the rounds were cut."""
        nonlocal total
        # Every round after this one costs its runs and its checkpoint.
        by_time = int(max(0.0, budget - per * (step - in_round)) // (per * step + ckpt))
        fits = max(floor, round_no + by_time)
        if fits >= total:
            return False
        was = total
        total = fits
        os.environ["DLA_ROUNDS"] = str(max(0, total - WARMUP))
        why = (
            f"the test loop cut its rounds from {was} to {total}: one run takes {per:.1f} s and the "
            f"{LIMIT_SECS:g} s time limit would cut the process"
            + (" (the run is for the capture: its profiles and the retention record need 3 rounds)" if CAPTURING else "")
        )
        dla.note(why)
        _say(why)
        return True

    def fit(round_no, in_round):
        """Lowers `step` (never below 1), cuts the rounds (a run with a capture first, not below
        the rounds its profiles need), then drops the slowest test files, when the units left
        would not fit the time limit. Returns True when files were dropped."""
        nonlocal step
        if not LIMIT_SECS > 0 or done == 0:
            return False
        left = total - round_no
        per = max(per_unit(), 1e-6)
        ckpt = checkpoint_secs()
        budget = LIMIT_SECS * 0.85 - 5 - (time.monotonic() - _T0)
        runs = per * (step - in_round + step * left)
        need = runs + ckpt * left
        if need <= budget:
            return False
        units = int(budget / per) if budget > 0 else 0
        lowered = max(1, min(step, (units + in_round) // (left + 1)))
        if lowered < step:
            dla.note(
                f"the test loop lowered its step from {step} to {lowered} runs of the tests per round in "
                f"round {round_no}: one run takes {per:.1f} s and the {LIMIT_SECS:g} s time limit would cut "
                "the process (the x axis is still the cumulative run count)"
            )
            step = lowered
        runs = per * (step - in_round + step * left)
        changed = False
        if CAPTURING and runs + ckpt * left > budget and total > max(round_no, CAPTURE_MIN_ROUNDS):
            # A run for the capture needs its profiles, not a long series: the rounds go first.
            if cut_rounds(round_no, in_round, per, ckpt, budget, max(round_no, CAPTURE_MIN_ROUNDS)):
                left = total - round_no
                runs = per * (step - in_round + step * left)
        if runs + ckpt * left > budget and len(active) > 1:
            every = sum(file_secs.get(f, 0.0) for f in active)
            share = 1.0
            while runs + ckpt * left > budget and len(active) > 1 and every > 0:
                slowest = max(active, key=lambda f: (file_secs.get(f, 0.0), f))
                secs = file_secs.get(slowest, 0.0)
                active.remove(slowest)
                runs = runs * (1.0 - secs / every)
                share *= 1.0 - secs / every
                every -= secs
                why = (
                    f"dropped test file {slowest}: it took {secs:.1f} s of {per:.1f} s per run and the series "
                    f"would not fit {LIMIT_SECS:g} s (left out from run {done + 1} on)"
                )
                dla.note(why)
                _say(why)
                dropped.append(slowest)
                changed = True
            if changed:
                _record_files(active, dropped)
                # The next reading is of the files that are left.
                state["estimate"] = max(1e-6, per * share)
                state["since_secs"] = 0.0
                state["since_units"] = 0
        need = runs + ckpt * left
        if need > budget and not CAPTURING:
            # One file alone does not fit. A first reading holds the one-time costs (the imports of
            # the project): judge it only when a second run is out of the question or a steady
            # reading exists.
            if state["since_units"] >= 1 or per > budget:
                raise _Stop(
                    3,
                    f"the tests are too slow for a series: even {step} run per round of {len(active)} "
                    f"test file(s) needs {need:.0f} s for the runs left and {max(0.0, budget):.0f} s "
                    "remain; choose fewer or faster test files (--tests) or raise timeout_secs in harness.json",
                )
        elif need > budget and state["since_units"] >= 1:
            note_once(
                f"the time limit is short for a capture: {total} rounds of {len(active)} test file(s) need "
                f"{need:.0f} s and {max(0.0, budget):.0f} s remain; the profiles and the retention record are "
                "taken with the rounds that are left (the process limit has the margin)"
            )
        return changed

    round_no = 0
    while round_no < total:
        round_no += 1
        dla.mark_phase(f"pytest unit (round {round_no} of {total}, runs {done + 1} to {done + step})")
        n = 0
        while n < step:
            n += 1
            done += 1
            repeated = 0
            while True:
                tally = _Tally()
                code, took, head, size = _run_pytest(active, tally)
                cut = done > 1 and (code not in (0, 1) or tally.ran == 0)
                if not cut:
                    break
                counts["cut_runs"] += 1
                why = (
                    f"run {done} of the tests was ended early by pytest (exit code {code}, "
                    f"{tally.ran} test report(s); first error: {_error_line(tally, head)})"
                    + ("; the tests that ran before the cut are in the round's figures" if tally.ran else "")
                )
                # What the cut run took is the time of getting this unit.
                state["since_secs"] += took
                if repeated >= MAX_UNIT_REPEATS or counts["repeats"] >= MAX_LOOP_REPEATS:
                    counts["cut_units"] += 1
                    cut_runs.append(done)
                    text = f"{why}; it was made again {repeated} time(s) and counts as a dead unit"
                    dla.note(text)
                    _say(text)
                    break
                repeated += 1
                counts["repeats"] += 1
                text = f"{why}: it is not a unit and is made again ({repeated} of at most {MAX_UNIT_REPEATS} for this unit, {counts['repeats']} of at most {MAX_LOOP_REPEATS} in the loop)"
                dla.note(text)
                _say(text)
            counts["units"] = done
            passed, failed, skipped = tally.totals()
            if done == 1:
                state["first"] = took
                file_secs.update({f: s[3] for f, s in tally.files.items()})
                _show(1, head, size, LOG_FIRST_BYTES)
                if passed + failed + skipped == 0:
                    raise _Stop(
                        3,
                        f"no test found in {', '.join(TESTS)} (pytest exit code {code}): check the paths "
                        "(--tests) and that the files hold pytest or unittest tests",
                    )
                if passed + failed == 0:
                    raise _Stop(
                        3,
                        f"no test passed or failed: {skipped} test(s) were skipped, so a service or a "
                        "setting the tests need may be missing",
                    )
                if passed == 0:
                    counts["first_failure"] = tally.first_failure()
                    raise _Stop(2, f"no test passed in any test file: {tally.first_failure()}")
                _check_origin(roots)
                active[:] = sorted(f for f in tally.files if f != "<session>")
                broken = [f for f in active if tally.files[f][0] == 0 and tally.files[f][1] > 0]
                gone = 0.0
                for f in broken:
                    if f in tally.collect_failed:
                        why = (
                            f"dropped test file {f}: it failed to collect in the first run (first failure: "
                            f"{tally.first.get(f, 'failed')}), so it costs its own tests only and is left out "
                            "of the loop; the other files are measured"
                        )
                        collect_noted.append(f)
                    else:
                        why = (
                            f"dropped test file {f}: none of its tests passed in the first run (first failure: "
                            f"{tally.first.get(f, 'failed')}), so it is left out of the loop and the other files "
                            "are measured"
                        )
                    dla.note(why)
                    _say(why)
                    dropped.append(f)
                    active.remove(f)
                    gone += file_secs.get(f, 0.0)
                if broken:
                    state["estimate"] = max(1e-6, took - gone)
                _record_files(active, dropped)
            else:
                state["since_secs"] += took
                state["since_units"] += 1
            counts["tests_run"] += passed + failed
            counts["skipped"] += skipped
            if failed > 0 and done > 1:
                counts["failed_units"] += 1
                if not first_failure_noted:
                    first_failure_noted = True
                    counts["first_failure"] = counts["first_failure"] or tally.first_failure()
                    dla.note(
                        f"run {done} of the tests had {failed} failure(s) (first: {tally.first_failure()}): a "
                        "failing test may stop before the code it exercises, so the tests that passed are the "
                        "measurement; the log counts the runs with a failure (failed_units)"
                    )
                    if not shown_later:
                        shown_later = True
                        _show(done, head, size, LOG_LATER_BYTES)
            elif failed > 0:
                counts["failed_units"] += 1
                counts["first_failure"] = counts["first_failure"] or tally.first_failure()
            for f in tally.collect_failed:
                if f not in collect_noted:
                    collect_noted.append(f)
                    dla.note(
                        f"test file {f} failed to collect in run {done} (first failure: "
                        f"{tally.first.get(f, 'failed')}): it costs its own tests only, the other files ran"
                    )
            cut_now = bool(cut_runs) and cut_runs[-1] == done
            if passed == 0 or cut_now:
                counts["dead_units"] += 1
            if done <= 2:
                fit(round_no, n)
        if round_no == total and cut_runs:
            dla.note(
                f"{len(cut_runs)} unit(s) of the loop were cut by pytest and counted as dead "
                f"(run {', '.join(str(r) for r in cut_runs)}) after {counts['repeats']} repeat(s) in the loop"
            )
        t_ckpt = time.monotonic()
        dla.checkpoint(done, last=round_no == total)
        state["ckpt_secs"] += time.monotonic() - t_ckpt
        state["ckpts"] += 1
        if round_no < total:
            fit(round_no + 1, 0)
    counts["files"] = len(active)
    counts["dropped"] = len(dropped)
    return 2 if counts["dead_units"] == done else 0


def _summary(counts):
    failure = counts["first_failure"]
    sys.stderr.write(
        "DLA-PYTEST summary: units=%d tests_run=%d files=%d dropped=%d failed_units=%d cut_units=%d repeats=%d%s\n"
        % (
            counts["units"],
            counts["tests_run"],
            counts["files"],
            counts["dropped"],
            counts["failed_units"],
            counts["cut_units"],
            counts["repeats"],
            f" first_failure={failure}" if failure else "",
        )
    )
    sys.stderr.flush()


if __name__ == "__main__":
    # The process ends when the loop is done and the last checkpoint is written (os._exit skips the
    # atexit handlers; the agent's own exit record is not wanted after a driver's last checkpoint).
    code = 0
    counts = {
        "units": 0,
        "tests_run": 0,
        "skipped": 0,
        "files": len(TESTS),
        "dropped": 0,
        "failed_units": 0,
        "dead_units": 0,
        "cut_units": 0,
        "cut_runs": 0,
        "repeats": 0,
        "first_failure": None,
    }
    try:
        code = main(counts)
        if code == 2:
            _say(f"every run of the tests had no passing test: {counts['first_failure']}")
    except _Stop as stop:
        _say(stop.text)
        code = stop.code
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else 1
    except BaseException:
        import traceback

        traceback.print_exc()
        code = 1
    try:
        _summary(counts)
        sys.stdout.flush()
        sys.stderr.flush()
    finally:
        os._exit(code)
