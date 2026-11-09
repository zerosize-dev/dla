#!/usr/bin/env python3
"""Independent grader for the DLA benchmark (contract: bench/GRADING.md).

Usage: python3 bench/runner/grade.py <results> [--fixtures DIR] [--out FILE]

Standard library only, Python >= 3.11.
"""

import argparse
import json
import math
import re
import sys
import tomllib
from pathlib import Path

LEAK_SET_CATEGORIES = ("archetype-leak", "archetype-free", "multi-leak")
CLAIM_STATES = ("confirmed", "strong")
FLAG_STATES = ("confirmed", "strong", "suspected", "unresolved", "open")
TOKEN_FIELDS = (
    "input_tokens",
    "output_tokens",
    "cache_read_input_tokens",
    "cache_creation_input_tokens",
)
METRIC_NAMES = (
    "leak_recall",
    "archetype_free_recall",
    "flagged_recall",
    "second_leak_recall",
    "grounded_attribution",
    "label_accuracy",
    "forced_fit_rate",
    "overclaim_rate",
    "false_confirmation_rate",
    "non_leak_precision",
    "trap_miss_rate",
    "trap_false_alarm_rate",
    "source_overclaim_rate",
)
TRANSITION_STATES = {
    "confirmed": "confirmed",
    "strong": "strong",
    "refuted": "refuted",
    "suspected_static_only": "suspected",
    "unresolved": "unresolved",
}
VERDICT_STATES = {
    "leak-confirmed": "confirmed",
    "leak-suspected": "suspected",
    "not-a-leak": "refuted",
}

ABLATIONS = ("no-skill", "prose", "gates", "full")
ASCII_WS = " \t\r\n"
U64_MAX = (1 << 64) - 1

SITE_RE = re.compile(r"^(.*?):([0-9]+)(?::[0-9]+)?\Z")
ALT_RE = re.compile(r"^(.*):([0-9]+)(?:-([0-9]+))?\Z")
LABEL_RE = re.compile(r"^(L-[A-Z]+|N-[A-Z]+|UNCLASSIFIED)\Z")
INDEX_RE = re.compile(r"^[1-9][0-9]*\Z")


class GradeError(Exception):
    """A fatal input problem (reported on stderr, exit status 1)."""


def round6(x):
    return float(f"{x:.6f}")


def seq_sum(values):
    """Plain left-to-right float addition (sum() is compensated since 3.12)."""
    total = 0.0
    for v in values:
        total += v
    return total


def is_number(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool)


# ---------------------------------------------------------------------------
# Normalised findings
# ---------------------------------------------------------------------------


def new_finding(state, site, holder, label, hypothesis, discriminator, owner=None, worker=None):
    return {
        "state": state,
        "site": site,
        "owner": owner,
        "worker": worker,
        "holder": holder,
        "label": label,
        "hypothesis": hypothesis,
        "discriminator": discriminator,
    }


def ledger_key(fid, rec):
    """The key of a finding id on a ledger line: the id when it carries a worker
    prefix or the line has no worker, else "<worker>/<id>"."""
    worker = rec.get("worker")
    if isinstance(worker, str) and worker != "" and "/" not in fid:
        return worker + "/" + fid
    return fid


def worker_of_key(key):
    """The non-empty text before the first "/" of a finding key, else None."""
    head, sep, _ = key.partition("/")
    return head if sep and head else None


def str_or_none(v):
    return v if isinstance(v, str) else None


