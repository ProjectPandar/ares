"""Extrude an independently defined ledge with a short near-collinear step."""
import hashlib
import json
import pathlib
import xml.etree.ElementTree as ET
import zipfile

ROOT = pathlib.Path(__file__).parent
TEMPLATE = ROOT.parent / 'mesh-edge-pairing/cube-control.3mf'
NS = 'http://schemas.microsoft.com/3dmanufacturing/core/2015/02'
ET.register_namespace('', NS)
ET.register_namespace('p', 'http://schemas.microsoft.com/3dmanufacturing/production/2015/06')
# Millimetres; the short step is independently constructed, not a model extract.
points = [(0, 0), (9, -0.08), (10, -0.002), (10.0052, 0), (20, 0), (20, 10), (0, 10)]
height = 0.8
vertices = [(x, y, z) for z in (0, height) for x, y in points]
triangles = []
n = len(points)

def cross(a, b, c):
    return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])

remaining = list(range(n))
while len(remaining) > 3:
    for k, b in enumerate(remaining):
        a, c = remaining[k-1], remaining[(k+1) % len(remaining)]
        if cross(points[a], points[b], points[c]) <= 0:
            continue
        if any(all(cross(points[u], points[v], points[p]) >= 0 for u, v in ((a,b),(b,c),(c,a)))
               for p in remaining if p not in (a,b,c)):
            continue
        triangles.extend(((a+n, b+n, c+n), (c,b,a)))
        remaining.pop(k)
        break
    else:
        raise ValueError('no ear in synthetic polygon')
a, b, c = remaining
triangles.extend(((a+n,b+n,c+n),(c,b,a)))
for a in range(n):
    b = (a+1) % n
    triangles.extend(((a,b,b+n),(a,b+n,a+n)))
with zipfile.ZipFile(TEMPLATE) as archive:
    members = {name: archive.read(name) for name in archive.namelist()}
config = json.loads(members['Metadata/project_settings.config'])
config.update({'wall_generator':'arachne','wall_loops':'1','layer_height':'0.2',
               'initial_layer_print_height':'0.2','top_shell_layers':'0',
               'bottom_shell_layers':'0','sparse_infill_density':'0%',
               'resolution':'0.001','wall_maximum_resolution':'0.5','wall_maximum_deviation':'0.05'})
members['Metadata/project_settings.config'] = (json.dumps(config, indent=2, ensure_ascii=False)+'\n').encode()
model_name = '3D/Objects/cube10.stl_1.model'
model = ET.fromstring(members[model_name])
obj = model.find(f'{{{NS}}}resources/{{{NS}}}object')
obj.remove(obj.find(f'{{{NS}}}mesh'))
mesh = ET.SubElement(obj,f'{{{NS}}}mesh')
vs = ET.SubElement(mesh,f'{{{NS}}}vertices')
for p in vertices:
    ET.SubElement(vs,f'{{{NS}}}vertex',{k:str(v) for k,v in zip(('x','y','z'),p)})
ts = ET.SubElement(mesh,f'{{{NS}}}triangles')
for t in triangles:
    ET.SubElement(ts,f'{{{NS}}}triangle',{k:str(v) for k,v in zip(('v1','v2','v3'),t)})
members[model_name] = ET.tostring(model,encoding='utf-8',xml_declaration=True)
root_model = ET.fromstring(members['3D/3dmodel.model'])
root_model.find(f'{{{NS}}}build/{{{NS}}}item').set('transform','1 0 0 0 1 0 0 0 1 100 100 0')
members['3D/3dmodel.model'] = ET.tostring(root_model,encoding='utf-8',xml_declaration=True)
output = ROOT/'short-step.3mf'
with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as archive:
    for name,data in sorted(members.items()):
        info = zipfile.ZipInfo(name,(1980,1,1,0,0,0))
        info.external_attr = 0o644 << 16
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info,data)
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = {'template':str(TEMPLATE.relative_to(ROOT.parent)), 'template_sha256':sha(TEMPLATE.read_bytes()),
            'archive_sha256':sha(output.read_bytes()),'polygon_mm':points,'height_mm':height,
            'vertex_count':len(vertices),'triangle_count':len(triangles),
            'members_sha256':{name:sha(data) for name,data in sorted(members.items())}}
(ROOT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(output.name,manifest['archive_sha256'])
