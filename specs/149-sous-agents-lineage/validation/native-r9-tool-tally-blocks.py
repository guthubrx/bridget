import re,sys
t=open(sys.argv[1]).read()
blocks=re.split(r'\n\s+Running |\n\s+Doc-tests ',t)[1:]
tot=[0,0,0]
for b in blocks:
    ms=re.findall(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',b,re.M)
    if ms:
        p,f,i=map(int,ms[-1])  # dernier résultat = celui de l'exécutable
        tot[0]+=p;tot[1]+=f;tot[2]+=i
print(sys.argv[1].split('/')[-1],'executables',len(blocks),'passed',tot[0],'failed',tot[1],'ignored',tot[2])
