"""Input-only, closed orthogonal meshes; no Native output is read.

Run from repository root with Python 3. Reproducible ZIP metadata and mesh
edge checks make each 3MF a verifiable input, not a reference golden.
"""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).parent
BASE = Path('tests/parity/legacy-brim-ears/O0.3mf')
RECT = [[[-6, -6], [6, -6], [6, 6], [-6, 6]]]
CONCAVE = [[[-7, -7], [7, -7], [7, -1], [1, -1], [1, 7], [-7, 7]]]
TWO = [[[-11, -5], [-3, -5], [-3, 5], [-11, 5]],
       [[3, -5], [11, -5], [11, 5], [3, 5]]]
NECK = [[[-11, -6], [-3, -6], [-3, -1], [3, -1], [3, -6],
         [11, -6], [11, 6], [3, 6], [3, 1], [-3, 1], [-3, 6], [-11, 6]]]
CASES = [('rectangle', RECT, '4', '0'), ('concave', CONCAVE, '4', '0'),
         ('two-holes', TWO, '2', '0'), ('narrow-neck', NECK, '2', '0'),
         ('positive-gap', RECT, '4', '0.1'), ('collapsed-inset', RECT, '8', '0'),
         ('zero-width', RECT, '0', '0'), ('sub-pair-width', RECT, '0.1', '0'),
         ('no-hole', [], '4', '0'),
         ('two-islands', [[[-17,-4],[-7,-4],[-7,4],[-17,4]],
                          [[7,-4],[17,-4],[17,4],[7,4]]], '2', '0')]
COMMON = {'brim_type': 'outer_and_inner', 'brim_ears': '0',
          'brim_use_efc_outline': '0', 'elefant_foot_compensation': '0',
          'combine_brims': '0', 'skirt_loops': '0', 'skirt_height': '0',
          'sparse_infill_density': '0%', 'top_shell_layers': '0',
          'bottom_shell_layers': '0'}


def inside(x, y, polygon):
    hit = False
    for (ax, ay), (bx, by) in zip(polygon, polygon[1:] + polygon[:1]):
        if (ay > y) != (by > y) and x < ax + (y - ay) * (bx - ax) / (by - ay):
            hit = not hit
    return hit


def mesh(holes, contours):
    xs = sorted({p[0] for polygon in contours + holes for p in polygon})
    ys = sorted({p[1] for polygon in contours + holes for p in polygon})
    cells = {(i, j) for i in range(len(xs) - 1) for j in range(len(ys) - 1)
             if any(inside((xs[i] + xs[i+1])/2, (ys[j] + ys[j+1])/2, c) for c in contours)
             and not any(inside((xs[i] + xs[i+1])/2, (ys[j] + ys[j+1])/2, h)
                        for h in holes)}
    vertices, ids, triangles = [], {}, []

    def vertex(x, y, z):
        point = (x, y, z)
        if point not in ids:
            ids[point] = len(vertices)
            vertices.append(point)
        return ids[point]

    def quad(points):
        a, b, c, d = [vertex(*p) for p in points]
        triangles.extend([(a, b, c), (a, c, d)])

    for i, j in sorted(cells):
        a, b, c, d = [(xs[i], ys[j]), (xs[i+1], ys[j]),
                      (xs[i+1], ys[j+1]), (xs[i], ys[j+1])]
        quad([(*p, .5) for p in [a, b, c, d]])
        quad([(*p, -.5) for p in [d, c, b, a]])
        for p, q, neighbor in [(a, b, (i, j-1)), (b, c, (i+1, j)),
                               (c, d, (i, j+1)), (d, a, (i-1, j))]:
            if neighbor not in cells:
                quad([(*p, -.5), (*q, -.5), (*q, .5), (*p, .5)])
    edges = Counter((a, b) for t in triangles for a, b in zip(t, t[1:] + t[:1]))
    assert all(n == 1 and edges[b, a] == 1 for (a, b), n in edges.items())
    signed_volume = sum(
        vertices[a][0]*(vertices[b][1]*vertices[c][2]-vertices[b][2]*vertices[c][1])
        + vertices[a][1]*(vertices[b][2]*vertices[c][0]-vertices[b][0]*vertices[c][2])
        + vertices[a][2]*(vertices[b][0]*vertices[c][1]-vertices[b][1]*vertices[c][0])
        for a, b, c in triangles) / 6
    assert signed_volume > 0
    xml = '<mesh><vertices>' + ''.join(
        f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in vertices)
    xml += '</vertices><triangles>' + ''.join(
        f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in triangles)
    xml += '</triangles></mesh>'
    return xml, len(vertices), len(triangles), signed_volume


with zipfile.ZipFile(BASE) as archive:
    original = {n: archive.read(n) for n in archive.namelist()}
manifest = {'base': str(BASE), 'base_sha256': sha256(BASE.read_bytes()).hexdigest(),
            'source': 'libslic3r/Brim.cpp:448-458,512-570,661-711,813-875',
            'scope': 'input-only single watertight model part, z=0..1, center=110,110',
            'common_settings': COMMON, 'cases': {}}
for label, holes, width, gap in CASES:
    contours = [[[-15,-15],[15,-15],[15,15],[-15,15]]]
    if label == 'two-islands':
        contours = [[[-21,-10],[-3,-10],[-3,10],[-21,10]],
                    [[3,-10],[21,-10],[21,10],[3,10]]]
    model, vertex_count, triangle_count, volume = mesh(holes, contours)
    members = dict(original)
    name = '3D/Objects/cube10.stl_1.model'
    members[name] = re.sub(r'<mesh>.*?</mesh>', model, members[name].decode(),
                           flags=re.S).encode()
    members['3D/3dmodel.model'] = members['3D/3dmodel.model'].replace(
        b'1 110 110 5', b'1 110 110 0.5')
    settings = json.loads(members['Metadata/project_settings.config'])
    settings.update(COMMON | {'brim_width': width, 'brim_object_gap': gap})
    members['Metadata/project_settings.config'] = json.dumps(
        settings, indent=2, sort_keys=True).encode()
    path = ROOT / f'{label}.3mf'
    with zipfile.ZipFile(path, 'w') as out:
        for n, data in members.items():
            info = zipfile.ZipInfo(n, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            out.writestr(info, data)
    manifest['cases'][label] = {
        'input_sha256': sha256(path.read_bytes()).hexdigest(),
        'brim_width': width, 'brim_object_gap': gap,
        'holes_local_mm': holes, 'contours_local_mm': contours, 'vertices': vertex_count,
        'triangles': triangle_count, 'signed_volume_mm3': volume,
        'every_directed_edge_once_with_reverse': True,
        'expect_outer_brim': width not in ('0', '0.1'),
        'expect_inner_brim': bool(holes) and width not in ('0', '0.1')}
(ROOT / 'provenance.json').write_text(json.dumps(manifest, sort_keys=True) + '\n')