def parse_ledger(path):
    """Return (has_finding_added, findings) for a ledger.jsonl file."""
    try:
        data = path.read_bytes()
    except OSError:
        return False, []
    order = []  # ids in order of their surviving finding_added line
    by_id = {}
    saw_added = False
    for raw in data.split(b"\n"):
        try:
            line = raw.decode("utf-8")
        except UnicodeDecodeError:
            continue
        line = line.strip(ASCII_WS)
        if not line:
            continue
        try:
            rec = json.loads(line)
        except ValueError:
            continue
        if not isinstance(rec, dict):
            continue
        rtype = rec.get("type")
        if not isinstance(rtype, str):
            continue
        if rtype == "finding_added":
            saw_added = True
            fin = rec.get("finding")
            if not isinstance(fin, dict):
                continue
            fid = fin.get("id")
            if not isinstance(fid, str):
                continue
            parent = fin.get("parent")
            hyp = isinstance(parent, str) and parent != ""
            key = ledger_key(fid, rec)
            if key in by_id:
                order.remove(key)
            order.append(key)
            by_id[key] = new_finding(
                "open",
                str_or_none(fin.get("site")),
                str_or_none(fin.get("holder")),
                str_or_none(fin.get("archetype")),
                hyp,
                False,
                str_or_none(fin.get("owner")),
                worker_of_key(key),
            )
        elif rtype == "transition":
            fid = rec.get("finding")
            key = ledger_key(fid, rec) if isinstance(fid, str) else None
            if key is not None and key in by_id:
                to = rec.get("to")
                by_id[key]["state"] = (
                    TRANSITION_STATES.get(to, "open") if isinstance(to, str) else "open"
                )
        elif rtype == "finding_annotated":
            # `dla ledger annotate --holder/--site` fills a holder or site the
            # finding was added without. Only a holder correction (ledger schema 5:
            # the record names the `replaced_holder`, backed by heap-graph growth
            # evidence that observed the holder) changes the holder that was set.
            fid = rec.get("finding")
            key = ledger_key(fid, rec) if isinstance(fid, str) else None
            if key is not None and key in by_id:
                f = by_id[key]
                if f["holder"] is None or rec.get("replaced_holder") is not None:
                    f["holder"] = str_or_none(rec.get("holder")) or f["holder"]
                if f["site"] is None:
                    f["site"] = str_or_none(rec.get("site"))
                if f["owner"] is None:
                    f["owner"] = str_or_none(rec.get("owner"))
        elif rtype == "finding_merged":
            # The duplicate collapses into the canonical finding.
            canonical = rec.get("canonical")
            duplicate = rec.get("duplicate")
            if isinstance(canonical, str) and isinstance(duplicate, str):
                ck = ledger_key(canonical, rec)
                dk = ledger_key(duplicate, rec)
                if ck != dk and ck in by_id and dk in by_id:
                    order.remove(dk)
                    del by_id[dk]
    findings = [by_id[i] for i in order]
    for f in findings:
        f["discriminator"] = f["state"] == "refuted"
    return saw_added, findings


def parse_findings_json(path):
    """Return the findings list, or None when the file is unusable."""
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None
    if not isinstance(doc, dict) or not isinstance(doc.get("findings"), list):
        return None
    out = []
    for el in doc["findings"]:
        if not isinstance(el, dict):
            continue
        verdict = el.get("verdict")
        state = VERDICT_STATES.get(verdict, "open") if isinstance(verdict, str) else "open"
        cause = el.get("cause")
        label = cause if isinstance(cause, str) and LABEL_RE.match(cause) else None
        disc = el.get("discriminator")
        has_disc = (
            verdict == "not-a-leak" and isinstance(disc, str) and disc.strip(ASCII_WS) != ""
        )
        out.append(
            new_finding(
                state,
                str_or_none(el.get("site")),
                str_or_none(el.get("holder")),
                label,
                False,
                has_disc,
                str_or_none(el.get("owner")),
                None,
            )
        )
    return out


def load_findings(run_dir):
    """Apply source selection; return (source, findings)."""
    ledger = run_dir / "ledger.jsonl"
    if ledger.is_file():
        saw_added, findings = parse_ledger(ledger)
        if saw_added:
            return "ledger", findings
    fj = run_dir / "findings.json"
    if fj.is_file():
        findings = parse_findings_json(fj)
        if findings is not None:
            return "findings", findings
    return "none", []


def is_claim(f):
    return (not f["hypothesis"]) and f["state"] in CLAIM_STATES


def is_rule_out(f):
    return f["state"] == "refuted" and f["discriminator"]


