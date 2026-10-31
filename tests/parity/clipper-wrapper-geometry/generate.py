"""Generate input-only projects for the Clipper2 wrapper rewrite.

Run from the repository root with Python 3. No oracle output is read or embedded.
The existing archive supplies packaging/configuration only; all meshes below are
explicit closed voxel-boundary surfaces with reproducible ZIP metadata.
"""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).parent
BASE = Path('tests/parity/legacy-brim-ears/O0.3mf')


def rectangle(left, bottom, right, top):
    return [[left, bottom], [right, bottom], [right, top], [left, top]]


def inside(x, y, polygon):
    result = False
    for (ax, ay), (bx, by) in zip(polygon, polygon[1:] + polygon[:1]):
        if (ay > y) != (by > y) and x < ax + (y - ay) * (bx - ax) / (by - ay):
            result = not result
    return result


def make_mesh(bands):
    polygons = [polygon for _, _, regions in bands
                for contour, holes in regions for polygon in [contour] + holes]
    xs = sorted({p[0] for polygon in polygons for p in polygon})
    ys = sorted({p[1] for polygon in polygons for p in polygon})
    zs = sorted({z for low, high, _ in bands for z in [low, high]})
    cells = set()
    for k in range(len(zs) - 1):
        z = (zs[k] + zs[k + 1]) / 2
        regions = [region for low, high, members in bands if low < z < high
                   for region in members]
        for i in range(len(xs) - 1):
            for j in range(len(ys) - 1):
                x, y = (xs[i] + xs[i + 1]) / 2, (ys[j] + ys[j + 1]) / 2
                if any(inside(x, y, contour) and not any(inside(x, y, h) for h in holes)
                       for contour, holes in regions):
                    cells.add((i, j, k))
    vertices, ids, triangles = [], {}, []

    def vertex(point):
        if point not in ids:
            ids[point] = len(vertices)
            vertices.append(point)
        return ids[point]

    def quad(points):
        a, b, c, d = [vertex(p) for p in points]
        triangles.extend([(a, b, c), (a, c, d)])

    for i, j, k in sorted(cells):
        x0, x1, y0, y1 = xs[i], xs[i + 1], ys[j], ys[j + 1]
        low, high = zs[k], zs[k + 1]
        a, b, c, d = (x0, y0), (x1, y0), (x1, y1), (x0, y1)
        if (i, j, k + 1) not in cells:
            quad([(*p, high) for p in [a, b, c, d]])
        if (i, j, k - 1) not in cells:
            quad([(*p, low) for p in [d, c, b, a]])
        for p, q, neighbor in [(a, b, (i, j - 1, k)), (b, c, (i + 1, j, k)),
                               (c, d, (i, j + 1, k)), (d, a, (i - 1, j, k))]:
            if neighbor not in cells:
                quad([(*p, low), (*q, low), (*q, high), (*p, high)])
    edges = Counter((a, b) for t in triangles for a, b in zip(t, t[1:] + t[:1]))
    assert all(n == 1 and edges[b, a] == 1 for (a, b), n in edges.items())
    volume = sum(
        vertices[a][0] * (vertices[b][1] * vertices[c][2] - vertices[b][2] * vertices[c][1])
        + vertices[a][1] * (vertices[b][2] * vertices[c][0] - vertices[b][0] * vertices[c][2])
        + vertices[a][2] * (vertices[b][0] * vertices[c][1] - vertices[b][1] * vertices[c][0])
        for a, b, c in triangles) / 6
    assert volume > 0
    xml = '<mesh><vertices>' + ''.join(
        f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in vertices)
    xml += '</vertices><triangles>' + ''.join(
        f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in triangles)
    return xml + '</triangles></mesh>', len(vertices), len(triangles), volume


def needle(width, tip=5.8, bottom=None):
    bottom = -width / 2 if bottom is None else bottom
    top = bottom + width
    return [[-5, -5], [5, -5], [5, bottom], [tip, bottom],
            [tip, top], [5, top], [5, 5], [-5, 5]]


def flat(regions):
    return [[-0.5, 0.5, regions]]


