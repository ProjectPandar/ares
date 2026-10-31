"""Build one input-only, four-island default-tour witness; no oracle outputs."""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).parent
BASE = ROOT.parent / 'clipper-wrapper-geometry/nested-hole-island.3mf'
CENTERS = [(-20, -20), (20, -17), (17, 20), (-17, -5)]
FACES = [(0, 2, 1), (0, 3, 2), (4, 5, 6), (4, 6, 7),
         (0, 1, 5), (0, 5, 4), (1, 2, 6), (1, 6, 5),
         (2, 3, 7), (2, 7, 6), (3, 0, 4), (3, 4, 7)]
vertices, triangles = [], []
for x, y in CENTERS:
    offset = len(vertices)
    vertices.extend([(x - 5, y - 5, -0.5), (x + 5, y - 5, -0.5),
                     (x + 5, y + 5, -0.5), (x - 5, y + 5, -0.5),
                     (x - 5, y - 5, 0.5), (x + 5, y - 5, 0.5),
                     (x + 5, y + 5, 0.5), (x - 5, y + 5, 0.5)])
    triangles.extend(tuple(offset + i for i in face) for face in FACES)
edges = Counter((a, b) for t in triangles for a, b in zip(t, t[1:] + t[:1]))
assert all(n == 1 and edges[b, a] == 1 for (a, b), n in edges.items())
assert all(abs(ax - bx) > 10 or abs(ay - by) > 10
           for i, (ax, ay) in enumerate(CENTERS) for bx, by in CENTERS[i + 1:])
mesh = '<mesh><vertices>' + ''.join(
    f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in vertices)
mesh += '</vertices><triangles>' + ''.join(
    f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in triangles)
mesh += '</triangles></mesh>'
with zipfile.ZipFile(BASE) as archive:
    members = {name: archive.read(name) for name in archive.namelist()}
name = '3D/Objects/cube10.stl_1.model'
members[name] = re.sub(r'<mesh>.*?</mesh>', mesh, members[name].decode(), flags=re.S).encode()
path = ROOT / 'four-disjoint-prisms.3mf'
with zipfile.ZipFile(path, 'w') as archive:
    for name, data in members.items():
        info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        archive.writestr(info, data)
manifest = {
    'upstream': '345efec64c7b95a05ce5cf96df0dde66a158835c',
    'scope': 'input-only; no Native output or expected output fragments',
    'base': 'tests/parity/clipper-wrapper-geometry/nested-hole-island.3mf',
    'base_sha256': sha256(BASE.read_bytes()).hexdigest(),
    'input_sha256': sha256(path.read_bytes()).hexdigest(),
    'instances': 1, 'model_parts': 1, 'disjoint_closed_components': 4,
    'local_centers_mm': CENTERS, 'component_dimensions_mm': [10, 10, 1],
    'vertices': len(vertices), 'triangles': len(triangles),
    'every_directed_edge_once_with_reverse': True,
    'packaging_config_and_transform_unchanged': True,
}
(ROOT / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(manifest['input_sha256'])
