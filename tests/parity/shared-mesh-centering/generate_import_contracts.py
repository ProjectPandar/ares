"""Authored inputs for approved SPEC415d796bd; never invokes a slicer."""
from pathlib import Path
from hashlib import sha256
import json, re, zipfile

ROOT = Path(__file__).parent
CASES = {
    'unassigned-owner': ('implicit-shared', [2, 3], [1, 1]),
    'default-scope': ('plate-local', [2, 3], [1, 1]),
    'local-id-bfs': ('nested-bfs', [2], [2]),
}
manifest = {'spec': '415d796bd', 'source': 'bbs_3mf.cpp:2058-2082/3851-3872/4867-4891', 'cases': {}}
for label, (source, ids, counts) in CASES.items():
    base = ROOT / (source + '.3mf')
    with zipfile.ZipFile(base) as z:
        files = {name: z.read(name) for name in z.namelist()}
    model = files['3D/3dmodel.model'].decode()
    meta = files['Metadata/model_settings.config'].decode()
    if label == 'unassigned-owner':
        model = re.sub(r'(<item objectid="2"[^>]*printable=")1', r'\g<1>0', model)
        meta = re.sub(r'<model_instance><metadata key="object_id" value="2"/>[\s\S]*?</model_instance>', '', meta)
        changes = ['owner object2 printable1->0', 'remove only owner plate1 membership']
    elif label == 'default-scope':
        def move(match):
            x = '374' if match[2] == '2' else '155'
            return match[1] + match[2] + match[3] + x + match[4]
        model = re.sub(r'(<item objectid=")(2|3)(" transform="1 0 0 0 1 0 0 0 1 )[0-9]+( 110 0.5")', move, model)
        def swap(match):
            value = match[2]
            return match[1] + {'2': '3', '3': '2', '14': '15', '15': '14'}[value] + match[3]
        meta = re.sub(r'(<metadata key="(?:object_id|identify_id)" value=")(2|3|14|15)("/>)', swap, meta)
        changes = ['swap plate object/label memberships', 'owner object2 globalX374, current object3 globalX155']
    else:
        model = model.replace(' p:path="/3D/3dmodel.model"', '')
        changes = ['ONLY omit root-to-object5 p:path; original literal-ID fixture retained']
    files['3D/3dmodel.model'] = model.encode()
    files['Metadata/model_settings.config'] = meta.encode()
    path = ROOT / (label + '.3mf')
    with zipfile.ZipFile(path, 'w') as z:
        for name, data in files.items():
            assert not name.endswith('.gcode')
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, data)
    settings = json.loads(files['Metadata/project_settings.config'])
    manifest['cases'][label] = {
        'base': source, 'base_sha256': sha256(base.read_bytes()).hexdigest(),
        'input_sha256': sha256(path.read_bytes()).hexdigest(), 'changes': changes,
        'object_ids': ids, 'volume_counts': counts,
        'settings': {k: settings[k] for k in ['wall_generator', 'seam_position', 'enable_support', 'skirt_loops', 'brim_type', 'brim_width']},
        'native_contract': 'strict ordered output in official and seeded',
    }
(ROOT / 'import-contract-provenance.json').write_text(json.dumps(manifest, sort_keys=True) + '\n')
