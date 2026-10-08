from pathlib import Path
import collections, hashlib, json, struct
ROOT=Path(__file__).resolve().parent

def mesh(lower_hole,upper_hole,height,centers):
    vertices=[];triangles=[]
    def triangle(a,b,c):
        indices=[]
        for point in (a,b,c):
            if point not in vertices:vertices.append(point)
            indices.append(vertices.index(point))
        triangles.append(indices)
    def quad(points,reverse=False):
        if reverse:points=list(reversed(points))
        triangle(*points[:3]);triangle(points[0],points[2],points[3])
    def ring(outer,inner,reverse=False):
        for i in range(4):
            j=(i+1)%4
            quad([outer[i],outer[j],inner[j],inner[i]],reverse)
    for cx,cy in centers:
        def square(r,z):return [(cx-r,cy-r,z),(cx+r,cy-r,z),(cx+r,cy+r,z),(cx-r,cy+r,z)]
        out0=square(12,0);out1=square(12,height)
        walls=[(out0,out1,False)]
        if lower_hole:
            hole0=square(lower_hole//2,0);hole3=square(upper_hole//2,height)
            ring(out0,hole0,True);ring(out1,hole3)
            if lower_hole!=upper_hole:
                hole1=square(lower_hole//2,.32);hole2=square(upper_hole//2,.32)
                ring(hole1,hole2,True)
                walls.extend([(hole0,hole1,True),(hole2,hole3,True)])
            else:walls.append((hole0,hole3,True))
        else:quad(out0,True);quad(out1)
        for bottom,top,reverse in walls:
            for i in range(4):
                j=(i+1)%4
                quad([bottom[i],bottom[j],top[j],top[i]],reverse)
    edges=collections.Counter(tuple(sorted((t[i],t[(i+1)%3]))) for t in triangles for i in range(3))
    assert set(edges.values())=={2}
    stl=b'WEMAKE3D stock partiallybridge two-layer stepped square hole'.ljust(80,b'\0')+struct.pack('<I',len(triangles))
    for a,b,c in triangles:
        p,q,r=(vertices[i] for i in (a,b,c))
        u=[q[i]-p[i] for i in range(3)];v=[r[i]-p[i] for i in range(3)]
        normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        assert sum(n*n for n in normal)>0
        stl+=struct.pack('<12fH',*normal,*p,*q,*r,0)
    return stl,{'vertices':len(vertices),'triangles':len(triangles),'edge_use_counts':[2]}

rows=[]
for name,lower,upper,height,centers in [('stepped-hole',8,4,.52,[(150,150)]),('first-layer-hole',8,8,.32,[(150,150)]),('straight-hole',8,8,.52,[(150,150)]),('solid-plate',0,0,.52,[(150,150)]),('two-islands',8,4,.52,[(128,150),(172,150)])]:
    data,summary=mesh(lower,upper,height,centers)
    digest=hashlib.sha256(data).hexdigest()
    if name=='stepped-hole':assert digest=='1274c56d97fd205740505d88f077971c7ffe79f1552df78a4e7432de557e465e'
    (ROOT/(name+'.stl')).write_bytes(data)
    rows.append({'name':name,'sha256':digest,'lower_hole_mm':lower,'upper_hole_mm':upper,'height_mm':height,'centers':centers,**summary})
(ROOT/'geometry-manifest.json').write_text(json.dumps({'provenance':'frozen before RED Native runs; unmodified stock profiles except separate None control','geometry':rows},indent=2)+'\n')
print(json.dumps(rows,indent=2))
