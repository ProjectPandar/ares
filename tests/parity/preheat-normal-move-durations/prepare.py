#!/usr/bin/env python3
"""Author geometry, then let pinned Orca load the unchanged stock preset inputs."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

REVISION = "345efec64c7b95a05ce5cf96df0dde66a158835c"
MACHINE = "Bambu Lab X2D 0.4 nozzle"
PROCESS = "0.20mm Standard @BBL X2D"
FILAMENT = "Bambu PLA Basic @BBL X2D 0.4 nozzle"
TRIANGLES = [(0, 2, 1), (0, 3, 2), (4, 5, 6), (4, 6, 7),
             (0, 1, 5), (0, 5, 4), (1, 2, 6), (1, 6, 5),
             (2, 3, 7), (2, 7, 6), (3, 0, 4), (3, 4, 7)]
PRISMS = [(12, 12, 0.6, 30, 40), (80, 40, 0.6, 80, 40)]


def mesh(index, width, depth, height):
    points = [(0, 0, 0), (width, 0, 0), (width, depth, 0), (0, depth, 0),
              (0, 0, height), (width, 0, height), (width, depth, height), (0, depth, height)]
    vertices = "".join(f'<vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in points)
    triangles = "".join(f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in TRIANGLES)
    return f'<object id="{index}" type="model"><mesh><vertices>{vertices}</vertices><triangles>{triangles}</triangles></mesh></object>'


def write_zip(path, entries):
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, payload in entries.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, payload)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source = args.source.resolve()
    assert subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip() == REVISION
    assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"])
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    profiles = {}
    source_paths = {}
    for vendor in ["BBL", "OrcaFilamentLibrary"]:
        for path in sorted((source / "resources/profiles" / vendor).rglob("*.json")):
            value = json.loads(path.read_bytes())
            if "name" in value and "type" in value:
                profiles[value["name"]] = value
                source_paths[value["name"]] = path
    consumed = {}

    def resolve(name):
        value = profiles[name]
        path = source_paths[name]
        consumed[str(path.relative_to(source))] = hashlib.sha256(path.read_bytes()).hexdigest()
        fields = resolve(value["inherits"]) if value.get("inherits") else {}
        for include in value.get("include", []):
            fields.update(resolve(include))
        fields.update({key: value for key, value in value.items() if key != "include"})
        return fields

    files = []
    for kind, name in [("machine", MACHINE), ("process", PROCESS), ("filament", FILAMENT)]:
        fields = resolve(name)
        fields["inherits"] = ""
        path = output / f"{kind}.json"
        path.write_text(json.dumps(fields, indent=2) + "\n")
        files.append(path)
    resources = "".join(mesh(i, *p[:3]) for i, p in enumerate(PRISMS, 1))
    build = "".join(f'<item objectid="{i}" transform="1 0 0 0 1 0 0 0 1 {p[3]} {p[4]} 0" printable="1"/>'
                    for i, p in enumerate(PRISMS, 1))
    objects = "".join(f'<object id="{i}"><metadata key="name" value="prism-{i}"/><metadata key="extruder" value="{i}"/>'
                      f'<part id="{i}" subtype="normal_part"><metadata key="name" value="prism-{i}"/></part></object>'
                      for i in [1, 2])
    instances = "".join(f'<model_instance><metadata key="object_id" value="{i}"/>'
                        f'<metadata key="instance_id" value="0"/><metadata key="identify_id" value="{i}"/></model_instance>'
                        for i in [1, 2])
    plate = f'<plate><metadata key="plater_id" value="1"/><metadata key="filament_map_mode" value="Manual"/>' \
            f'<metadata key="filament_maps" value="1 2"/>{instances}</plate>'
    geometry = output / "geometry.3mf"
    write_zip(geometry, {
        "[Content_Types].xml": '<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>',
        "_rels/.rels": '<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>',
        "3D/3dmodel.model": f'<?xml version="1.0"?><model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><metadata name="Application">OrcaSlicer-2.5.0-beta</metadata><resources>{resources}</resources><build>{build}</build></model>',
        "Metadata/model_settings.config": f'<?xml version="1.0"?><config>{objects}{plate}</config>',
    })
    exported = output / "exported.3mf"
    command = [str(args.binary), "--load-settings", f"{files[0]};{files[1]}", "--load-filaments",
               f"{files[2]};{files[2]}", "--arrange", "0", "--export-3mf", str(exported), str(geometry)]
    (output / "command.json").write_text(json.dumps(command, indent=2) + "\n")
    with (output / "native-export.log").open("wb") as log:
        subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True)
    with zipfile.ZipFile(exported) as archive:
        entries = {name: archive.read(name) for name in archive.namelist()}
    settings = json.loads(entries["Metadata/project_settings.config"])
    settings.update({"filament_map_mode": "Manual", "filament_map": ["1", "2"],
                     "layer_height": "0.2", "initial_layer_print_height": "0.2", "print_sequence": "by layer"})
    entries["Metadata/project_settings.config"] = json.dumps(settings, indent=2).encode()
    write_zip(output / "x2d-two-prisms.3mf", entries)
    manifest = {"upstream": REVISION, "tree": subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD^{tree}"], text=True).strip(),
                "geometry": PRISMS, "triangles_per_prism": TRIANGLES, "source_profiles": consumed,
                "selection": {"machine": MACHINE, "process": PROCESS, "filaments": [FILAMENT, FILAMENT]},
                "input_sha256": hashlib.sha256((output / "x2d-two-prisms.3mf").read_bytes()).hexdigest(),
                "entries_sha256": {name: hashlib.sha256(value).hexdigest() for name, value in entries.items()}}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(output / "x2d-two-prisms.3mf")


if __name__ == "__main__":
    main()
