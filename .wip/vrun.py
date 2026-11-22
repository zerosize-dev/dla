#!/usr/bin/env python3
"""Run the DLA verifier agent headless on one finding of a replayed run directory.

usage: vrun.py <name> <base project dir (with .dla)> <finding> <plugin dir> <dla binary>

Copies the base project to /root/dla-real/<name>/project, makes the bundle with the given
binary, starts the plugin's verifier agent on it (as the skill does) and prints what it recorded.
"""
import json
import os
import shutil
import subprocess
import sys
import time

ROOT = "/root/dla-real"
DENY_PREFIX = ("CLAUDE", "CCR_", "DOCUMENTS_MCP", "SBX_", "DLA_", "MODAL_", "FSSPEC_", "CLOUDSDK_")
DENY_NAME = {"AI_AGENT", "SESSION_INGRESS_URL", "MAX_THINKING_TOKENS", "GH_TOKEN", "GITHUB_TOKEN",
             "IS_SANDBOX"}


def main():
    name, base, finding, plugin, binary = sys.argv[1:6]
    model = "claude-sonnet-5-5"
    work = os.path.join(ROOT, name)
    if os.path.exists(work):
        sys.exit(f"{work} exists; choose another name")
    project = os.path.join(work, "project")
    tmp = os.path.join(work, "tmp")
    config = os.path.join(work, "config")
    os.makedirs(tmp)
    os.makedirs(os.path.join(config, "plugins", "data"))
    shutil.copytree(base, project, symlinks=True)
    run = open(os.path.join(project, ".dla", "current")).read().strip()

    env = {k: v for k, v in os.environ.items()
           if not k.startswith(DENY_PREFIX) and k not in DENY_NAME}
    env.update({
        "CLAUDE_CONFIG_DIR": config,
        "DISABLE_AUTOUPDATER": "1",
        "CLAUDE_CODE_TMPDIR": tmp,
        "DLA_BIN": binary,
        "DLA_PLUGIN_ROOT": plugin,
        "PATH": env.get("PATH", "") + os.pathsep + os.path.join(plugin, "bin"),
    })
    out = subprocess.run([binary, "--project", project, "bundle", finding], env=env,
                         capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"bundle failed: {out.stdout} {out.stderr}")
    b = json.loads(out.stdout)
    bundle = os.path.join(project, b["dir"])
    sha = b["manifest_sha256"]

    deny_read = ["/home/user/dla", "/tmp/claude-0", "/root/.claude"]
    keep = {name, os.path.basename(plugin.rstrip("/")), os.path.basename(os.path.dirname(binary))}
    for other in os.listdir(ROOT):
        p = os.path.join(ROOT, other)
        if other not in keep and os.path.isdir(p):
            deny_read.append(p)
    allow = ["Bash", "Read", "Glob", "Grep"]
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
            "filesystem": {"denyRead": deny_read, "allowWrite": [project, tmp, "/tmp"]},
            "network": {"allowedDomains": []},
        },
        "permissions": {"allow": allow, "deny": deny},
    }
    settings_path = os.path.join(work, "settings.json")
    with open(settings_path, "w") as f:
        json.dump(settings, f, indent=2)

    prompt = (f"Run id {run}, finding {finding}. Bundle directory: {bundle} (manifest sha256 {sha}). "
              f"Project root: {project}. Follow your verifier instructions and record with dla ledger verify.")
    cmd = ["claude", "-p", prompt, "--agent", "dla:verifier", "--output-format", "stream-json",
           "--verbose", "--max-turns", "60", "--settings", settings_path, "--strict-mcp-config",
           "--no-session-persistence", "--model", model, "--plugin-dir", plugin]
    started = time.time()
    with open(os.path.join(work, "transcript.jsonl"), "w") as o, \
            open(os.path.join(work, "stderr.log"), "w") as e:
        code = subprocess.run(cmd, cwd=project, env=env, stdin=subprocess.DEVNULL, stdout=o,
                              stderr=e, timeout=900).returncode
    wall = round(time.time() - started)

    final, verify = "", ""
    for line in open(os.path.join(work, "transcript.jsonl")):
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if d.get("type") == "result":
            final = d.get("result") or ""
        m = d.get("message", {})
        c = m.get("content")
        if isinstance(c, list):
            for x in c:
                if x.get("type") == "tool_use" and "ledger verify" in json.dumps(x.get("input", {})):
                    verify = x["input"].get("command", "")
    with open(os.path.join(work, "run.json"), "w") as f:
        json.dump({"name": name, "finding": finding, "exit_code": code, "wall_seconds": wall,
                   "bundle": b, "verify_command": verify, "final": final}, f, indent=2)
    print(f"== {name} {finding} exit {code} wall {wall}s")
    print("VERIFY:", verify[-900:])
    print("FINAL:", final[:1500])


if __name__ == "__main__":
    main()
