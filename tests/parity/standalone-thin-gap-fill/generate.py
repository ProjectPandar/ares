"""Synthetic square frame and solid block, with two loaded PLA filaments."""
import decimal
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
vertices = []
indices = {}
triangles = []


def quad(points):
    ids = []
    for point in points:
        if point not in indices:
            indices[point] = len(vertices)
            vertices.append(point)
        ids.append(indices[point])
    triangles.extend(((ids[0], ids[1], ids[2]), (ids[0], ids[2], ids[3])))


# Coordinates are integer hundredths of a millimetre.
outer = [(0, 0), (2000, 0), (2000, 2000), (0, 2000)]
inner = [(120, 120), (1880, 120), (1880, 1880), (120, 1880)]
height = 240
for i in range(4):
    j = (i + 1) % 4
    top = [(*outer[i], height), (*outer[j], height), (*inner[j], height), (*inner[i], height)]
    quad(top)
    quad([(*point[:2], 0) for point in reversed(top)])
    quad([(*outer[i], 0), (*outer[j], 0), (*outer[j], height), (*outer[i], height)])
    quad([(*inner[i], 0), (*inner[i], height), (*inner[j], height), (*inner[j], 0)])
box = [(2500, 200), (4100, 200), (4100, 1800), (2500, 1800)]
quad([(*point, height) for point in box])
quad([(*point, 0) for point in reversed(box)])
for i in range(4):
    j = (i + 1) % 4
    quad([(*box[i], 0), (*box[j], 0), (*box[j], height), (*box[i], height)])

with zipfile.ZipFile(TEMPLATE) as archive:
    members = {name: archive.read(name) for name in archive.namelist()}
config = json.loads(members['Metadata/project_settings.config'])
for key, value in list(config.items()):
    if isinstance(value, list) and len(value) == 1 and (
        key.startswith('filament_') or key.startswith('nozzle_temperature')
        or key.endswith('_temp') or key.endswith('_temp_initial_layer')
    ):
        config[key] = value * 2
for key in ('inherits_group', 'different_settings_to_system'):
    config[key].append(config[key][-1])
config.update({
    'filament_colour': ['#B87333', '#00A0FF'], 'filament_map': ['1', '1'],
    'flush_volumes_matrix': ['0', '280', '280', '0'],
    'flush_volumes_vector': ['140', '140', '140', '140'],
    'wall_generator': 'classic', 'wall_loops': '2',
    'outer_wall_filament_id': '1', 'inner_wall_filament_id': '1',
    'internal_solid_filament_id': '1', 'top_surface_filament_id': '1',
    'bottom_surface_filament_id': '1', 'sparse_infill_filament_id': '2',
    'layer_height': '0.2', 'initial_layer_print_height': '0.2',
    'bottom_shell_layers': '3', 'top_shell_layers': '3',
    'sparse_infill_density': '15%', 'sparse_infill_pattern': 'rectilinear',
    'outer_wall_line_width': '0.45', 'inner_wall_line_width': '0.45',
    'internal_solid_infill_line_width': '0.45', 'sparse_infill_line_width': '0.45',
    'enable_prime_tower': '0',
})
members['Metadata/project_settings.config'] = (json.dumps(config, indent=2, ensure_ascii=False) + '\n').encode()
model_name = '3D/Objects/cube10.stl_1.model'
model = ET.fromstring(members[model_name])
obj = model.find(f'{{{NS}}}resources/{{{NS}}}object')
obj.remove(obj.find(f'{{{NS}}}mesh'))
mesh = ET.SubElement(obj, f'{{{NS}}}mesh')
vs = ET.SubElement(mesh, f'{{{NS}}}vertices')
for point in vertices:
    ET.SubElement(vs, f'{{{NS}}}vertex', {key: str(decimal.Decimal(value) / 100) for key, value in zip(('x', 'y', 'z'), point)})
ts = ET.SubElement(mesh, f'{{{NS}}}triangles')
for triangle in triangles:
    ET.SubElement(ts, f'{{{NS}}}triangle', {key: str(value) for key, value in zip(('v1', 'v2', 'v3'), triangle)})
members[model_name] = ET.tostring(model, encoding='utf-8', xml_declaration=True)
root_model = ET.fromstring(members['3D/3dmodel.model'])
root_model.find(f'{{{NS}}}build/{{{NS}}}item').set('transform', '1 0 0 0 1 0 0 0 1 90 90 0')
members['3D/3dmodel.model'] = ET.tostring(root_model, encoding='utf-8', xml_declaration=True)
settings = ET.fromstring(members['Metadata/model_settings.config'])
plate = settings.find('plate')
ET.SubElement(plate, 'metadata', {'key': 'filament_maps', 'value': '1 1'})
members['Metadata/model_settings.config'] = ET.tostring(settings, encoding='utf-8', xml_declaration=True)
output = ROOT / 'frame-and-block.3mf'
with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for name, data in sorted(members.items()):
        info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
        info.external_attr = 0o644 << 16
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info, data)
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = {'template': str(TEMPLATE.relative_to(ROOT.parent)), 'template_sha256': sha(TEMPLATE.read_bytes()), 'archive_sha256': sha(output.read_bytes()), 'vertex_count': len(vertices), 'triangle_count': len(triangles), 'members_sha256': {name: sha(data) for name, data in sorted(members.items())}}
(ROOT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(output.name, manifest['archive_sha256'], len(vertices), len(triangles))
