#!/usr/bin/env python3
"""Copy the synthetic seam box with alternate selector leaves."""

import hashlib
import json
import re
from pathlib import Path
from zipfile import ZipFile

root = Path(__file__).resolve().parent
source = root.parent / 'seam-paint'
settings = 'Metadata/project_settings.config'
variants = {
    'front-state-3': ('front', b'4', b'0C'),
    'rear-state-4': ('rear', b'8', b'1C'),
    'split-front-state-15': ('front', b'4', b'CCCC1'),
    'front-state-17': ('front', b'4', b'EC'),
    'rear-state-18': ('rear', b'8', b'0FC'),
    'front-state-32': ('front', b'4', b'EFC'),
    'all-state-32': ('plain', b'', b'EFC'),
    'enforcer-with-rear-state-32': ('both', b'8', b'EFC'),
}

checksums = []
for mode in ('aligned_back', 'nearest', 'random'):
    for printer in ('k1', 'mk4', 'x1c'):
        for variant, (original, old_state, new_state) in sorted(variants.items()):
            output = root / f'{mode}-{printer}-{variant}.3mf'
            with ZipFile(source / f'{printer}-{original}.3mf') as old, ZipFile(output, 'w') as new:
                edited_model = False
                for member in old.infolist():
                    data = old.read(member)
                    if member.filename == settings:
                        expected = json.loads(data)
                        assert expected['seam_position'] == 'aligned'
                        expected['seam_position'] = mode
                        data, count = re.subn(
                            rb'("seam_position"\s*:\s*)"aligned"',
                            rb'\g<1>"' + mode.encode() + b'"', data,
                        )
                        assert count == 1 and json.loads(data) == expected
                    elif member.filename.startswith('3D/Objects/') and member.filename.endswith('.model'):
                        attr = b'paint_seam="' + new_state + b'"'
                        if old_state:
                            old_attr = b'paint_seam="' + old_state + b'"'
                            assert data.count(old_attr) == 2
                            data = data.replace(old_attr, attr)
                        else:
                            assert b'paint_seam=' not in data and data.count(b'<triangle ') == 12
                            data = data.replace(b'<triangle ', b'<triangle ' + attr + b' ')
                        assert not edited_model
                        edited_model = True
                    new.writestr(member, data)
                assert edited_model
            checksums.append(f'{hashlib.sha256(output.read_bytes()).hexdigest()}  {output.name}\n')
(root / 'SHA256SUMS').write_text(''.join(checksums))
