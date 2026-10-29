"""Input-only fixtures: TriangleMesh.cpp:520-613; SPEC49c31802f/addendum8b4162da3."""
from pathlib import Path
import json,zipfile,re,hashlib,tempfile,subprocess,datetime,os
from collections import Counter
repo=Path(__file__).resolve().parents[3];root=Path(__file__).parent; base=repo/'tests/parity/orca_cli_ender3.3mf'
with zipfile.ZipFile(base) as z:original={n:z.read(n) for n in z.namelist()}
faces=[(0,2,1),(1,2,3),(4,5,6),(5,7,6),(0,1,4),(1,5,4),(2,6,3),(3,6,7),(0,4,2),(2,4,6),(1,3,5),(3,7,5)]
def geometry(shared):
 vertices=[];triangles=[];lookup={}
 for x,y in [(0,0),(10,10)]:
  points=[(x,y,0),(x+10,y,0),(x,y+10,0),(x+10,y+10,0),(x,y,2),(x+10,y,2),(x,y+10,2),(x+10,y+10,2)];ids=[]
  for p in points:
   if shared and p in lookup:i=lookup[p]
   else:i=len(vertices);vertices.append(p);lookup[p]=i
   ids.append(i)
  triangles.extend(tuple(ids[i] for i in f) for f in faces)
 return vertices,triangles
inputs={}
for label,shared,order in [('shared-original', True, 'original'), ('split-original', False, 'original'), ('shared-reverse', True, 'reverse'), ('shared-interleaved', True, 'interleave'), ('shared-rotated', True, 'rotate'), ('cube-control', False, 'original'), ('odd-original', True, 'original'), ('six-original', True, 'original'), ('six-reverse', True, 'reverse'), ('six-rotated', True, 'rotate'), ('arachne-shared', True, 'original'), ('arachne-split', False, 'original'), ('same-direction', True, 'original'), ('odd-reverse', True, 'reverse'), ('odd-rotated', True, 'rotate')]:
 v,t=geometry(shared)
 if label=='same-direction':v=v[:8];t=t[:12]+[t[10],t[10]]
 elif label=='cube-control':v=v[:8];t=t[:12]
 elif label.startswith('odd-'):
  v=v[:8]+[(20,15,0)];t=t[:12]+[(3,7,8)]
 elif label.startswith('six-'):t=t+t[:12]
 elif label=='collapsed-diagnostic':t=t+[(0,0,0)]
 if order=='reverse':t=t[::-1]
 elif order=='interleave':t=[a for pair in zip(t[:12],t[12:]) for a in pair]
 elif order=='rotate':t=t[7:]+t[:7]
 counts=Counter(tuple(sorted((f[i],f[(i+1)%3]))) for f in t for i in range(3));high={str(k):c for k,c in counts.items() if c>2}
 assert bool(high)==shared
 files=original.copy();name='3D/Objects/cube10.stl_1.model';s=files[name].decode();mesh='<mesh><vertices>'+''.join(f'<vertex x="{x}" y="{y}" z="{z}"/>' for x,y,z in v)+'</vertices><triangles>'+''.join(f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a,b,c in t)+'</triangles></mesh>';files[name]=re.sub(r'<mesh>[\s\S]*?</mesh>',mesh,s).encode()
 s=files['3D/3dmodel.model'].decode();s=re.sub(r'(<component\s[^>]*?) transform="[^"]+"',r'\1 transform="1 0 0 0 1 0 0 0 1 0 0 0"',s);s=re.sub(r'(<item\s[^>]*?) transform="[^"]+"',r'\1 transform="1 0 0 0 1 0 0 0 1 110 110 0"',s);files['3D/3dmodel.model']=s.encode()
 s=files['Metadata/model_settings.config'].decode();s=re.sub(r'(<metadata key="matrix" value=")[^"]+',r'\g<1>1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1',s);files['Metadata/model_settings.config']=s.encode()
 cfg=json.loads(files['Metadata/project_settings.config']);cfg.update(wall_generator='classic',skirt_loops='0',brim_type='no_brim',brim_width='0',enable_support='0',enable_prime_tower='0',timelapse_type='0',seam_position='aligned',print_sequence='by layer',sparse_infill_pattern='rectilinear',sparse_infill_density='15%');cfg['wall_generator']='arachne' if label.startswith('arachne-') else 'classic';files['Metadata/project_settings.config']=json.dumps(cfg,sort_keys=True).encode()
 p=root/(label+'.3mf')
 with zipfile.ZipFile(p,'w') as z:
  for n,b in files.items():
   assert not n.endswith('.gcode');i=zipfile.ZipInfo(n,(2026,1,1,0,0,0));i.compress_type=zipfile.ZIP_DEFLATED;z.writestr(i,b)
 inputs[label]={'file':p.name, 'native_result':-100 if label in ['odd-reverse','odd-rotated'] else 0, 'classification':'diagnostic-only' if label in ['odd-reverse','odd-rotated'] else 'strict-parity','sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'vertices':len(v),'triangles':len(t),'high_valence':high,'face_order':order}
manifest={"base_main":"de1211be5067d0eb6f709602a1299a0854c89145","base_input":"tests/parity/orca_cli_ender3.3mf","base_sha256":hashlib.sha256(base.read_bytes()).hexdigest(),"source":"TriangleMesh.cpp:520-613; TriangleMeshSlicer.cpp:1865-1889","cases":inputs}
(root/"provenance.json").write_text(json.dumps(manifest,indent=2)+"\n")
