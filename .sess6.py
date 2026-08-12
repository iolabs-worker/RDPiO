import sys
import re

s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
lines = s.splitlines()
idxs = [i for i, l in enumerate(lines) if 'session.rs' in l and '-->' in l]
dead_locs = []
for i in idxs:
    msg = ''
    for j in range(i - 1, max(0, i - 4), -1):
        if lines[j].startswith('error'):
            msg = lines[j]
            break
    if 'never' in msg:
        m = re.search(r'session\.rs:(\d+):', lines[i])
        if m:
            dead_locs.append(int(m.group(1)))
print('A' * sum(1 for x in dead_locs if x < 400))
print('B' * sum(1 for x in dead_locs if 400 <= x < 1000))
print('C' * sum(1 for x in dead_locs if 1000 <= x < 1500))
print('D' * sum(1 for x in dead_locs if x >= 1500))
