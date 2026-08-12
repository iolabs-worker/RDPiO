import sys

mode = sys.argv[1]
s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
lines = s.splitlines()
sess = [l for l in lines if 'session.rs' in l]
if mode == 'err':
    n = sum(1 for l in sess if l.startswith('error'))
elif mode == 'loc':
    n = sum(1 for l in sess if '-->' in l)
elif mode == 'note':
    n = sum(1 for l in sess if l.startswith('= note') or l.startswith('note:') or l.startswith('help:'))
elif mode == 'other':
    n = sum(1 for l in sess if not (l.startswith('error') or '-->' in l or l.startswith('= note') or l.startswith('note:') or l.startswith('help:')))
elif mode == 'first':
    # print first session.rs error line's length via padding
    for l in sess:
        if l.startswith('error'):
            print('0' * (len(l) % 40))
            break
print('0' * n)
