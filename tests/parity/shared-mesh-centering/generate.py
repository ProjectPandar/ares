"""Input-only bbs_3mf.cpp4867-5052 fixtures, approved C2 SPEC5d0f9f2d9."""
from pathlib import Path
from hashlib import sha256
from xml.sax.saxutils import escape
import json,re,zipfile

ROOT=Path(__file__).parent
BASE=Path('tests/parity/orca_cli_ender3.3mf')
I='1 0 0 0 1 0 0 0 1'
R='0 1 0 -1 0 0 0 0 1'
S='1.25 0 0 0 1 0 0 0 1'
CASES={
 'implicit-shared':{'second_id':1},
 'distinct-mesh':{},
 'explicit-shared':{'shared':'1'},
 'unseen-key':{'shared':'77'},
 'minus-one-loaded':{'second_id':1,'shared':'-1'},
 'minus-one-fresh':{'shared':'-1'},
 'negative-key':{'first_shared':'-2','shared':'-2'},
 'signed-prefix':{'shared':' +01suffix'},
 'zero-no-digits':{'first_shared':'0','shared':'not-an-integer'},
 'reversed-build':{'second_id':1,'reverse':True},
 'different-owner-shift':{'shared':'1','second_center':(-11,9)},
 'fresh-rotation':{'second_transform':R},
 'shared-rotation':{'shared':'1','second_transform':R},
 'fresh-scale':{'second_transform':S},
 'shared-scale':{'shared':'1','second_transform':S},
 'same-path-repeat':{'repeat':True},
 'nested-bfs':{'nested':True,'second_id':1},
 'volume-settings':{'shared':'1','wall_loops':'4'},
 'facet-isolation':{'shared':'1','facets':True},
 'facet-control':{'facets':True},
 'plate-local':{'second_id':1,'two_plates':True},
}
with zipfile.ZipFile(BASE) as z:base={n:z.read(n) for n in z.namelist()}
meshname='3D/Objects/cube10.stl_1.model'
secondname='3D/Objects/second.model'
object_xml=re.search(r'<object id="2"[\s\S]*?</object>',base['3D/3dmodel.model'].decode())[0]
object_meta=re.search(r'<object id="2">[\s\S]*?</object>',base['Metadata/model_settings.config'].decode())[0]
part_meta=re.search(r'<part[\s\S]*?</part>',object_meta)[0]
plate_xml=re.search(r'<plate>[\s\S]*?</plate>',base['Metadata/model_settings.config'].decode())[0]

def mesh(width,center,mid,height=1,paint=None):
 def vertex(m):
  x,y,z=map(float,m.groups())
  taper=.6 if height==5 and z<0 else 1
  return '<vertex x="%.9g" y="%.9g" z="%.9g"/>'%(x*width/10*taper+center[0],y*.6+center[1],z*.1*height)
 xml=re.sub(r'<vertex x="([^"]+)" y="([^"]+)" z="([^"]+)"/>',vertex,base[meshname].decode())
 if paint:xml=re.sub(r'(<triangle\s+[^>]+?)/>',rf'\1 paint_supports="{paint}"/>',xml)
 return xml.replace('id="1"',f'id="{mid}"').replace('00010000-',f'{mid:08x}-').encode()

def part(mid,name,key=None,wall_loops=None):
 xml=part_meta.replace('part id="1"',f'part id="{mid}"').replace('cube10.stl',name)
 extra=''
 if key is not None:extra+=f'<metadata key="mesh_shared" value="{escape(key)}"/>'
 if wall_loops is not None:extra+=f'<metadata key="wall_loops" value="{wall_loops}"/>'
 return xml.replace('<mesh_stat',extra+'<mesh_stat')

def component(path,mid,transform=I,x=0):
 return f'<component p:path="/{path}" objectid="{mid}" transform="{transform} {x} 0 0"/>'

def object_resource(oid,components):
 return f'<object id="{oid}" type="model"><components>{components}</components></object>'

def object_settings(oid,parts):
 return f'<object id="{oid}"><metadata key="name" value="object-{oid}"/><metadata key="extruder" value="1"/>{parts}</object>'

def plate(pid,oid):
 xml=plate_xml.replace('key="plater_id" value="1"',f'key="plater_id" value="{pid}"')
 return xml.replace('key="object_id" value="2"',f'key="object_id" value="{oid}"').replace('key="identify_id" value="14"',f'key="identify_id" value="{12+oid}"')

