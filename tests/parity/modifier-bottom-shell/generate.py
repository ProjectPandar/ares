"""Input-only modifier bottom-shell fixtures; SPEC 7f0717e07."""
from pathlib import Path
import zipfile,json,re,hashlib,sys
repo=Path(__file__).resolve().parents[3];out=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).parent;out.mkdir(parents=True,exist_ok=True)
manifest={'base_main':'73e8e712e','source':'PrintApply.cpp:1032-1088; PrintObject.cpp:3664-3710/2252-2280/3973-3995','cases':{}}
for backend in ['classic','arachne']:
 base=repo/f'tests/parity/validation/modifier-wall-loops-{backend}.3mf'
 with zipfile.ZipFile(base) as z:original={n:z.read(n) for n in z.namelist()}
 variants=[('none',None),('zero',0),('one',1),('four',4),('equal',2),('outside',4),('overlap',4)]
 if backend=='classic':variants += [('horizontal-none',None),('horizontal-one',1),('horizontal-four',4)]
 for label,count in variants:
  files=original.copy();settings=json.loads(files['Metadata/project_settings.config']);settings.update(bottom_shell_layers='2',bottom_shell_thickness='0',sparse_infill_pattern='rectilinear',sparse_infill_density='15%',ensure_vertical_shell_thickness='none' if label.startswith('horizontal') else 'ensure_all')
  files['Metadata/project_settings.config']=json.dumps(settings,sort_keys=True).encode()
  meta=files['Metadata/model_settings.config'].decode();meta=meta.replace('<metadata key="wall_loops" value="4"/>','' if count is None else f'<metadata key="bottom_shell_layers" value="{count}"/>')
  if label=='outside':
   meta=meta.replace('120.5','150.5');files['3D/3dmodel.model']=files['3D/3dmodel.model'].decode().replace('120.5','150.5').encode()
  if label=='overlap':
   files['3D/Objects/top-overlap.model']=files['3D/Objects/epsilon-modifier.model'].decode().replace('id="3"','id="4"').encode()
   s=files['3D/3dmodel.model'].decode();s=s.replace('</components>','<component p:path="/3D/Objects/top-overlap.model" objectid="4" transform="0.25 0 0 0 1 0 0 0 1 121.75 138 5"/></components>');files['3D/3dmodel.model']=s.encode()
   meta=meta.replace('</object>','<part id="4" subtype="modifier_part"><metadata key="name" value="top-overlap"/><metadata key="matrix" value="0.25 0 0 121.75 0 1 0 138 0 0 1 5 0 0 0 1"/><metadata key="bottom_shell_layers" value="1"/></part></object>')
   n='3D/_rels/3dmodel.model.rels';files[n]=files[n].decode().replace('</Relationships>','<Relationship Target="/3D/Objects/top-overlap.model" Id="top-overlap" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>').encode()
  files['Metadata/model_settings.config']=meta.encode();name=f'{backend}-{label}';p=out/(name+'.3mf')
  with zipfile.ZipFile(p,'w') as z:
   for n,b in files.items():
    assert not n.endswith('.gcode');i=zipfile.ZipInfo(n,(2026,1,1,0,0,0));i.compress_type=zipfile.ZIP_DEFLATED;z.writestr(i,b)
  manifest['cases'][name]={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'base':str(base.relative_to(repo)),'base_sha256':hashlib.sha256(base.read_bytes()).hexdigest(),'bottom':count,'parent_bottom':2,'horizontal':label.startswith('horizontal'),'outside':label=='outside','overlap_later_bottom':1 if label=='overlap' else None}
(out/'provenance.json').write_text(json.dumps(manifest,indent=2)+'\n')
