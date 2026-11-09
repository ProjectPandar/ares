"""Generate input-only watertight meshes and reproducible 3MF settings pairs.

No Native outputs are read. Run from repository root with Python 3.
"""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).parent
BASE = Path('tests/parity/legacy-brim-ears/O0.3mf')
COMMON = {'wall_generator': 'arachne', 'wall_loops': '2',
          'sparse_infill_density': '10%', 'sparse_infill_pattern': 'rectilinear',
          'top_shell_layers': '0', 'bottom_shell_layers': '0',
          'top_shell_thickness': '0', 'bottom_shell_thickness': '0',
          'only_one_wall_top': '0', 'only_one_wall_first_layer': '0',
          'detect_overhang_wall': '0', 'fuzzy_skin': 'disabled_fuzzy',
          'enable_support': '0', 'enable_prime_tower': '0',
          'single_extruder_multi_material': '0', 'spiral_mode': '0',
          'brim_type': 'no_brim', 'brim_width': '0', 'brim_ears': '0',
          'brim_use_efc_outline': '0', 'elefant_foot_compensation': '0',
          'skirt_loops': '0', 'skirt_height': '0',
          'layer_height': '0.2', 'initial_layer_print_height': '0.2'}
CASES = [('prism', {}, 'prism'),
         ('density-zero', {'sparse_infill_density': '0%'}, 'prism'),
         ('density-half', {'sparse_infill_density': '0.5%'}, 'prism'),
         ('density-one', {'sparse_infill_density': '1%'}, 'prism'),
         ('single-top', {'only_one_wall_top': '1'}, 'prism'),
         ('stepped-top', {'only_one_wall_top': '1', 'sparse_infill_density': '1%'}, 'step'),
         ('single-first', {'only_one_wall_first_layer': '1'}, 'prism'),
         ('zero-walls', {'wall_loops': '0'}, 'prism'),
         ('two-islands', {}, 'islands'),
         ('spiral', {'spiral_mode': '1', 'wall_loops': '1',
                     'sparse_infill_density': '0%',
                     'bottom_shell_layers': '2',
                     'bottom_shell_thickness': '0.4',
                     'filament_retract_when_changing_layer': ['0']}, 'prism')]


def mesh(shape):
    xs = [-15, -7.5, 7.5, 15] if shape == 'step' else [-15, 15]
    if shape == 'islands':
        xs = [-21, -3, 3, 21]
    ys = [-15, -7.5, 7.5, 15] if shape == 'step' else [-15, 15]
    zs = [-0.6, 0.2, 0.6]
    cells = {(i, j, k) for i in range(len(xs)-1)
             for j in range(len(ys)-1) for k in range(len(zs)-1)
             if (shape != 'islands' or i != 1)
             and (shape != 'step' or k == 0 or (i == 1 and j == 1))}
    vertices, indices, triangles = [], {}, []

    def vertex(point):
        if point not in indices:
            indices[point] = len(vertices)
            vertices.append(point)
        return indices[point]

    def quad(points):
        a, b, c, d = [vertex(p) for p in points]
        triangles.extend([(a, b, c), (a, c, d)])

    for i, j, k in sorted(cells):
        x, X, y, Y, z, Z = xs[i], xs[i+1], ys[j], ys[j+1], zs[k], zs[k+1]
        faces = [((i, j, k-1), [(x,Y,z), (X,Y,z), (X,y,z), (x,y,z)]),
                 ((i, j, k+1), [(x,y,Z), (X,y,Z), (X,Y,Z), (x,Y,Z)]),
                 ((i, j-1, k), [(x,y,z), (X,y,z), (X,y,Z), (x,y,Z)]),
                 ((i+1, j, k), [(X,y,z), (X,Y,z), (X,Y,Z), (X,y,Z)]),
                 ((i, j+1, k), [(X,Y,z), (x,Y,z), (x,Y,Z), (X,Y,Z)]),
                 ((i-1, j, k), [(x,Y,z), (x,y,z), (x,y,Z), (x,Y,Z)])]
        for neighbor, points in faces:
            if neighbor not in cells:
                quad(points)
    edges = Counter((a,b) for t in triangles for a,b in zip(t,t[1:]+t[:1]))
    assert all(n == 1 and edges[b,a] == 1 for (a,b),n in edges.items())
    volume = sum(vertices[a][0]*(vertices[b][1]*vertices[c][2]-vertices[b][2]*vertices[c][1])
                 + vertices[a][1]*(vertices[b][2]*vertices[c][0]-vertices[b][0]*vertices[c][2])
                 + vertices[a][2]*(vertices[b][0]*vertices[c][1]-vertices[b][1]*vertices[c][0])
                 for a,b,c in triangles)/6
    assert volume > 0
    xml = '<mesh><vertices>' + ''.join(
        f'<vertex x="{x}" y="{y}" z="{z}"/>' for x,y,z in vertices)
    xml += '</vertices><triangles>' + ''.join(
        f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a,b,c in triangles)
    return xml + '</triangles></mesh>', len(vertices), len(triangles), volume


with zipfile.ZipFile(BASE) as source:
    original = {n:source.read(n) for n in source.namelist()}
manifest = {'base':str(BASE), 'base_sha256':sha256(BASE.read_bytes()).hexdigest(),
            'source':'PerimeterGenerator.cpp::process_arachne:2703-2744,2751-2877',
            'common_settings':COMMON, 'pairs':{}}
for label, overrides, shape in CASES:
    model, vertices, triangles, volume = mesh(shape)
    pair = {'shape':shape, 'overrides':overrides, 'cases':{}}
    for mode in ['off', 'on']:
        members = dict(original)
        name = '3D/Objects/cube10.stl_1.model'
        members[name] = re.sub(r'<mesh>.*?</mesh>', model, members[name].decode(),
                               flags=re.S).encode()
        members['3D/3dmodel.model'] = members['3D/3dmodel.model'].replace(
            b'1 110 110 5', b'1 110 110 0.6')
        settings = json.loads(members['Metadata/project_settings.config'])
        settings.update(COMMON | overrides | {'alternate_extra_wall':'1' if mode == 'on' else '0'})
        members['Metadata/project_settings.config'] = json.dumps(
            settings, sort_keys=True, indent=2).encode()
        path = ROOT / f'{label}-{mode}.3mf'
        with zipfile.ZipFile(path, 'w') as out:
            for n,data in members.items():
                info = zipfile.ZipInfo(n, (2026,1,1,0,0,0))
                info.compress_type = zipfile.ZIP_DEFLATED
                out.writestr(info,data)
        pair['cases'][mode] = {'input_sha256':sha256(path.read_bytes()).hexdigest(),
                              'settings':COMMON | overrides | {'alternate_extra_wall':settings['alternate_extra_wall']},
                              'vertices':vertices, 'triangles':triangles,
                              'signed_volume_mm3':volume,
                              'directed_edges_once_with_reverse':True}
    manifest['pairs'][label] = pair
(ROOT / 'provenance.json').write_text(json.dumps(manifest,sort_keys=True)+'\n')