COMMON = {
    'brim_type': 'no_brim', 'brim_ears': '0', 'brim_use_efc_outline': '0',
    'combine_brims': '0', 'skirt_loops': '0', 'skirt_height': '0',
    'sparse_infill_density': '0%', 'top_shell_layers': '0', 'bottom_shell_layers': '0',
    'elefant_foot_compensation': '0', 'enable_support': '0',
    'enable_prime_tower': '0', 'wall_generator': 'classic', 'wall_loops': '2',
    'layer_height': '0.2', 'initial_layer_print_height': '0.2',
    'resolution': '0', 'slice_gap_closing_radius': '0', 'seam_position': 'aligned',
    'xy_hole_compensation': '0', 'xy_contour_compensation': '0',
    'enable_arc_fitting': '0', 'outer_wall_line_width': '0.4',
    'inner_wall_line_width': '0.4', 'line_width': '0.4',
    'initial_layer_line_width': '0.4',
}
NESTED = flat([[rectangle(-15, -15, 15, 15), [rectangle(-10, -10, 10, 10)]],
               [rectangle(-6, -6, 6, 6), [rectangle(-2, -2, 2, 2)]]])
TOUCHING = flat([[rectangle(-15, -15, 15, 15),
                 [rectangle(-10, -10, -0.4, -0.4), rectangle(0.4, 0.4, 10, 10)]]])
STEP = [[-0.5, 0.05, [[rectangle(-5, -5, 5, 5), []]]],
        [0.05, 0.5, [[[[-5, -5], [7, -5], [7, 0], [5.5, 0],
                       [5.5, 5.5], [-5, 5.5]], []]]]]
CASES = [
    ('nested-hole-island', NESTED, {}),
    ('holes-touch-after-compensation', TOUCHING, {'xy_hole_compensation': '0.4'}),
    ('collinear-boundary-with-thin-spike', flat([[needle(0.0001), []]]), {}),
    ('round-offset-short-edge-below-threshold', flat([[needle(0.0098, tip=9, bottom=0.015625), []]]),
     {'brim_type': 'brim_ears', 'brim_width': '8', 'brim_object_gap': '2',
      'brim_ears_max_angle': '180', 'brim_ears_detection_length': '0'}),
    ('round-offset-short-edge-at-threshold', flat([[needle(0.010000002, tip=9, bottom=0.015625), []]]),
     {'brim_type': 'brim_ears', 'brim_width': '8', 'brim_object_gap': '2',
      'brim_ears_max_angle': '180', 'brim_ears_detection_length': '0'}),
    ('round-offset-short-edge-above-threshold', flat([[needle(0.0102, tip=9, bottom=0.015625), []]]),
     {'brim_type': 'brim_ears', 'brim_width': '8', 'brim_object_gap': '2',
      'brim_ears_max_angle': '180', 'brim_ears_detection_length': '0'}),
    ('coincident-width-overhang-intersections', STEP,
     {'wall_generator': 'arachne', 'wall_loops': '1', 'nozzle_diameter': ['0.5'],
      'outer_wall_line_width': '0.5', 'inner_wall_line_width': '0.5',
      'initial_layer_line_width': '0.5', 'line_width': '0.5',
      'detect_overhang_wall': '1', 'extra_perimeters_on_overhangs': '0'}),
]

with zipfile.ZipFile(BASE) as archive:
    original = {name: archive.read(name) for name in archive.namelist()}
manifest = {'base': str(BASE), 'base_sha256': sha256(BASE.read_bytes()).hexdigest(),
            'upstream': '345efec64c7b95a05ce5cf96df0dde66a158835c',
            'scope': 'input-only; one model part; geometry boundary, not Native reference output',
            'common_settings': COMMON, 'cases': {}}
for label, bands, overrides in CASES:
    xml, vertices, triangles, volume = make_mesh(bands)
    members = dict(original)
    name = '3D/Objects/cube10.stl_1.model'
    members[name] = re.sub(r'<mesh>.*?</mesh>', xml, members[name].decode(), flags=re.S).encode()
    members['3D/3dmodel.model'] = members['3D/3dmodel.model'].replace(
        b'1 110 110 5', b'1 110 110 0.5')
    settings = json.loads(members['Metadata/project_settings.config'])
    settings.update(COMMON | overrides)
    members['Metadata/project_settings.config'] = json.dumps(
        settings, indent=2, sort_keys=True).encode()
    path = ROOT / f'{label}.3mf'
    with zipfile.ZipFile(path, 'w') as archive:
        for name, data in members.items():
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)
    manifest['cases'][label] = {'input_sha256': sha256(path.read_bytes()).hexdigest(),
                                'bands': bands, 'overrides': overrides,
                                'vertices': vertices, 'triangles': triangles,
                                'signed_volume_mm3': volume,
                                'every_directed_edge_once_with_reverse': True}
(ROOT / 'provenance.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
