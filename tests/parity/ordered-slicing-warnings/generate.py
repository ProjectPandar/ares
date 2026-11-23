"""Generate one object with a grounded cantilever and an internal Z gap."""

from pathlib import Path
import json
import re
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

root = Path(__file__).resolve().parent
vertices = []
triangles = []


def prism(outline, end_faces, half_depth):
    offset = len(vertices)
    n = len(outline)
    vertices.extend((x, y, z) for y in (-half_depth, half_depth) for x, z in outline)
    triangles.extend((offset + a, offset + b, offset + c) for a, b, c in end_faces)
    triangles.extend((offset + n + c, offset + n + b, offset + n + a) for a, b, c in end_faces)
    for a in range(n):
        b = (a + 1) % n
        triangles.extend(
            ((offset + a, offset + n + a, offset + b),
             (offset + b, offset + n + a, offset + n + b))
        )


prism(
    [(-2, 0), (2, 0), (2, 3), (10, 3), (10, 5), (-10, 5), (-10, 3), (-2, 3)],
    [(7, 0, 1), (7, 1, 2), (4, 2, 3), (4, 7, 2), (4, 6, 7), (4, 5, 6)],
    5,
)
prism([(-4, 9), (4, 9), (4, 11), (-4, 11)], [(0, 1, 2), (0, 2, 3)], 4)

mesh = (
    "<mesh><vertices>"
    + "".join(f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in vertices)
    + "</vertices><triangles>"
    + "".join(f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in triangles)
    + "</triangles></mesh>"
)
with (
    ZipFile(root.parent / "mesh-edge-pairing/cube-control.3mf") as base,
    ZipFile(root / "cantilever-with-gap.3mf", "w") as output,
):
    for name in base.namelist():
        data = base.read(name)
        if name == "3D/Objects/cube10.stl_1.model":
            data = re.sub(r"<mesh>[\s\S]*?</mesh>", mesh, data.decode()).encode()
        if name == "Metadata/model_settings.config":
            data = data.replace(b'value="cube10.stl"', b'value="cantilever-with-gap"')
        if name == "Metadata/project_settings.config":
            settings = json.loads(data)
            settings.update(
                layer_height="0.2", initial_layer_print_height="0.2",
                support_top_z_distance="0.2", enforce_support_layers="0", raft_layers="0",
            )
            data = json.dumps(settings, sort_keys=True).encode()
        output.writestr(ZipInfo(name, (2026, 1, 1, 0, 0, 0)), data, ZIP_DEFLATED)
