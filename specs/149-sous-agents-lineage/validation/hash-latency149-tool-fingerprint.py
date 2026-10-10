import hashlib, os, sys, json
ROOT = '/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage'
files = []
for d, dirs, fs in os.walk(os.path.join(ROOT, 'crates')):
    dirs[:] = [x for x in dirs if x not in ('target', '.git')]
    for f in fs:
        if f.endswith('.rs') or f.endswith('.toml') or f == 'Cargo.lock':
            files.append(os.path.relpath(os.path.join(d, f), ROOT))
for f in ('Cargo.toml', 'Cargo.lock'):
    files.append(f)
files = sorted(set(files))
import subprocess
OVR={}
if os.environ.get('R5_CHECK'):
    OVR={'Cargo.lock':open('/tmp/b149t-r6/Cargo.lock.avant','rb').read(),'Cargo.toml':subprocess.check_output(['git','-C',ROOT,'show','HEAD:Cargo.toml'])}
def rd(p):
    return OVR[p] if p in OVR else open(os.path.join(ROOT,p),'rb').read()
def digest(paths):
    lines = []
    for p in paths:
        h = hashlib.sha256(rd(p)).hexdigest()
        lines.append(f'{p}\0{h}')
    return hashlib.sha256('\n'.join(lines).encode()).hexdigest()
def is_test(p):
    return '/tests/' in p or p.endswith('_test.rs') or p.endswith('_tests.rs')
prod = [p for p in files if not is_test(p)]
out = {'all': {'value': digest(files), 'fileCount': len(files)}, 'prod': {'value': digest(prod), 'fileCount': len(prod)}}
if len(sys.argv) > 1:
    out['files'] = {p: hashlib.sha256(rd(p)).hexdigest() for p in sys.argv[1:]}
print(json.dumps(out, indent=1))
