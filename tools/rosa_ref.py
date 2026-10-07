"""Referencia ROSA (copia literal de rosa() en external/lang/RWKV-LM/RWKV-v8/251014_rosa_1bit_layer.py)
para validar el port en Rust. Escribe casos aleatorios en JSON."""
import json, random, sys

def rosa(x):
	n=len(x); y=[-1]*n; s=2*n+1; b=[None]*s; c=[-1]*s; d=[0]*s; e=[-1]*s; b[0]={}; g=0; z=1
	for i,t in enumerate(x):
		r=z; z+=1; b[r]={}; d[r]=d[g]+1; p=g
		while p!=-1 and t not in b[p]: b[p][t]=r; p=c[p]
		if p==-1: c[r]=0
		else:
			q=b[p][t]
			if d[p]+1==d[q]: c[r]=q
			else:
				u=z; z+=1; b[u]=b[q].copy(); d[u]=d[p]+1; c[u]=c[q]; e[u]=e[q]
				while p!=-1 and b[p][t]==q: b[p][t]=u; p=c[p]
				c[q]=c[r]=u
		v=g=r; a=-1
		while v!=-1:
			if d[v]>0 and e[v]>=0: a=x[e[v]+1]; break
			v=c[v]
		y[i]=a; v=g
		while v!=-1 and e[v]<i: e[v]=i; v=c[v]
	return y

random.seed(0)
cases = []
for k in range(300):
    n = random.randint(1, 400); a = random.choice([2, 3, 5, 30])
    x = [random.randrange(a) for _ in range(n)]
    cases.append({"x": x, "y": rosa(x)})
json.dump(cases, open(sys.argv[1], "w"))
