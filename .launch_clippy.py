import subprocess

log = open('/tmp/clippy3.log', 'wb')
p = subprocess.Popen(
    ['cargo', 'clippy', '--workspace', '--all-targets', '--all-features',
     '--message-format=human', '--', '-D', 'warnings'],
    cwd='/srv/swarm_web_runs/run-1786509241896-0002/codebase_output/repo',
    stdout=log, stderr=subprocess.STDOUT)
open('/tmp/clippy3.pid', 'w').write(str(p.pid))
print('PID', p.pid)
