#!/usr/bin/env python3
"""Derive Aligned Back and spiral projects from the synthetic seam-paint boxes."""

import hashlib
import json
import re
from pathlib import Path
from zipfile import ZipFile

root = Path(__file__).resolve().parent
source = root.parent / 'seam-paint'
settings = 'Metadata/project_settings.config'


def derive(destination, fixtures, overrides):
    destination.mkdir(exist_ok=True)
    checksums = []
    for fixture in fixtures:
        output = destination / fixture.name
        with ZipFile(fixture) as old, ZipFile(output, 'w') as new:
            for member in old.infolist():
                data = old.read(member)
                if member.filename == settings:
                    expected = json.loads(data)
                    for key, (before, after) in overrides.items():
                        assert expected[key] == before
                        pattern = rb'("' + key.encode() + rb'"\s*:\s*)' + re.escape(json.dumps(before).encode())
                        data, count = re.subn(pattern, rb'\g<1>' + json.dumps(after).encode(), data)
                        assert count == 1
                        expected[key] = after
                    assert json.loads(data) == expected
                new.writestr(member, data)
        checksums.append(f'{hashlib.sha256(output.read_bytes()).hexdigest()}  {output.name}\n')
    (destination / 'SHA256SUMS').write_text(''.join(checksums))


aligned = sorted(source.glob('*.3mf'))
assert len(aligned) == 15
derive(root, aligned, {'seam_position': ('aligned', 'aligned_back')})
derive(root.with_name('seam-paint-spiral'), [source / f'k1-{paint}.3mf' for paint in ('front', 'plain')], {
    'spiral_mode': ('0', '1'),
    'top_shell_layers': ('5', '0'),
    'wall_loops': ('2', '1'),
    'sparse_infill_density': ('15%', '0%'),
    'brim_type': ('auto_brim', 'no_brim'),
})
