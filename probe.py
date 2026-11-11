"""搜索能自发产生自催化环的类型规则。"""
def scc(nodes, adj):
    idx = {}; low = {}; on = {}; st = []; out = []; c = [0]
    def go(v):
        idx[v] = low[v] = c[0]; c[0] += 1
        st.append(v); on[v] = True
        for w in adj.get(v, ()):
            if w not in idx:
                go(w); low[v] = min(low[v], low[w])
            elif on.get(w):
                low[v] = min(low[v], idx[w])
        if low[v] == idx[v]:
            comp = []
            while True:
                w = st.pop(); on[w] = False; comp.append(w)
                if w == v: break
            out.append(comp)
    for v in nodes:
        if v not in idx: go(v)
    return [c for c in out if len(c) > 1]

def test(K, f, label):
    adj = {i: set() for i in range(K)}
    for a in range(K):
        for b in range(K):
            c, k = f(a, b, K)
            adj[k].add(c)
    comps = scc(range(K), adj)
    deg = sum(len(v) for v in adj.values()) / K
    print(f'{label:28s} K={K:2d} 边/节点={deg:4.1f} 非平凡SCC={[len(c) for c in comps]}')
    if comps:
        print('   ', comps[:4])

rules = [
    ('c=a*b+1, k=a+b',      lambda a,b,K: ((a*b+1)%K, (a+b)%K)),
    ('c=a*b+1, k=a*b',      lambda a,b,K: ((a*b+1)%K, (a*b)%K)),
    ('c=a+b+1, k=a*b',      lambda a,b,K: ((a+b+1)%K, (a*b)%K)),
    ('c=a*3+b*5+1, k=a*7+b*11+3', lambda a,b,K: ((a*3+b*5+1)%K, (a*7+b*11+3)%K)),
    ('c=a^b, k=a*b+1',      lambda a,b,K: ((a^b)%K, (a*b+1)%K)),
]
for K in (6, 8, 12, 16):
    for label, f in rules:
        test(K, f, label)
    print()
