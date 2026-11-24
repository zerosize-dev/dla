#!/bin/bash
# Start one unattended real-project run of round 2 in the background.
# usage: start-run.sh <project key> <run name>
# Project keys: set A (development set): otel-python otel-go interceptors fastrace logstash
#               set B (held out): mcp crypto jni micrometer        owner's list: jetty
SP=/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad
R=/root/dla-real
key="$1"; name="$2"
[ -n "$key" ] && [ -n "$name" ] || { echo "usage: start-run.sh <project key> <run name>"; exit 2; }
common=("$R/prompt-generic.txt" --timeout 12600 --max-turns 600)
case "$key" in
  otel-python) args=("$R/src/otel-python" "${common[@]}" --py-venv python3.12 --py-setup "uv sync --frozen --inexact && uv pip install pytest==7.4.4 flaky==3.7.0");;
  otel-go)     args=("$R/src/otel-go" "${common[@]}");;
  interceptors) args=("$R/src/interceptors" "${common[@]}");;
  fastrace)    args=("$R/src/fastrace" "${common[@]}");;
  logstash)    args=("$R/src/logstash" "${common[@]}" --warm-m2 "$SP/u107/logstash-try/m2");;
  mcp)         args=("$R/src/mcp-sdk" "${common[@]}" --py-venv python3.12 --py-setup "uv sync --frozen --inexact --all-extras");;
  crypto)      args=("$R/src/x-crypto" "${common[@]}");;
  jni)         args=("$R/src/jni-rs" "${common[@]}" --env LD_LIBRARY_PATH=/usr/lib/jvm/java-21-openjdk-amd64/lib/server);;
  micrometer)  args=("$R/src/micrometer" "${common[@]}" --warm-gradle "$SP/u119/micrometer-try/gradle" --env JAVA_HOME=/usr/lib/jvm/java-11-openjdk-amd64 --path-prepend /usr/lib/jvm/java-11-openjdk-amd64/bin);;
  jetty)       args=("$R/src/jetty" "${common[@]}" --warm-m2 "$SP/u107/jetty-try/m2");;
  *) echo "unknown project key $key"; exit 2;;
esac
nohup setsid python3 "$R/run.py" "$name" "${args[@]}" > "$R/$name.driver.log" 2>&1 &
echo "started $name ($key) at $(date -u +%H:%M) pid $!"
