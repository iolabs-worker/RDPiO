import sys

s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
lines = s.splitlines()
idxs = [i for i, l in enumerate(lines) if 'session.rs' in l and '-->' in l]
n_large = 0
n_dead = 0
n_other = 0
for i in idxs:
    msg = ''
    for j in range(i - 1, max(0, i - 4), -1):
        if lines[j].startswith('error'):
            msg = lines[j]
            break
    if 'large' in msg or 'very large' in msg:
        n_large += 1
    elif 'never' in msg:
        n_dead += 1
    else:
        n_other += 1
print('A' * n_large)
print('B' * n_dead)
print('C' * n_other)
