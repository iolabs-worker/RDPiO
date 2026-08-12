import sys

cat = sys.argv[1]
s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
errs = [l for l in s.splitlines() if l.startswith('error')]
if cat == 'rustc':
    n = sum(1 for l in errs if l.startswith('error['))
elif cat == 'could':
    n = sum(1 for l in errs if 'could not compile' in l)
elif cat == 'dead':
    n = sum(1 for l in errs if 'never used' in l or 'never constructed' in l or 'never read' in l)
elif cat == 'large':
    n = sum(1 for l in errs if 'very large' in l or 'large size difference' in l)
else:
    n = sum(1 for l in errs if not (l.startswith('error[') or 'could not compile' in l or 'never used' in l or 'never constructed' in l or 'never read' in l or 'very large' in l or 'large size difference' in l))
print('0' * n)
