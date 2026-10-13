"""Authored layer-range E2E inputs, not Native captures."""
import copy
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET
import zipfile

DEST = Path(__file__).parent
BASE = Path('tests/parity/object-layer-profiles/absent.3mf')
CORE = 'http://schemas.microsoft.com/3dmanufacturing/core/2015/02'
PROD = 'http://schemas.microsoft.com/3dmanufacturing/production/2015/06'
ET.register_namespace('', CORE)
ET.register_namespace('p', PROD)
ET.register_namespace('BambuStudio', 'http://schemas.bambulab.com/package/2021')
manifest = {'sources': {}, 'variants': {}}


def read(path):
    manifest['sources'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
    with zipfile.ZipFile(path) as archive:
        return {name: archive.read(name) for name in archive.namelist()}


def xml(root):
    return ET.tostring(root, encoding='utf-8', xml_declaration=True)


def ranges(items):
    root = ET.Element('objects')
    obj = ET.SubElement(root, 'object', {'id': '1'})
    for low, high, options in items:
        interval = ET.SubElement(obj, 'range', {'min_z': str(low), 'max_z': str(high)})
        for key, value in options.items():
            ET.SubElement(interval, 'option', {'opt_key': key}).text = str(value)
    return xml(root)


def write(label, members, baseline=None, contract=None):
    path = DEST / f'{label}.3mf'
    with zipfile.ZipFile(path, 'w') as archive:
        for name, payload in members.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 10, 13, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, payload)
    manifest['variants'][label] = {
        'input_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'baseline': baseline, 'contract': contract,
    }


members = read(BASE)
leaf_name = '3D/Objects/boundary.stl_1.model'
leaf = ET.fromstring(members[leaf_name])
mesh = leaf.find(f'.//{{{CORE}}}mesh')
mesh.clear()
vertices = ET.SubElement(mesh, f'{{{CORE}}}vertices')
for x, y, z in [(-10,-10,-3),(10,-10,-3),(10,10,-3),(-10,10,-3),
                (-10,-10,3),(10,-10,3),(10,10,3),(-10,10,3)]:
    ET.SubElement(vertices, f'{{{CORE}}}vertex', {'x': str(x), 'y': str(y), 'z': str(z)})
triangles = ET.SubElement(mesh, f'{{{CORE}}}triangles')
for a,b,c in [(0,2,1),(0,3,2),(4,5,6),(4,6,7),(0,1,5),(0,5,4),
              (1,2,6),(1,6,5),(2,3,7),(2,7,6),(3,0,4),(3,4,7)]:
    ET.SubElement(triangles, f'{{{CORE}}}triangle', {'v1':str(a),'v2':str(b),'v3':str(c)})
members[leaf_name] = xml(leaf)
root = ET.fromstring(members['3D/3dmodel.model'])
root.find(f'.//{{{CORE}}}component').set('transform', '1 0 0 0 1 0 0 0 1 0 0 3')
members['3D/3dmodel.model'] = xml(root)
settings = ET.fromstring(members['Metadata/model_settings.config'])
for node in settings.findall('.//metadata'):
    if node.get('key') == 'matrix':
        node.set('value', '1 0 0 0 0 1 0 0 0 0 1 3 0 0 0 1')
    if node.get('key') == 'source_offset_z': node.set('value', '3')
members['Metadata/model_settings.config'] = xml(settings)
config = json.loads(members['Metadata/project_settings.config'])
config.update({'wall_generator':'classic','wall_loops':'2','sparse_infill_density':'20%',
               'sparse_infill_pattern':'rectilinear','top_shell_layers':'2','bottom_shell_layers':'2'})
members['Metadata/project_settings.config'] = json.dumps(config).encode()
write('baseline', members, contract='Authored 20x20x6 closed cube; existing project presets; rectilinear20%, 2 walls/shells')
for label, key, value in [('density','sparse_infill_density','40%'),
                           ('width','outer_wall_line_width','0.5'),
                           ('speed','outer_wall_speed','35')]:
    doc = ranges([(1,3,{'layer_height':'0.2',key:value})])
    write(label, dict(members, **{'Metadata/layer_config_ranges.xml':doc}), 'baseline', doc.decode())

profile = b'object_id=1|0;0.3;0.3;0.15;3;0.15;3;0.2;6;0.2\n'
profiled = dict(members, **{'Metadata/layer_heights_profile.txt':profile})
write('profile-baseline', profiled, contract=profile.decode())
scenarios = {
    'profile-mixed': [(1,2,{'layer_height':'0.2','sparse_infill_density':'40%'}),
                      (3,4,{'outer_wall_speed':'35'})],
    'profile-region-only': [(1,3,{'sparse_infill_density':'40%'})],
    'profile-overlap': [(1,3,{'layer_height':'0.2','sparse_infill_density':'40%'}),
                       (2,4,{'layer_height':'0.2','outer_wall_speed':'35'})],
    'profile-gap': [(1,2,{'layer_height':'0.2','sparse_infill_density':'40%'}),
                   (3,4,{'layer_height':'0.2','outer_wall_speed':'35'})],
    'profile-epsilon': [(0.825,1.12495,{'layer_height':'0.2','outer_wall_speed':'35'}),
                       (1.125,2,{'layer_height':'0.2','sparse_infill_density':'40%'})],
}
for label, items in scenarios.items():
    doc = ranges(items)
    write(label, dict(profiled, **{'Metadata/layer_config_ranges.xml':doc}), 'profile-baseline', doc.decode())

conflict = copy.deepcopy(settings)
part = conflict.find('object/part')
part.insert(list(part).index(part.find('mesh_stat')),
            ET.Element('metadata', {'key':'outer_wall_line_width','value':'0.6'}))
volume = dict(members, **{'Metadata/model_settings.config':xml(conflict)})
write('volume-baseline', volume, contract='Model-part outer_wall_line_width=0.6')
doc = ranges([(1,3,{'layer_height':'0.2','outer_wall_line_width':'0.5'})])
write('volume-precedence', dict(volume, **{'Metadata/layer_config_ranges.xml':doc}), 'volume-baseline', doc.decode())

modifier_path = Path('tests/parity/validation/modifier-wall-loops-classic.3mf')
modifier = read(modifier_path)
modifier.pop('Metadata/layer_config_ranges.xml', None)
write('modifier-baseline', modifier, contract='Existing tracked modifier project; no geometry/config changes')
doc = ranges([(0.3,0.6,{'layer_height':'0.2','outer_wall_speed':'35'})])
write('modifier-range', dict(modifier, **{'Metadata/layer_config_ranges.xml':doc}), 'modifier-baseline', doc.decode())

(DEST/'provenance.json').write_text(json.dumps(manifest,indent=2)+'\n')
