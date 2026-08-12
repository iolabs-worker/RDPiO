#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd /srv/swarm_web_runs/run-1786496646702-0001/codebase_output/repo
cargo check --workspace 2>&1 | tail -80