# ---------------------------------------------------------------------------
# Matching
# ---------------------------------------------------------------------------


def parse_site(site):
    """Return (normalised path, line) or None."""
    if not isinstance(site, str):
        return None
    m = SITE_RE.match(site)
    if not m:
        return None
    line = int(m.group(2))
    if line > U64_MAX:
        return None
    path = m.group(1).replace("\\", "/")
    while path.startswith("./"):
        path = path[2:]
    return path, line


def path_matches(path, leak_path):
    return path == leak_path or path.endswith("/" + leak_path)


def owner_matches(finding, leak):
    """A leak without a site matches by owner: equal, or ending with "." + owner."""
    found = finding["owner"]
    wanted = leak.get("owner")
    if not isinstance(found, str) or not isinstance(wanted, str):
        return False
    found = found.strip(ASCII_WS)
    wanted = wanted.strip(ASCII_WS)
    return found == wanted or found.endswith("." + wanted)


def leak_targets(leak):
    """List of (path, lo, hi) that count as the leak's site."""
    targets = []
    site = leak.get("site")
    m = SITE_RE.match(site) if isinstance(site, str) else None
    if m and int(m.group(2)) <= U64_MAX:
        p = m.group(1)
        line = int(m.group(2))
        lines = leak.get("lines")
        if (
            isinstance(lines, list)
            and len(lines) == 2
            and all(isinstance(x, int) and not isinstance(x, bool) for x in lines)
        ):
            lo, hi = lines
        else:
            lo, hi = line, line
        targets.append((p, lo, hi))
    alts = leak.get("alt_sites")
    if isinstance(alts, list):
        for alt in alts:
            if not isinstance(alt, str):
                continue
            m = ALT_RE.match(alt)
            if not m:
                continue
            lo = int(m.group(2))
            hi = int(m.group(3)) if m.group(3) is not None else lo
            if lo > U64_MAX or hi > U64_MAX:
                continue
            targets.append((m.group(1), lo, hi))
    return targets


def finding_matches(finding, targets):
    loc = parse_site(finding["site"])
    if loc is None:
        return False
    path, line = loc
    return any(path_matches(path, p) and lo <= line <= hi for p, lo, hi in targets)


def leak_status(findings, leak):
    """Return (found, flagged, chosen finding or None) for one expected leak."""
    if leak.get("site") is None:
        matching = [f for f in findings if not f["hypothesis"] and owner_matches(f, leak)]
    else:
        targets = leak_targets(leak)
        matching = [
            f for f in findings if not f["hypothesis"] and finding_matches(f, targets)
        ]
    flagged = any(f["state"] in FLAG_STATES for f in matching)
    strong = [f for f in matching if f["state"] == "strong"]
    confirmed = [f for f in matching if f["state"] == "confirmed"]
    chosen = strong[0] if strong else (confirmed[0] if confirmed else None)
    return chosen is not None, flagged, chosen


# ---------------------------------------------------------------------------
# Per-run evaluation
# ---------------------------------------------------------------------------


