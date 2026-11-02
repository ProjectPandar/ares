"""Input-only Classic unsupported-loop deferral witnesses; no oracle output."""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent / "arachne-fuzzy-f1/none.3mf"
COMMON = {
    "wall_generator": "classic", "wall_sequence": "outer wall/inner wall",
    "wall_direction": "ccw", "line_width": "0.45",
    "outer_wall_line_width": "0.45", "inner_wall_line_width": "0.45",
    "initial_layer_line_width": "0.45", "layer_height": "0.2",
    "initial_layer_print_height": "0.2", "precise_outer_wall": "0",
    "sparse_infill_density": "15%", "sparse_infill_pattern": "rectilinear",
    "internal_solid_infill_pattern": "monotonic", "is_infill_first": "0",
    "top_shell_layers": "2", "bottom_shell_layers": "3",
    "top_shell_thickness": "0", "bottom_shell_thickness": "0",
    "only_one_wall_top": "0", "only_one_wall_first_layer": "0",
    "detect_thin_wall": "0", "detect_overhang_wall": "1",
    "overhang_reverse": "0", "fuzzy_skin": "none",
    "gap_fill_target": "nowhere", "seam_position": "aligned", "seam_gap": "0",
    "seam_slope_type": "none", "spiral_mode": "0",
    "enable_arc_fitting": "disabled", "enable_support": "0",
    "enable_prime_tower": "0", "brim_type": "no_brim", "brim_width": "0",
    "skirt_loops": "0", "elefant_foot_compensation": "0", "wipe_inward": "0",
}
CASES = [
    ("unattached-ledge", 3.0, "3", "1"),
    ("disabled-control", 3.0, "3", "0"),
    ("attached-stack", 7.0, "8", "1"),
]


def mesh(half):
    xs = [-10.0, -half, half, 10.0]
    zs = [-1.0, 0.2, 1.0]
    cells = {(1, 1, 0)} | {(i, j, 1) for i in range(3) for j in range(3)}
    vertices, ids, triangles = [], {}, []

    def quad(points):
        indexes = []
        for point in points:
            if point not in ids:
                ids[point] = len(vertices)
                vertices.append(point)
            indexes.append(ids[point])
        a, b, c, d = indexes
        triangles.extend([(a, b, c), (a, c, d)])

    for i, j, k in sorted(cells):
        x0, x1, y0, y1, z0, z1 = xs[i], xs[i + 1], xs[j], xs[j + 1], zs[k], zs[k + 1]
        faces = [
            ((i, j, k - 1), [(x0, y1, z0), (x1, y1, z0), (x1, y0, z0), (x0, y0, z0)]),
            ((i, j, k + 1), [(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]),
            ((i, j - 1, k), [(x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)]),
            ((i + 1, j, k), [(x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)]),
            ((i, j + 1, k), [(x1, y1, z0), (x0, y1, z0), (x0, y1, z1), (x1, y1, z1)]),
            ((i - 1, j, k), [(x0, y1, z0), (x0, y0, z0), (x0, y0, z1), (x0, y1, z1)]),
        ]
        for neighbor, points in faces:
            if neighbor not in cells:
                quad(points)
    edges = Counter((a, b) for t in triangles for a, b in zip(t, t[1:] + t[:1]))
    assert all(n == 1 and edges[b, a] == 1 for (a, b), n in edges.items())
    volume = sum(
        vertices[a][0] * (vertices[b][1] * vertices[c][2] - vertices[b][2] * vertices[c][1])
        + vertices[a][1] * (vertices[b][2] * vertices[c][0] - vertices[b][0] * vertices[c][2])
        + vertices[a][2] * (vertices[b][0] * vertices[c][1] - vertices[b][1] * vertices[c][0])
        for a, b, c in triangles
    ) / 6
    assert abs(volume - ((2 * half) ** 2 * 1.2 + 400 * 0.8)) < 1e-8
    xml = "<mesh><vertices>" + "".join(
        f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in vertices
    ) + "</vertices><triangles>" + "".join(
        f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in triangles
    ) + "</triangles></mesh>"
    return xml, {"lower_half_width_mm": half, "upper_half_width_mm": 10.0,
                 "shoulder_height_mm": 1.2, "total_height_mm": 2.0,
                 "vertices": len(vertices), "triangles": len(triangles),
                 "closed_oriented_manifold": True, "signed_volume_mm3": volume}


with zipfile.ZipFile(BASE) as source:
    original = {name: source.read(name) for name in source.namelist()}
manifest = {"upstream": "345efec64c7b95a05ce5cf96df0dde66a158835c",
            "source": "PerimeterGenerator.cpp:1391-1455,1965;ExtrusionEntity.hpp:474;GCode.cpp:7742-7755,8889-8933",
            "base": "tests/parity/arachne-fuzzy-f1/none.3mf",
            "base_sha256": sha256(BASE.read_bytes()).hexdigest(), "cases": {}}
for name, half, walls, enabled in CASES:
    members = original.copy()
    xml, geometry = mesh(half)
    key = "3D/Objects/cube10.stl_1.model"
    members[key] = re.sub(r"<mesh>.*?</mesh>", xml, members[key].decode(), flags=re.S).encode()
    members["3D/3dmodel.model"] = members["3D/3dmodel.model"].replace(
        b"1 110 110 0.4", b"1 110 110 1.0"
    )
    members["Metadata/model_settings.config"] = members["Metadata/model_settings.config"].replace(
        b'value="cube10.stl"', f'value="{name}.stl"'.encode()
    )
    settings = json.loads(members["Metadata/project_settings.config"])
    changes = COMMON | {"wall_loops": walls, "unsupported_wall_last": enabled}
    settings.update(changes)
    members["Metadata/project_settings.config"] = json.dumps(settings, sort_keys=True, indent=2).encode()
    path = ROOT / f"{name}.3mf"
    with zipfile.ZipFile(path, "w") as out:
        for member, data in members.items():
            assert not member.endswith(".gcode")
            info = zipfile.ZipInfo(member, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            out.writestr(info, data)
    manifest["cases"][name] = {"input_sha256": sha256(path.read_bytes()).hexdigest(),
        "geometry": geometry, "settings_overrides": changes,
        "member_sha256": {member: sha256(data).hexdigest() for member, data in members.items()},
        "Native_output_embedded": False}
(ROOT / "provenance.json").write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
for name, case in manifest["cases"].items():
    print(name, case["input_sha256"])
