#!/usr/bin/env python3
"""Copy the seam-paint projects with only seam_position changed."""

import hashlib
import json
import re
from pathlib import Path
from zipfile import ZipFile

root = Path(__file__).resolve().parent
source = root.parent / 'seam-paint'
settings = 'Metadata/project_settings.config'
pattern = rb'("seam_position"\s*:\s*)"aligned"'
checksums = []
for fixture in sorted(source.glob('*.3mf')):
    output = root / fixture.name
    with ZipFile(fixture) as old, ZipFile(output, 'w') as new:
        for member in old.infolist():
            data = old.read(member)
            if member.filename == settings:
                original = json.loads(data)
                assert original['seam_position'] == 'aligned'
                data, count = re.subn(pattern, rb'\g<1>"random"', data)
                assert count == 1
                original['seam_position'] = 'random'
                assert json.loads(data) == original
            new.writestr(member, data)
    checksums.append(f'{hashlib.sha256(output.read_bytes()).hexdigest()}  {output.name}\n')
assert len(checksums) == 15
(root / 'SHA256SUMS').write_text(''.join(checksums))