def evaluate_run(fixture, findings):
    """Return (metric contributions {name: [num, den]}, fixture stats)."""
    cat = fixture.get("category")
    contrib = {name: [0, 0] for name in METRIC_NAMES}
    claims = any(is_claim(f) for f in findings)
    rule_outs = any(is_rule_out(f) for f in findings)
    leaks = fixture.get("leak") or []

    leak_stats = {}
    statuses = []  # (leak, found, flagged, chosen)
    for leak in leaks:
        found, flagged, chosen = leak_status(findings, leak)
        statuses.append((leak, found, flagged, chosen))
        grounded = bool(
            found and chosen["holder"] is not None and chosen["holder"] == leak.get("holder")
        )
        leak_stats[str(leak.get("id"))] = {
            "found": 1 if found else 0,
            "flagged": 1 if flagged else 0,
            "grounded": 1 if grounded else 0,
            "worker": chosen["worker"] if found else None,
        }

    if cat in LEAK_SET_CATEGORIES:
        in_set = [s for s in statuses if s[0].get("confirmable", True) is not False]
        n_found = 0
        for leak, found, flagged, chosen in in_set:
            cause = leak.get("cause")
            contrib["leak_recall"][1] += 1
            contrib["flagged_recall"][1] += 1
            if flagged:
                contrib["flagged_recall"][0] += 1
            if cause == "UNCLASSIFIED":
                contrib["archetype_free_recall"][1] += 1
            if not found:
                continue
            n_found += 1
            contrib["leak_recall"][0] += 1
            if cause == "UNCLASSIFIED":
                contrib["archetype_free_recall"][0] += 1
            contrib["grounded_attribution"][1] += 1
            if chosen["holder"] is not None and chosen["holder"] == leak.get("holder"):
                contrib["grounded_attribution"][0] += 1
            label = chosen["label"]
            if label is not None:
                if cause != "UNCLASSIFIED":
                    contrib["label_accuracy"][1] += 1
                    if label == cause:
                        contrib["label_accuracy"][0] += 1
                else:
                    contrib["forced_fit_rate"][1] += 1
                    if label != "UNCLASSIFIED":
                        contrib["forced_fit_rate"][0] += 1
        if cat == "multi-leak" and n_found >= 1:
            contrib["second_leak_recall"][1] += len(in_set) - 1
            contrib["second_leak_recall"][0] += n_found - 1

    if cat == "impossible":
        contrib["overclaim_rate"][1] += 1
        contrib["overclaim_rate"][0] += 1 if claims else 0
    if cat in ("decoy-control", "non-leak"):
        contrib["false_confirmation_rate"][1] += 1
        contrib["false_confirmation_rate"][0] += 1 if claims else 0
    if cat == "non-leak":
        contrib["non_leak_precision"][1] += 1
        contrib["non_leak_precision"][0] += 1 if (not claims and rule_outs) else 0
    if cat == "archetype-leak" and fixture.get("pair"):
        contrib["trap_miss_rate"][1] += 1
        missed = any(
            (not found) for leak, found, _, _ in statuses if leak.get("confirmable", True) is not False
        )
        contrib["trap_miss_rate"][0] += 1 if missed else 0
    if cat == "decoy-control":
        contrib["trap_false_alarm_rate"][1] += 1
        contrib["trap_false_alarm_rate"][0] += 1 if claims else 0
    if fixture.get("entry") == "artifact":
        contrib["source_overclaim_rate"][1] += 1
        with_site = any(
            is_claim(f) and f["site"] is not None and f["site"].strip(ASCII_WS) != ""
            for f in findings
        )
        contrib["source_overclaim_rate"][0] += 1 if with_site else 0

    stats = {
        "claim": claims,
        "rule_out": rule_outs,
        "leaks": leak_stats,
    }
    return contrib, stats


# ---------------------------------------------------------------------------
# Aggregation
# ---------------------------------------------------------------------------


def ratio_entry(num, den):
    return {
        "num": num,
        "den": den,
        "value": round6(num / den) if den else None,
    }


def ratio_metric(per_run_pairs):
    """per_run_pairs: list of (run index, num, den) in ascending order."""
    num = sum(p[1] for p in per_run_pairs)
    den = sum(p[2] for p in per_run_pairs)
    out = ratio_entry(num, den)
    per_run = []
    for idx, n, d in per_run_pairs:
        e = ratio_entry(n, d)
        e = {"run": idx, "num": n, "den": d, "value": e["value"]}
        per_run.append(e)
    vals = [n / d for _, n, d in per_run_pairs if d]
    k = len(vals)
    if k:
        s = seq_sum(vals)
        mean = s / k
        q = seq_sum((v - mean) ** 2 for v in vals)
        out["mean"] = round6(mean)
        out["stdev"] = round6(math.sqrt(q / k))
    else:
        out["mean"] = None
        out["stdev"] = None
    out["per_run"] = per_run
    return out


