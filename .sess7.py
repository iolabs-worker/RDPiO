import sys
import re

mode = sys.argv[1]
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
lo, hi = {'a': (0, 400), 'b': (400, 1000), 'c': (1000, 1500), 'd': (1500, 10**9)}[mode]
n = sum(1 for x in dead_locs if lo <= x < hi)
print('0' * n)
