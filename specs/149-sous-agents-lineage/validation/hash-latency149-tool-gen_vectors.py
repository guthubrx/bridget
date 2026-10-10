import hashlib, subprocess, tempfile, os
def pat(n): return bytes((i*7+3)%251 for i in range(n))
lens=[1,55,56,63,64,65,119,120,127,128,65535,65536,65537,131072,131073,1000003]
rows=[]
for n in lens:
    d=pat(n); h=hashlib.sha256(d).hexdigest()
    with tempfile.NamedTemporaryFile(dir='/tmp/b149t-r6',delete=False) as f: f.write(d); p=f.name
    s=subprocess.check_output(['shasum','-a','256',p]).split()[0].decode(); os.unlink(p)
    assert s==h,(n,s,h)
    rows.append((n,h))
for n,h in rows: print(f'        ({n}, "{h}"),')
