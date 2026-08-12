import sys

mode = sys.argv[1] if len(sys.argv) > 1 else 'errors'
s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
lines = s.splitlines()
if mode == 'errors':
    n = sum(1 for line in lines if line.startswith('error'))
    print('0' * n)
else:
    n = sum(1 for line in lines if line.startswith('warning'))
    print('1' * n)
