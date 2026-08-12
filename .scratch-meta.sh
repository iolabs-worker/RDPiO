#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd /srv/swarm_web_runs/run-1786496646702-0001/codebase_output/repo
cargo metadata --no-deps --format-version 1 | python3 -c "
import json,sys
m = json.load(sys.stdin)
print('workspace members:')
for p in m['packages']:
    print(' ', p['name'], '->', p['manifest_path'])
print('workspace root:', m['workspace_root'])
"
