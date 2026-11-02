"""Reproduce the input-only Classic narrow-island guard witness."""
from collections import Counter
from hashlib import sha256
import json
from math import cos, pi, sin
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent / "arachne-fuzzy-f1/none.3mf"
OUTPUT = ROOT / "narrow-tube.3mf"
SIDES = 128
OUTER_RADIUS = 10.0
INNER_RADIUS = 8.55
HEIGHT = 0.8
SETTINGS = {
    "wall_generator": "classic",
    "wall_sequence": "inner-outer-inner wall",
    "wall_loops": "3",
    "wall_direction": "ccw",
    "line_width": "0.45",
    "outer_wall_line_width": "0.45",
    "inner_wall_line_width": "0.45",
    "initial_layer_line_width": "0.45",
    "layer_height": "0.2",
    "initial_layer_print_height": "0.2",
    "precise_outer_wall": "0",
    "sparse_infill_density": "15%",
    "sparse_infill_pattern": "rectilinear",
    "top_shell_layers": "0",
    "bottom_shell_layers": "0",
    "top_shell_thickness": "0",
    "bottom_shell_thickness": "0",
    "only_one_wall_top": "0",
    "only_one_wall_first_layer": "0",
    "detect_thin_wall": "0",
    "detect_overhang_wall": "0",
    "overhang_reverse": "0",
    "fuzzy_skin": "none",
    "gap_fill_target": "everywhere",
    "gap_infill_speed": "30",
    "filter_out_gap_fill": "0",
    "seam_position": "aligned",
    "seam_gap": "0",
    "seam_slope_type": "none",
    "spiral_mode": "0",
    "enable_arc_fitting": "disabled",
    "enable_support": "0",
    "enable_prime_tower": "0",
    "brim_type": "no_brim",
    "brim_width": "0",
    "skirt_loops": "0",
    "elefant_foot_compensation": "0",
}
vertices = [
    (round(radius * cos(2 * pi * i / SIDES), 12),
     round(radius * sin(2 * pi * i / SIDES), 12), z)
    for radius, z in [(OUTER_RADIUS, -HEIGHT / 2), (OUTER_RADIUS, HEIGHT / 2),
                      (INNER_RADIUS, -HEIGHT / 2), (INNER_RADIUS, HEIGHT / 2)]
    for i in range(SIDES)
]
triangles = []


def quad(a, b, c, d):
    triangles.extend([(a, b, c), (a, c, d)])


for i in range(SIDES):
    j = (i + 1) % SIDES
    quad(i, j, SIDES + j, SIDES + i)
    quad(2 * SIDES + j, 2 * SIDES + i, 3 * SIDES + i, 3 * SIDES + j)
    quad(SIDES + i, SIDES + j, 3 * SIDES + j, 3 * SIDES + i)
    quad(j, i, 2 * SIDES + i, 2 * SIDES + j)
edges = Counter((a, b) for t in triangles for a, b in zip(t, t[1:] + t[:1]))
assert all(n == 1 and edges[b, a] == 1 for (a, b), n in edges.items())
volume = sum(
    vertices[a][0] * (vertices[b][1] * vertices[c][2] - vertices[b][2] * vertices[c][1])
    + vertices[a][1] * (vertices[b][2] * vertices[c][0] - vertices[b][0] * vertices[c][2])
    + vertices[a][2] * (vertices[b][0] * vertices[c][1] - vertices[b][1] * vertices[c][0])
    for a, b, c in triangles
) / 6
assert volume > 0
mesh = "<mesh><vertices>" + "".join(
    f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in vertices
)
mesh += "</vertices><triangles>" + "".join(
    f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in triangles
) + "</triangles></mesh>"
with zipfile.ZipFile(BASE) as source:
    members = {name: source.read(name) for name in source.namelist()}
model = "3D/Objects/cube10.stl_1.model"
members[model] = re.sub(r"<mesh>.*?</mesh>", mesh, members[model].decode(), flags=re.S).encode()
members["Metadata/model_settings.config"] = members["Metadata/model_settings.config"].replace(
    b'value="cube10.stl"', b'value="narrow-tube.stl"'
)
settings = json.loads(members["Metadata/project_settings.config"])
settings.update(SETTINGS)
members["Metadata/project_settings.config"] = json.dumps(settings, sort_keys=True, indent=2).encode()
with zipfile.ZipFile(OUTPUT, "w") as output:
    for name, data in members.items():
        assert not name.endswith(".gcode")
        info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        output.writestr(info, data)
provenance = {
    "upstream": "345efec64c7b95a05ce5cf96df0dde66a158835c",
    "source": "PerimeterGenerator.cpp:1713,1715,1718,1861-1863,1867",
    "base": "tests/parity/arachne-fuzzy-f1/none.3mf",
    "base_sha256": sha256(BASE.read_bytes()).hexdigest(),
    "input_sha256": sha256(OUTPUT.read_bytes()).hexdigest(),
    "geometry": {"shape": "closed faceted circular tube", "sides": SIDES,
                 "outer_radius_mm": OUTER_RADIUS, "inner_radius_mm": INNER_RADIUS,
                 "wall_thickness_mm": OUTER_RADIUS - INNER_RADIUS, "height_mm": HEIGHT,
                 "signed_volume_mm3": volume, "closed_oriented_manifold": True},
    "settings_overrides": SETTINGS,
    "member_sha256": {name: sha256(data).hexdigest() for name, data in members.items()},
    "native_output_embedded": False,
}
(ROOT / "provenance.json").write_text(json.dumps(provenance, sort_keys=True, indent=2) + "\n")
print(provenance["input_sha256"])
