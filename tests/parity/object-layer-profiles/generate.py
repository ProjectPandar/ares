"""Authored input variants for the bbs_3mf stored layer profile boundary."""
import hashlib
import json
from pathlib import Path
import zipfile

BASE = Path('tests/parity/validation/object-range-layer-height.3mf')
DEST = Path(__file__).parent
PROFILES = {
    'absent': None,
    'ramp': 'object_id=1|0;0.3;0.3;0.2;0.6;0.1\n',
    'step': 'object_id=1|0;0.3;0.3;0.15;0.6;0.15\n',
}
manifest = {'base': str(BASE), 'base_sha256': hashlib.sha256(BASE.read_bytes()).hexdigest(),
            'removed_member': 'Metadata/layer_config_ranges.xml', 'variants': {}}
with zipfile.ZipFile(BASE) as source:
    for label, profile in PROFILES.items():
        dest = DEST / f'{label}.3mf'
        with zipfile.ZipFile(dest, 'w') as output:
            for name in source.namelist():
                if name != 'Metadata/layer_config_ranges.xml':
                    info = zipfile.ZipInfo(name, date_time=(2026, 10, 13, 0, 0, 0))
                    info.compress_type = zipfile.ZIP_DEFLATED
                    output.writestr(info, source.read(name))
            if profile is not None:
                info = zipfile.ZipInfo('Metadata/layer_heights_profile.txt', date_time=(2026, 10, 13, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                output.writestr(info, profile)
        manifest['variants'][label] = {'profile': profile, 'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()}
(DEST / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
