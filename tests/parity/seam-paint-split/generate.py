#!/usr/bin/env python3
"""Derive the split-paint boxes from the synthetic aligned seam-paint inputs."""

import hashlib
import re
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo


def paint(payload, vertices, code):
    face = ('<triangle v1="%d" v2="%d" v3="%d"' % vertices).encode()
    payload, count = re.subn(face + rb'[^>]* />', face + b' paint_seam="' + code + b'" />', payload)
    assert count == 1, vertices
    return payload


def main():
    target = Path(__file__).resolve().parent
    original = target.with_name("seam-paint")
    sums = []
    for printer in ("x1c", "k1", "mk4"):
        for variant in ("plain", "front", "side"):
            name = f"{printer}-{variant}.3mf"
            source = original / name
            if variant == "plain":
                sums.append((name, hashlib.sha256(source.read_bytes()).hexdigest()))
                continue
            if variant == "front":
                # Split all edges of (0,5,4). Only child 0 is green. Its edge
                # along 0--5 meets the unsplit green (0,1,5) at a T-junction.
                vertices, code = (0, 5, 4), b"40003"
            else:
                # Bisect 1--6 without changing either colored side patch.
                vertices, code = (1, 2, 6), b"445"
            dest = target / name
            with ZipFile(source) as base, ZipFile(dest, "w") as project:
                for path in base.namelist():
                    payload = base.read(path)
                    if path == "3D/Objects/part-11.model":
                        payload = paint(payload, vertices, code)
                    project.writestr(ZipInfo(path), payload, compress_type=ZIP_DEFLATED)
            sums.append((name, hashlib.sha256(dest.read_bytes()).hexdigest()))
    (target / "SHA256SUMS").write_text("".join(f"{sha}  {name}\n" for name, sha in sorted(sums)))


if __name__ == "__main__":
    main()
