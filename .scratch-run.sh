#!/usr/bin/env bash
# Start a cargo command in the background, logging to a file.
# Env: CARGO_ARGS (the cargo subcommand + args), LOGFILE
set -u
export PATH="$HOME/.cargo/bin:$PATH"
LOG="${LOGFILE:-/srv/swarm_web_runs/run-1786496646702-0001/codebase_output/repo/.scratch-check.log}"
cd /srv/swarm_web_runs/run-1786496646702-0001/codebase_output/repo
echo "START $(date +%s) :: cargo $CARGO_ARGS" > "$LOG"
nohup cargo $CARGO_ARGS >> "$LOG" 2>&1 &
echo "PID $!" >> "$LOG"