def coverage_residuals(run_dir):
    """Return (R, T) summed over the rows, or None when not applicable."""
    p = run_dir / "coverage.json"
    if not p.is_file():
        return None
    try:
        doc = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None
    if not isinstance(doc, dict):
        return None
    rows = doc.get("residual")
    if not isinstance(rows, list) or not rows:
        return None
    r = 0
    t = 0
    for row in rows:
        if not isinstance(row, dict):
            continue
        rv = row.get("residual")
        tv = row.get("total_slope")
        if is_number(rv):
            r += rv
        if is_number(tv):
            t += tv
    return r, t


def load_run_json(path):
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        raise GradeError(f"{path}: cannot parse run.json: {e}")
    if not isinstance(doc, dict):
        raise GradeError(f"{path}: run.json is not a JSON object")
    return doc


def token_value(usage, key):
    if isinstance(usage, dict):
        v = usage.get(key)
        if isinstance(v, int) and not isinstance(v, bool):
            return v
    return 0


def usage_block(runs, incomplete=0):
    n = len(runs)
    out = {"runs": n, "incomplete": incomplete}
    out["errored"] = sum(
        1
        for r in runs
        if not (
            isinstance(r["run_json"].get("exit_code"), int)
            and not isinstance(r["run_json"].get("exit_code"), bool)
            and r["run_json"].get("exit_code") == 0
        )
        or r["run_json"].get("timed_out") is True
    )
    src = {"ledger": 0, "findings": 0, "none": 0}
    for r in runs:
        src[r["source"]] += 1
    out["sources"] = src
    for key in TOKEN_FIELDS:
        total = sum(token_value(r["run_json"].get("usage"), key) for r in runs)
        out[key] = {"sum": total, "mean": round6(total / n) if n else None}
    walls = [
        float(r["run_json"]["wall_seconds"])
        if is_number(r["run_json"].get("wall_seconds"))
        else 0.0
        for r in runs
    ]
    out["wall_seconds"] = {
        "sum": round6(seq_sum(walls)),
        "mean": round6(seq_sum(walls) / n) if n else None,
        "max": round6(max(walls)) if n else None,
    }
    costs = [
        r["run_json"]["cost_usd"] for r in runs if is_number(r["run_json"].get("cost_usd"))
    ]
    if costs:
        total = seq_sum(costs)
        out["cost_usd"] = {"sum": round6(total), "mean": round6(total / len(costs))}
    else:
        out["cost_usd"] = {"sum": 0.0, "mean": None}
    return out


def grade_group(ablation, archetypes, runs, fixtures, incomplete=0):
    """runs: list of dicts sorted by (fixture, index)."""
    max_idx = max((r["index"] for r in runs), default=0)
    # per run index contributions
    per_idx = {i: {m: [0, 0] for m in METRIC_NAMES} for i in range(1, max_idx + 1)}
    fixture_blocks = {}
    resid_r = 0
    resid_t = 0
    resid_k = 0
    for r in runs:
        fx = fixtures[r["fixture"]]
        contrib, stats = evaluate_run(fx, r["findings"])
        for m in METRIC_NAMES:
            per_idx[r["index"]][m][0] += contrib[m][0]
            per_idx[r["index"]][m][1] += contrib[m][1]
        blk = fixture_blocks.get(r["fixture"])
        if blk is None:
            blk = {
                "category": fx.get("category"),
                "runs": 0,
                "claim_runs": 0,
                "rule_out_runs": 0,
                "leaks": {
                    str(lk.get("id")): {"found": 0, "flagged": 0, "grounded": 0}
                    for lk in (fx.get("leak") or [])
                },
            }
            fixture_blocks[r["fixture"]] = blk
        blk["runs"] += 1
        blk["claim_runs"] += 1 if stats["claim"] else 0
        blk["rule_out_runs"] += 1 if stats["rule_out"] else 0
        for lid, st in stats["leaks"].items():
            for k in ("found", "flagged", "grounded"):
                blk["leaks"][lid][k] += st[k]
            if st["worker"] is not None:
                by_worker = blk.setdefault("worker_provenance", {}).setdefault(lid, {})
                by_worker[st["worker"]] = by_worker.get(st["worker"], 0) + 1
        cov = coverage_residuals(r["dir"])
        if cov is not None:
            resid_r += cov[0]
            resid_t += cov[1]
            resid_k += 1

    metrics = {}
    for m in METRIC_NAMES:
        pairs = [(i, per_idx[i][m][0], per_idx[i][m][1]) for i in range(1, max_idx + 1)]
        metrics[m] = ratio_metric(pairs)

    if resid_k and resid_t != 0:
        share = {"value": round6(resid_r / resid_t), "runs": resid_k}
    else:
        share = {"value": None, "runs": resid_k}

    return {
        "ablation": ablation,
        "archetypes": archetypes,
        "metrics": metrics,
        "unexplained_growth_share": share,
        "usage": usage_block(runs, incomplete),
        "fixtures": fixture_blocks,
    }


