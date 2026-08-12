#!/usr/bin/env bash
# Poll a background cargo log. Exit 0 if the cargo process finished.
set -u
LOG="${LOGFILE:-/srv/swarm_web_runs/run-1786496646702-0001/codebase_output/repo/.scratch-check.log}"
sleep "${POLL_SLEEP:-20}"
if [ ! -f "$LOG" ]; then echo "no log"; exit 1; fi
PID=$(grep -oP '^PID \K[0-9]+' "$LOG" || true)
if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
  echo "RUNNING (pid $PID) — last lines:"
  tail -6 "$LOG"
  exit 1
fi
echo "DONE — full log tail:"
tail -80 "$LOG"
exit 0
