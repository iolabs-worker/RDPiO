import re

s = open('/tmp/clippy3.log', 'rb').read().decode('utf-8', 'replace')
lines = s.splitlines()
n_large = sum(1 for l in lines if 'session.rs' in l and 'very large' in l)
n_dead = sum(1 for l in lines if 'session.rs' in l and ('never used' in l or 'never constructed' in l or 'never read' in l))
n_other_sess = sum(1 for l in lines if 'session.rs' in l and not ('very large' in l or 'never used' in l or 'never constructed' in l or 'never read' in l))
print('A' * n_large)
print('B' * n_dead)
print('C' * n_other_sess)
