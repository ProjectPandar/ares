#!/usr/bin/env python3
"""Reproduce SPEC5189 frozen conditional-scarf E2E inputs, no Native invocation."""
from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).parent
BASE=ROOT.parent/'seam-rear/control.3mf'
COMMON=dict(seam_slope_type='external',seam_slope_conditional='0',scarf_angle_threshold='0',scarf_overhang_threshold='0%',seam_slope_steps='10',seam_slope_min_length='10',seam_slope_start_height='10%',seam_slope_inner_walls='0',seam_slope_entire_loop='0')
CASES={'control':{},'positive':dict(seam_slope_conditional='1'),'sharp':dict(seam_slope_conditional='1',scarf_angle_threshold='155'),'overhang-low':dict(seam_slope_conditional='1',scarf_overhang_threshold='1%'),'overhang-high':dict(seam_slope_conditional='1',scarf_overhang_threshold='100%')}
sha=lambda b:hashlib.sha256(b).hexdigest()
rows={}
for label,delta in CASES.items():
 settings=dict(COMMON,**delta)
 with zipfile.ZipFile(BASE) as z,zipfile.ZipFile(ROOT/f'{label}.3mf','w') as out:
  for info in z.infolist():
   b=z.read(info.filename)
   if info.filename=='Metadata/project_settings.config':
    config=json.loads(b);config.update(settings);b=(json.dumps(config,separators=(',',':'))+'\n').encode()
   if label.startswith('overhang-') and info.filename=='3D/Objects/overhang.stl_1.model':
    text=b.decode()
    for old,new in [('<vertex x="-5" y="-5" z="-6.7"/>','<vertex x="-5.75" y="-5.5" z="-6.7"/>'),('<vertex x="5" y="-5" z="-6.7"/>','<vertex x="5.75" y="-5.5" z="-6.7"/>'),('<vertex x="0" y="5" z="-6.7"/>','<vertex x="0" y="6" z="-6.7"/>')]:
     assert text.count(old)==1;text=text.replace(old,new)
    b=text.encode()
   out.writestr(info,b)
 path=ROOT/f'{label}.3mf'
 with zipfile.ZipFile(path) as z:
  rows[label]=dict(sha256=sha(path.read_bytes()),zip_entry_sha256={i.filename:sha(z.read(i.filename)) for i in z.infolist()},settings=settings,native_slope_count=0 if label in ['sharp','overhang-low'] else 48,native_first_slope=None if label in ['sharp','overhang-low'] else 'G1 X109.72 Y'+('114.313' if label=='overhang-high' else '113.938')+' Z.23 E.0027 ; perimeter')
expected={'control':'0e2448b51db7003392c83e123aed4d1340d49eecbee2b725c4ca5bf12fd842b1','positive':'b89abfb6df717f88a513723f469c966577c7361d789f623dcd3e9feddca4849d','overhang-low':'0cfebc322260c4ebfec8c9d1cac3cfd3f86c97055fbe722a187e31295c49d235','overhang-high':'3884be1b8d08fba07b1216613ee1152ad2e2101e64ab633f123352c852599ec3'}
for label,digest in expected.items():assert rows[label]['sha256']==digest
manifest=dict(spec='5189e53a586ccffde246c3e798b8563aac0f97e9',upstream='8500fcdccaa10b5099ac20d252af3a7c560046f1',base_sha256=sha(BASE.read_bytes()),cases=rows)
(ROOT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
