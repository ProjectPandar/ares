from pathlib import Path
import struct,collections,json,hashlib
ROOT=Path(__file__).resolve().parent
vertices=[]; triangles=[]
def tri(a,b,c):
    ids=[]
    for p in (a,b,c):
        if p not in vertices: vertices.append(p)
        ids.append(vertices.index(p))
    triangles.append(ids)
def quad(p,reverse=False):
    if reverse:p=p[::-1]
    tri(*p[:3]);tri(p[0],p[2],p[3])
def sq(r,z):return [(150-r,150-r,z),(150+r,150-r,z),(150+r,150+r,z),(150-r,150+r,z)]
def ring(a,b,reverse=False):
    for i in range(4):
        j=(i+1)%4;quad([a[i],a[j],b[j],b[i]],reverse)
outer0,outer1=sq(22,0),sq(22,.52)
h0,h1,h2,h3=sq(14,0),sq(14,.32),sq(10,.32),sq(10,.52)
ring(outer0,h0,True);ring(outer1,h3);ring(h1,h2,True)
for a,b,reverse in [(outer0,outer1,False),(h0,h1,True),(h2,h3,True)]:
    for i in range(4):
        j=(i+1)%4;quad([a[i],a[j],b[j],b[i]],reverse)
xy=[(-7,-3),(-1,-3),(-1,-.3),(1,-.3),(1,-3),(7,-3),(7,3),(1,3),(1,.3),(-1,.3),(-1,3),(-7,3)]
cap=[(0,1,2),(0,2,9),(0,9,10),(0,10,11),(2,3,8),(2,8,9),(5,6,7),(5,7,8),(5,8,3),(5,3,4)]
for z in [0,.52]:
    p=[(150+x,150+y,z) for x,y in xy]
    for a,b,c in cap:tri(p[a],p[c],p[b]) if z==0 else tri(p[a],p[b],p[c])
for i in range(12):
    j=(i+1)%12;x,y=xy[i];u,v=xy[j]
    quad([(150+x,150+y,0),(150+u,150+v,0),(150+u,150+v,.52),(150+x,150+y,.52)])
edges=collections.Counter(tuple(sorted((t[i],t[(i+1)%3]))) for t in triangles for i in range(3));assert set(edges.values())=={2}
data=b'N-split bounded discovery ring and connected dumbbell'.ljust(80,b'\0')+struct.pack('<I',len(triangles))
for ids in triangles:
    p,q,r=[vertices[i] for i in ids];u=[q[i]-p[i] for i in range(3)];v=[r[i]-p[i] for i in range(3)]
    n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];assert sum(x*x for x in n)>0
    data+=struct.pack('<12fH',*n,*p,*q,*r,0)
(ROOT/'n-split.stl').write_bytes(data)
summary={'hypothesis':'offset2 splits connected neck; not a branch witness','vertices':len(vertices),'triangles':len(triangles),'edge_use_counts':[2],'outer_halfwidth':22,'hole_halfwidths':[14,10],'post_xy':xy,'caps':cap}
(ROOT/'n-split-shape.json').write_text(json.dumps(summary,indent=2)+'\n')
assert hashlib.sha256(data).hexdigest() == '444935eac2493ed4f695e9fa635e8409fd4cad273ce817373a450c706bc9d807'
