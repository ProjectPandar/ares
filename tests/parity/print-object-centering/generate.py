"""Authored input-only PrintObject.cpp93-130 public E2E fixtures."""
from pathlib import Path
from hashlib import sha256
import json,re,zipfile

ROOT=Path(__file__).parent
BASE=Path('tests/parity/orca_cli_ender3.3mf')
I='1 0 0 0 1 0 0 0 1'
R='0 1 0 -1 0 0 0 0 1'
S='2 0 0 0 1 0 0 0 1'
M='-1 0 0 0 1 0 0 0 1'
R30='0.8660254037844386 0.5 0 -0.5 0.8660254037844386 0 0 0 1'
# transform, x, y, z, printable; all instances share one source ModelObject.
A=(I,110,110,.5,1)
B=(R,155,110,.5,1)
CASES={
 'single':([A],{}),
 'xy-copy':([A,(I,155,110,.5,1)],{}),
 'rotation-group':([A,B],{}),
 'rotation-exclusion':([A,B],{'exclude_object':'1'}),
 'scale-group':([A,(S,155,110,.5,1)],{}),
 'mirror-group':([A,(M,155,110,.5,1)],{}),
 'rotation-thirty':([A,(R30,155,110,.5,1)],{}),
 'reversed-source-order':([B,A],{}),
 'nonprintable-first':([(S,110,110,.5,0),A],{}),
 'fractional-shifts':([(I,110.0000007,110.0000003,.5,1),(R,155.0000003,110.0000007,.5,1)],{}),
 'z-shrink':([A,B],{'filament_shrinkage_compensation_z':['98']}),
 'skirt-groups':([A,B],{'skirt_loops':'1','skirt_height':'1'}),
 'brim-groups':([A,B],{'brim_type':'outer_only','brim_width':'3'}),
 'support-groups':([(I,110,110,2.5,1),(R,155,110,2.5,1)],{'enable_support':'1','support_type':'normal(auto)','support_threshold_angle':'60'}),
}
with zipfile.ZipFile(BASE) as z:base={n:z.read(n) for n in z.namelist()}
manifest={'source':'Model.cpp1586-1599; PrintObject.cpp93-130; PrintApply.cpp138-170','spec':'49cf8a57a','base':str(BASE),'base_sha256':sha256(BASE.read_bytes()).hexdigest(),'cases':{}}
for label,(instances,changes) in CASES.items():
 members=base.copy();settings=json.loads(members['Metadata/project_settings.config'])
 settings.update(wall_generator='classic',skirt_loops='0',brim_type='no_brim',brim_width='0',enable_support='0',enable_prime_tower='0',timelapse_type='0',seam_position='aligned',print_sequence='by layer');settings.update(changes)
 members['Metadata/project_settings.config']=json.dumps(settings,sort_keys=True,indent=2).encode()
 name='3D/Objects/cube10.stl_1.model'
 def vertex(match):
  x,y,z=map(float,match.groups());factor=1 if label=='support-groups' else .1
  if label=='support-groups':x*=.6 if z<0 else 1
  return '<vertex x="%.9g" y="%.9g" z="%.9g"/>'%(x+17,y*.6-3,z*factor*.5 if label=='support-groups' else z*factor)
 members[name]=re.sub(r'<vertex x="([^"]+)" y="([^"]+)" z="([^"]+)"/>',vertex,members[name].decode()).encode()
 model=members['3D/3dmodel.model'].decode()
 items=''.join(f'<item objectid="2" transform="{t} {x:.9f} {y:.9f} {z:.9f}" printable="{p}" auto_drop="1"/>' for t,x,y,z,p in instances)
 members['3D/3dmodel.model']=re.sub(r'<item[^>]+/>',items,model,count=1).encode()
 meta=members['Metadata/model_settings.config'].decode()
 for index in range(1,len(instances)):
  meta=meta.replace('</plate>',f'<model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="{index}"/><metadata key="identify_id" value="{14+index}"/></model_instance></plate>')
 members['Metadata/model_settings.config']=meta.encode()
 path=ROOT/(label+'.3mf')
 with zipfile.ZipFile(path,'w') as z:
  for name,data in members.items():
   assert not name.endswith('.gcode')
   info=zipfile.ZipInfo(name,(2026,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;z.writestr(info,data)
 manifest['cases'][label]={'input_sha256':sha256(path.read_bytes()).hexdigest(),'instances':[{'transform':t,'x':x,'y':y,'z':z,'printable':bool(p)} for t,x,y,z,p in instances],'settings':{k:settings[k] for k in ['wall_generator','seam_position','enable_support','skirt_loops','brim_type','brim_width','filament_shrinkage_compensation_z']},'source_objects':1,'source_volumes':1,'native_contract':'strict full ordered generator bytes in both modes'}
(ROOT/'provenance.json').write_text(json.dumps(manifest,sort_keys=True)+'\n')