manifest={'spec':'5d0f9f2d9','source':'bbs_3mf.cpp4867-5052; Model.cpp1241-1307/2600-2617/2821-2824','base':str(BASE),'base_sha256':sha256(BASE.read_bytes()).hexdigest(),'cases':{}}
for label,spec in CASES.items():
 files=base.copy();settings=json.loads(files['Metadata/project_settings.config'])
 settings.update(wall_generator='classic',skirt_loops='0',brim_type='no_brim',brim_width='0',enable_support='0',enable_prime_tower='0',timelapse_type='0',seam_position='aligned',print_sequence='by layer')
 if spec.get('facets'):settings.update(enable_support='1',support_type='normal(manual)',support_style='grid')
 files['Metadata/project_settings.config']=json.dumps(settings,sort_keys=True).encode()
 mid=spec.get('second_id',4)
 height=5 if spec.get('facets') else 1
 files[meshname]=mesh(10,(17,-3),1,height,'8' if spec.get('facets') else None)
 files[secondname]=mesh(16,spec.get('second_center',(17,-3)),mid,height,'4' if spec.get('facets') else None)
 first=component(meshname,1);second=component(secondname,mid,spec.get('second_transform',I))
 firstpart=part(1,'owner-box',spec.get('first_shared'))
 secondpart=part(mid,'current-box',spec.get('shared'),spec.get('wall_loops'))
 resources=object_resource(2,first)+object_resource(3,second)
 metadata=object_settings(2,firstpart)+object_settings(3,secondpart)
 ids=[2,3];volume_counts=[1,1]
 if spec.get('repeat'):
  resources=object_resource(2,first+component(meshname,1,x=45))
  metadata=object_settings(2,firstpart+part(1,'repeat-box'));ids=[2];volume_counts=[2]
 if spec.get('nested'):
  # Literal main/self-path misses Native's empty importer ID namespace.
  resources=object_resource(2,component('3D/3dmodel.model',5)+component(secondname,mid,x=45))+object_resource(5,first)
  metadata=object_settings(2,secondpart+firstpart);ids=[2];volume_counts=[1]
 if spec.get('reverse'):ids.reverse();volume_counts.reverse()
 model=files['3D/3dmodel.model'].decode()
 model=re.sub(r'<resources>[\s\S]*?</resources>',f'<resources>{resources}</resources>',model)
 items=''
 for oid in ids:
  x=110 if oid==2 else 155
  if spec.get('two_plates') and oid==3:x=374
  items+=f'<item objectid="{oid}" transform="{I} {x} 110 {height/2}" printable="1" auto_drop="1"/>'
 files['3D/3dmodel.model']=re.sub(r'<item[^>]+/>',items,model,count=1).encode()
 meta=files['Metadata/model_settings.config'].decode()
 meta=re.sub(r'<object id="2">[\s\S]*?</object>',metadata,meta,count=1)
 if spec.get('two_plates'):
  meta=re.sub(r'<plate>[\s\S]*?</plate>',plate(1,2)+plate(2,3),meta,count=1)
  files['Metadata/filament_sequence.json']=json.dumps({f'plate_{p}':{'nozzle_sequence':[],'optimal_assignment':[],'sequence':[]} for p in [1,2]}).encode()
 else:
  members=''.join(f'<model_instance><metadata key="object_id" value="{oid}"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="{12+oid}"/></model_instance>' for oid in ids)
  meta=re.sub(r'<model_instance>[\s\S]*?</model_instance>',members,meta,count=1)
 files['Metadata/model_settings.config']=meta.encode()
 rel='3D/_rels/3dmodel.model.rels'
 files[rel]=files[rel].decode().replace('</Relationships>',f'<Relationship Target="/{secondname}" Id="rel-2" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>').encode()
 path=ROOT/(label+'.3mf')
 with zipfile.ZipFile(path,'w') as z:
  for name,data in files.items():
   assert not name.endswith('.gcode')
   info=zipfile.ZipInfo(name,(2026,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;z.writestr(info,data)
 manifest['cases'][label]={'input_sha256':sha256(path.read_bytes()).hexdigest(),'parameters':spec,'object_ids':ids,'volume_counts':volume_counts,'plates':[1,2] if spec.get('two_plates') else [1],'settings':{k:settings[k] for k in ['wall_generator','seam_position','enable_support','skirt_loops','brim_type','brim_width']},'native_contract':'strict ordered output in official and seeded'}
(ROOT/'provenance.json').write_text(json.dumps(manifest,sort_keys=True)+'\n')