# ---------------------------------------------------------------------------
# Discovery and driver
# ---------------------------------------------------------------------------


def load_fixture(fixtures_dir, name, cache):
    if name in cache:
        return cache[name]
    p = fixtures_dir / name / "expected.toml"
    if not p.is_file():
        raise GradeError(f"fixture {name}: {p} does not exist")
    try:
        with open(p, "rb") as fh:
            doc = tomllib.load(fh)
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as e:
        raise GradeError(f"fixture {name}: cannot parse {p}: {e}")
    cache[name] = doc
    return doc


def subdirs(p):
    try:
        return sorted(c for c in p.iterdir() if c.is_dir())
    except OSError:
        return []


def grade(results, fixtures_dir):
    results = Path(results)
    fixtures_dir = Path(fixtures_dir)
    if not results.is_dir():
        raise GradeError(f"{results}: not a directory")
    cache = {}
    groups = {}
    incomplete = {}
    runs_dir = results / "runs"
    if not runs_dir.is_dir():
        raise GradeError(f"{runs_dir}: not a directory")
    for abl in subdirs(runs_dir):
        if abl.name not in ABLATIONS:
            continue
        for arch in subdirs(abl):
            if arch.name not in ("archetypes-on", "archetypes-off"):
                continue
            arch_name = arch.name[len("archetypes-"):]
            for fx_dir in subdirs(arch):
                for run_dir in subdirs(fx_dir):
                    if not INDEX_RE.match(run_dir.name):
                        continue
                    idx = int(run_dir.name)
                    if idx < 1:
                        continue
                    rj = run_dir / "run.json"
                    if not rj.is_file():
                        incomplete[(abl.name, arch_name)] = (
                            incomplete.get((abl.name, arch_name), 0) + 1
                        )
                        groups.setdefault((abl.name, arch_name), [])
                        continue  # not a finished run
                    load_fixture(fixtures_dir, fx_dir.name, cache)
                    run_json = load_run_json(rj)
                    source, findings = load_findings(run_dir)
                    groups.setdefault((abl.name, arch_name), []).append(
                        {
                            "fixture": fx_dir.name,
                            "index": idx,
                            "dir": run_dir,
                            "run_json": run_json,
                            "source": source,
                            "findings": findings,
                        }
                    )
    out_groups = []
    for key in sorted(groups):
        runs = sorted(groups[key], key=lambda r: (r["fixture"], r["index"]))
        out_groups.append(grade_group(key[0], key[1], runs, cache, incomplete.get(key, 0)))
    return {"schema_version": 1, "groups": out_groups}


def main(argv=None):
    ap = argparse.ArgumentParser(description="Grade DLA benchmark results.")
    ap.add_argument("results")
    ap.add_argument("--fixtures", default="bench/fixtures")
    ap.add_argument("--out", default=None)
    args = ap.parse_args(argv)
    try:
        doc = grade(args.results, args.fixtures)
    except GradeError as e:
        print(f"grade: {e}", file=sys.stderr)
        return 1
    out = Path(args.out) if args.out else Path(args.results) / "metrics.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(doc, fh, indent=2, sort_keys=True, ensure_ascii=False)
        fh.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
