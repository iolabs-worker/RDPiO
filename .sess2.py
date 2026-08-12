import sys

mode = sys.argv[1]
s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
lines = s.splitlines()
if mode == 'sl':
    n = sum(1 for l in lines if 'session.rs' in l and 'very large' in l)
elif mode == 'sd':
    n = sum(1 for l in lines if 'session.rs' in l and ('never used' in l or 'never constructed' in l or 'never read' in l))
elif mode == 'so':
    n = sum(1 for l in lines if 'session.rs' in l and not ('very large' in l or 'never used' in l or 'never constructed' in l or 'never read' in l))
print('0' * n)
