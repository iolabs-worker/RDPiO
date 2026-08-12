import sys

sub = sys.argv[1]
s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
n = sum(1 for l in s.splitlines() if sub in l)
print('0' * n)
