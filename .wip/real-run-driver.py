#!/usr/bin/env python3
"""Run the DLA plugin headless on a real project, the way the benchmark runner runs a fixture,
but with package registries reachable.

usage: run.py <name> <source dir> <prompt file> [--model M] [--timeout S] [--max-turns N]

Layout under /root/dla-real/<name>/: project (working copy, a git repository), cache, tmp,
config, settings.json, transcript.jsonl, stderr.log, run.json.
The plugin is /root/dla-real/plugin (a frozen copy) and the binary /root/dla-real/bin/dla.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import time

ROOT = "/root/dla-real"
REGISTRIES = [
    "pypi.org", "files.pythonhosted.org",
    "proxy.golang.org", "sum.golang.org", "storage.googleapis.com",
    "registry.npmjs.org",
    "repo.maven.apache.org", "repo1.maven.org", "plugins.gradle.org", "services.gradle.org",
    "downloads.gradle.org", "repo.gradle.org", "jcenter.bintray.com", "repo.spring.io",
    "crates.io", "static.crates.io", "index.crates.io",
    # analyzer downloads of `dla install`
    "github.com", "objects.githubusercontent.com", "release-assets.githubusercontent.com",
    "go.dev", "dl.google.com",
]
UNATTENDED = ("This is an unattended run: nobody can answer questions. Consent is already given "
              "for privilege tiers 1-4 and for raw values.")
DENY_PREFIX = ("CLAUDE", "CCR_", "DOCUMENTS_MCP", "SBX_", "DLA_", "MODAL_", "FSSPEC_", "CLOUDSDK_")
DENY_NAME = {"AI_AGENT", "SESSION_INGRESS_URL", "MAX_THINKING_TOKENS", "GH_TOKEN", "GITHUB_TOKEN",
             "IS_SANDBOX"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("source")
    ap.add_argument("prompt")
    ap.add_argument("--model", default="claude-sonnet-5-5")
    ap.add_argument("--timeout", type=int, default=3 * 3600)
    ap.add_argument("--max-turns", type=int, default=600)
    ap.add_argument("--deny-read", action="append", default=[])
    a = ap.parse_args()

    work = os.path.join(ROOT, a.name)
    if os.path.exists(work):
        sys.exit(f"{work} exists; choose another name")
    project = os.path.join(work, "project")
    cache = os.path.join(work, "cache")
    tmp = os.path.join(work, "tmp")
    config = os.path.join(work, "config")
    data = os.path.join(config, "plugins", "data")
    for d in (cache, tmp, data):
        os.makedirs(d)
    shutil.copytree(a.source, project, symlinks=True,
                    ignore=shutil.ignore_patterns(".dla", ".git"))
    git = ["git", "-C", project, "-c", "user.name=run", "-c", "user.email=run@example.invalid"]
    subprocess.run(["git", "-C", project, "init", "-q"], check=True)
    subprocess.run(["git", "-C", project, "add", "-A"], check=True)
    subprocess.run(git + ["commit", "-qm", "import"], check=True)

    deny_read = ["/home/user/dla", "/tmp/claude-0", "/root/.claude"] + a.deny_read
    for other in os.listdir(ROOT):
        p = os.path.join(ROOT, other)
        if other not in (a.name, "plugin", "bin") and os.path.isdir(p):
            deny_read.append(p)
    allow_write = [project, cache, tmp, data, "/tmp"]
    allow = ["Bash", "Read", "Glob", "Grep", "Agent", "Task", "Skill"]
    for tool in ("Edit", "Write"):
        for p in (project, cache, tmp):
            allow.append(f"{tool}(/{p}/**)")
    deny = ["WebFetch", "WebSearch"]
    for tool in ("Read", "Edit", "Write", "Glob", "Grep"):
        for p in deny_read:
            deny.append(f"{tool}(/{p}/**)")
    settings = {
        "sandbox": {
            "enabled": True,
            "autoAllowBashIfSandboxed": True,
            "allowUnsandboxedCommands": False,
            "failIfUnavailable": True,
            "filesystem": {"denyRead": deny_read, "allowWrite": allow_write},
            "network": {"allowedDomains": REGISTRIES},
        },
        "permissions": {"allow": allow, "deny": deny},
    }
    settings_path = os.path.join(work, "settings.json")
    with open(settings_path, "w") as f:
        json.dump(settings, f, indent=2)

    env = {k: v for k, v in os.environ.items()
           if not k.startswith(DENY_PREFIX) and k not in DENY_NAME}
    at = lambda sub: os.path.join(cache, sub)
    env.update({
        "CLAUDE_CONFIG_DIR": config,
        "DISABLE_AUTOUPDATER": "1",
        "CLAUDE_CODE_TMPDIR": tmp,
        "GOCACHE": at("go-build"), "GOPATH": at("gopath"), "GOMODCACHE": at("gopath/pkg/mod"),
        "GOFLAGS": "-modcacherw",
        "MAVEN_OPTS": f"-Dmaven.repo.local={at('m2')}",
        "GRADLE_USER_HOME": at("gradle"),
        "npm_config_cache": at("npm"), "npm_config_update_notifier": "false",
        "PIP_CACHE_DIR": at("pip"),
        "CARGO_HOME": at("cargo"),
        "XDG_CACHE_HOME": at("xdg"),
        "DLA_BIN": os.path.join(ROOT, "bin", "dla"),
        "DLA_CONSENT_TIERS": "1-4",
        "DLA_CONSENT_RAW_VALUES": "1",
    })
    for sub in ("go-build", "gopath/pkg/mod", "m2", "gradle", "npm", "pip", "cargo", "xdg"):
        os.makedirs(at(sub), exist_ok=True)

    task = open(a.prompt).read().strip()
    prompt = f"/dla:detect-leak-anywhere {task}\n\n{UNATTENDED}"
    cmd = ["claude", "-p", prompt, "--output-format", "stream-json", "--verbose",
           "--max-turns", str(a.max_turns), "--settings", settings_path, "--strict-mcp-config",
           "--no-session-persistence", "--model", a.model,
           "--plugin-dir", os.path.join(ROOT, "plugin")]
    started = time.time()
    with open(os.path.join(work, "transcript.jsonl"), "w") as out, \
            open(os.path.join(work, "stderr.log"), "w") as err:
        p = subprocess.Popen(cmd, cwd=project, env=env, stdin=subprocess.DEVNULL, stdout=out,
                             stderr=err, start_new_session=True)
        timed_out = False
        try:
            code = p.wait(timeout=a.timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(p.pid, 9)
            code = p.wait()
    wall = time.time() - started
    with open(os.path.join(work, "run.json"), "w") as f:
        json.dump({"name": a.name, "model": a.model, "exit_code": code, "timed_out": timed_out,
                   "wall_seconds": round(wall), "prompt": prompt}, f, indent=2)
    print(f"{a.name}: exit {code} timed_out {timed_out} wall {round(wall)} s")


if __name__ == "__main__":
    main()
