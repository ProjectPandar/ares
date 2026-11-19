#!/usr/bin/env python3
"""Build seam paint volume fixtures from the synthetic aligned boxes."""

from copy import deepcopy
from hashlib import sha256
from pathlib import Path
import xml.etree.ElementTree as ET
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

CORE = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
PROD = "http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
PART = "3D/Objects/part-11.model"
ROOT = "3D/3dmodel.model"
SETTINGS = "Metadata/model_settings.config"
ET.register_namespace("", CORE)
ET.register_namespace("p", PROD)


def project(base, variant):
    root, parts, settings = (ET.fromstring(base.read(p)) for p in (ROOT, PART, SETTINGS))
    components = root.find(f".//{{{CORE}}}components")
    resources = parts.find(f"{{{CORE}}}resources")
    settings_object = settings.find("object")
    component_template = deepcopy(components[0])
    mesh_template = deepcopy(resources[0])
    part_template = deepcopy(settings_object.find("part"))
    resources.remove(resources[0])
    components.remove(components[0])
    settings_object.remove(settings_object.find("part"))

    def add(kind, transform, color=()):
        part_id = str(11 + len(resources))
        mesh = deepcopy(mesh_template)
        mesh.set("id", part_id)
        triangles = mesh.find(f".//{{{CORE}}}triangles")
        for index in color:
            triangles[index].set("paint_seam", "4")
        resources.append(mesh)
        component = deepcopy(component_template)
        component.set("objectid", part_id)
        component.set("transform", transform)
        components.append(component)
        metadata = deepcopy(part_template)
        metadata.attrib.update(id=part_id, subtype=kind)
        metadata.find("metadata").set("value", f"Part {part_id}")
        settings_object.append(metadata)

    if variant.startswith("stack"):
        # Match the source box outline, with a shared edge between the slabs.
        # The first slab is reflected through both X and Y: its local rear is front.
        painted = variant == "stack-painted"
        add("normal_part", "-1 0 0 0 -1 0 0 0 .5 0 0 0", (8, 9) if painted else ())
        add("normal_part", "1 0 0 0 1 0 0 0 .5 0 0 3", (4, 5) if painted else ())
    else:
        add("normal_part", "1 0 0 0 1 0 0 0 1 0 0 0")
        if variant.startswith("hole"):
            painted = variant == "hole-painted"
            add("negative_part", ".5 0 0 0 .5 0 0 0 1.5 1 1 -1", (4, 5) if painted else ())
        else:
            # Both non-printable helpers share the front plane with the model.
            transform = "1 0 0 0 .5 0 0 0 1 0 -4.5 0"
            add("modifier_part", transform, (4, 5) if variant == "residual-modifier" else ())
            add("support_blocker", transform, (4, 5) if variant == "residual-helper" else ())

    return {
        path: ET.tostring(xml, encoding="utf-8", xml_declaration=True)
        for path, xml in ((ROOT, root), (PART, parts), (SETTINGS, settings))
    }


def main():
    target = Path(__file__).resolve().parent
    sums = []
    for printer in ("k1", "mk4", "x1c"):
        source = target.with_name("seam-paint") / f"{printer}-plain.3mf"
        with ZipFile(source) as base:
            for variant in (
                "stack-plain", "stack-painted", "hole-plain", "hole-painted",
                "residual-plain", "residual-modifier", "residual-helper",
            ):
                dest = target / f"{printer}-{variant}.3mf"
                updated = project(base, variant)
                with ZipFile(dest, "w") as output:
                    for path in base.namelist():
                        payload = updated[path] if path in updated else base.read(path)
                        output.writestr(ZipInfo(path), payload, compress_type=ZIP_DEFLATED)
                sums.append(f"{sha256(dest.read_bytes()).hexdigest()}  {dest.name}\n")
    (target / "SHA256SUMS").write_text("".join(sums))


if __name__ == "__main__":
    main()
