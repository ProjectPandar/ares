"""Reproduce frozen archives using only the standard library; output to a new directory.

TriangleSelector.cpp:1692-1768 and Model.cpp:3545-3599 at the manifest pin.
No Native output is generated here. Existing archives/manifest are never rewritten.
"""
from pathlib import Path
import hashlib
import json
import re
import sys
import zipfile

fixture = Path(__file__).resolve().parent
repo = fixture.parents[2]
manifest = json.loads((fixture / "manifest.json").read_text())
template = repo / "tests/parity/interface-shells/two-region-prism.3mf"
sha = lambda data: hashlib.sha256(data).hexdigest()
assert sha(template.read_bytes()) == manifest["template_SHA"]
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=False)
with zipfile.ZipFile(template) as archive:
    infos = archive.infolist()
    base = {info.filename: archive.read(info.filename) for info in infos}
config = json.loads(base["Metadata/project_settings.config"])
config.update(manifest["cases"]["unpainted"]["settings"])
base["Metadata/project_settings.config"] = json.dumps(config, indent=4).encode()
base["Metadata/model_settings.config"] = base["Metadata/model_settings.config"].replace(
    b'<metadata key="extruder" value="2"/>',
    b'<metadata key="extruder" value="1"/>',
)
# Unsplit: low two split bits 00, then two state bits. Model hex prepends.
extruder1 = 1 << 2
extruder2 = 2 << 2
# One split side and special_side 0 -> nibble 1. Serialize child1 before child0.
split_stream = [1, extruder2, extruder1]
split_code = "".join(format(nibble, "X") for nibble in reversed(split_stream))
annotations = {}
split_face = None
for name, data in base.items():
    if not name.startswith("3D/Objects/") or not name.endswith(".model"):
        continue
    xml = data.decode()
    vertices = [tuple(map(float, v)) for v in re.findall(
        r'<vertex x="([^"]+)" y="([^"]+)" z="([^"]+)"', xml)]
    triangles = [tuple(map(int, t)) for t in re.findall(
        r'<triangle v1="(\d+)" v2="(\d+)" v3="(\d+)"', xml)]
    lo = min(v[0] for v in vertices)
    hi = max(v[0] for v in vertices)
    annotations[name] = {}
    for index, triangle in enumerate(triangles):
        xs = [vertices[v][0] for v in triangle]
        if all(x == lo for x in xs):
            annotations[name][index] = format(extruder1, "X")
        if all(x == hi for x in xs):
            annotations[name][index] = format(extruder2, "X")
            split_face = split_face or (name, index)
for case, expected in manifest["cases"].items():
    members = dict(base)
    if case != "unpainted":
        for name, codes in annotations.items():
            xml = base[name].decode()
            triangles = re.findall(r'<triangle [^>]+/>', xml)
            for index, code in codes.items():
                if case == "painted-split" and split_face == (name, index):
                    code = split_code
                xml = xml.replace(triangles[index], triangles[index][:-2] +
                                  f' paint_color="{code}"/>')
            members[name] = xml.encode()
    assert {k: sha(v) for k, v in members.items()} == expected["member_sha256"]
    path = out / (case + ".3mf")
    with zipfile.ZipFile(path, "w") as archive:
        for info in infos:
            archive.writestr(info, members[info.filename])
    assert sha(path.read_bytes()) == expected["archive_sha256"]
    print(case, expected["archive_sha256"])
