"""Authored membership-complete WASM input; no slicer or oracle outputs."""
from pathlib import Path
from hashlib import sha256
import json, zipfile

ROOT = Path(__file__).parent
SOURCE = Path('tests/parity/validation/kobra3-centered/0.8.3mf')
with zipfile.ZipFile(SOURCE) as z:
    files = {name: z.read(name) for name in z.namelist()}
member = '<model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="14"/></model_instance>'
meta = files['Metadata/model_settings.config'].decode()
files['Metadata/model_settings.config'] = meta.replace('</plate>', member + '</plate>').encode()
output = ROOT / 'wasm-plate1.3mf'
with zipfile.ZipFile(output, 'w') as z:
    for name, data in files.items():
        assert not name.endswith('.gcode')
        info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        z.writestr(info, data)
(ROOT / 'wasm-plate1-provenance.json').write_text(json.dumps({
    'source': str(SOURCE), 'source_sha256': sha256(SOURCE.read_bytes()).hexdigest(),
    'input': str(output), 'input_sha256': sha256(output.read_bytes()).hexdigest(),
    'changes': ['only add plate1 object2 instance0 identify14 membership'],
    'native_boundary': 'bbs_3mf.cpp:2058-2082/4652-4666/4698',
    'unchanged': ['model geometry', 'transforms', 'settings', 'original shared fixture'],
}, sort_keys=True) + '\n')
