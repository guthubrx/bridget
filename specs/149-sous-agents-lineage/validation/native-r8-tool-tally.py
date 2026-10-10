import re,sys
t=open(sys.argv[1]).read()
tot=[0,0,0]; n=0
for m in re.finditer(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',t):
    n+=1; tot[0]+=int(m.group(2)); tot[1]+=int(m.group(3)); tot[2]+=int(m.group(4))
print(sys.argv[1],'executables',n,'passed',tot[0],'failed',tot[1],'ignored',tot[2])
print([l for l in t.splitlines() if 'FAILED' in l or l.startswith('error')][:10])
