import sys

mode = sys.argv[1]
s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
lines = s.splitlines()
idxs = [i for i, l in enumerate(lines) if 'session.rs' in l and '-->' in l]
n = 0
for i in idxs:
    msg = ''
    for j in range(i - 1, max(0, i - 4), -1):
        if lines[j].startswith('error'):
            msg = lines[j]
            break
    if mode == 'large' and 'large' in msg:
        n += 1
    elif mode == 'dead' and 'never' in msg:
        n += 1
    elif mode == 'other' and 'large' not in msg and 'never' not in msg:
        n += 1
print('0' * n)
