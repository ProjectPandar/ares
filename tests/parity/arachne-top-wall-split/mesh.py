from collections import Counter
from math import cos,sin,pi,tan
from struct import pack

tris=[]
def quad(a,b,c,d): tris.extend(((a,b,c),(a,c,d)))
def ring(cx,cy,sx,sy,h,inner,shape):
 def points(z,inside):
  r=[]
  for i in range(32):
   x,y=cos(2*pi*i/32),sin(2*pi*i/32)
   if inside: s=inner if shape=='round' else inner/max(abs(x),abs(y))
   else:s=1/max(abs(x)/sx,abs(y)/sy)
   r.append((cx+s*x,cy+s*y,z))
  return r
 a,b,c,d=points(0,False),points(0,True),points(h,False),points(h,True)
 for i in range(32):
  j=(i+1)%32
  quad(a[i],b[i],b[j],a[j]); quad(c[i],c[j],d[j],d[i]); quad(a[i],a[j],c[j],c[i]); quad(b[j],b[i],d[i],d[j])

def cross(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
def ears(poly):
 ids=list(range(len(poly)))
 while len(ids)>3:
  for j,b in enumerate(ids):
   a,c=ids[j-1],ids[(j+1)%len(ids)]
   p,q,r=poly[a],poly[b],poly[c]
   if cross(p,q,r)<=1e-9:continue
   if any(min(cross(p,q,poly[k]),cross(q,r,poly[k]),cross(r,p,poly[k]))>=-1e-9 for k in ids if k not in (a,b,c)):continue
   yield a,b,c; ids.pop(j); break
  else:raise ValueError(poly)
 yield tuple(ids)

def prism(poly,place):
 a,b=([place(u,v,w) for u,v in poly] for w in (0,1))
 for i,j,k in ears(poly):tris.extend(((a[k],a[j],a[i]),(b[i],b[j],b[k])))
 for i in range(len(a)):
  j=(i+1)%len(a); quad(a[i],a[j],b[j],b[i])

def box(x,y,w,d,h):
 prism([(0,0),(w,0),(w,d),(0,d)],lambda u,v,t:(x+u,y+v,t*h))

def generate(path):
 tris.clear()
 # Two broad shells with vertical holes, distinct top elevations, and solid-fill area.
 ring(18,16,18,16,7.4,5,'round'); ring(52,14,13,12,5.6,3.6,'square')
 # A through-slot, open from both ends: 18 mm clear unsupported horizontal span at Z=5.
 bridge=[(0,0),(4,0),(4,5),(22,5),(22,0),(26,0),(26,7.6),(0,7.6)]
 prism(bridge,lambda x,z,t:(3+x,36+(1-t)*15,z))
 for i,deg in enumerate((30,45,60)):
  shift=2.25/tan(deg*pi/180); x=34+9*i
  # Down-facing overhang segment at angle from horizontal; top is deliberately sloped.
  poly=[(0,0),(3.7,0),(3.7,1.6),(3.7+shift,3.85),(3.7+shift,5.0),(0,7.2-i*.4)]
  prism(poly,lambda u,z,t:(x+u,35+(1-t)*14,z))
 # Thin ribs, tapered narrow gap and small flat top islands. 2.75 mm square has
 # <15 mm² gross area; the width ladder leaves progressively narrower in-wall cores.
 for i,w in enumerate((0.47,0.88,1.22,1.52)):
  prism([(0,0),(w,0),(w*.70,9),(0,9)],lambda u,v,t:(3+i*3.5+u,55+v,t*3.4))
 for i,w in enumerate((2.75,3.6,4.35,6.1)):
  box(21+9*i,55,w,w,4.0+i*.6)
 edges=Counter(); oriented=Counter()
 for tri in tris:
  for a,b in zip(tri,tri[1:]+tri[:1]):
   a,b=(pack('<3f',*p) for p in (a,b)); assert a!=b
   edges[tuple(sorted((a,b)))]+=1;oriented[a,b]+=1
 assert set(edges.values())=={2} and all(n==oriented[b,a]==1 for (a,b),n in oriented.items())
 path.write_bytes(b'Synthetic perimeter plate'.ljust(80,b'\0')+pack('<I',len(tris))+b''.join(pack('<12fH',0,0,0,*(x for p in tri for x in p),0) for tri in tris))
 return len(tris)

if __name__=='__main__':
 from pathlib import Path
 from sys import argv
 generate(Path(argv[1]))
