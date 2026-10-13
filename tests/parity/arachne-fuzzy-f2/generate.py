"""Authored parameter-modifier inputs for FuzzySkin.cpp415-473/696-814.

No Native outputs are embedded. Run from the repository root.
"""
from hashlib import sha256
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).parent
PRISM = Path('tests/parity/arachne-fuzzy-f1/ripple-displacement.3mf')
OPEN = Path('tests/parity/arachne-fuzzy-f1/open-neck.3mf')
# Each tuple is label, base, global changes and ordered modifiers (scaleX, X, config).
LEFT = (0.5, -7.5)
CASES = [
    ('none-external', PRISM, {'fuzzy_skin':'none'},
     [(*LEFT, {'fuzzy_skin':'external'})]),
    ('external-none', PRISM, {}, [(*LEFT, {'fuzzy_skin':'none'})]),
    ('two-active', PRISM, {}, [(*LEFT, {'fuzzy_skin_thickness':'0.1',
                                     'fuzzy_skin_ripple_offset':'25%'})]),
    ('same-effect-types', PRISM, {}, [(*LEFT, {'fuzzy_skin':'all'})]),
    ('inner-eligibility', PRISM, {'fuzzy_skin':'allwalls'},
     [(*LEFT, {'fuzzy_skin':'external'})]),
    ('overlap-priority', PRISM, {},
     [(2/3, -5, {'fuzzy_skin_thickness':'0.1'}),
      (* (0.5, 7.5), {'fuzzy_skin_thickness':'0.3', 'fuzzy_skin_ripple_offset':'50%'})]),
    ('open-width-crossing', OPEN, {},
     [(0.5, -7.2, {'fuzzy_skin_thickness':'0.1', 'fuzzy_skin_ripple_offset':'25%'})]),
]

manifest = {'source':'FuzzySkin.cpp415-473/696-814; Algorithm/LineSplit.cpp64-314',
            'ruling':'791a9fbe9 accepted; F2 public RED before F1/F2 production',
            'cases':{}}
for label, base, changes, modifiers in CASES:
    with zipfile.ZipFile(base) as archive:
        members = {name:archive.read(name) for name in archive.namelist()}
    settings = json.loads(members['Metadata/project_settings.config'])
    settings.update(changes)
    members['Metadata/project_settings.config'] = json.dumps(settings,sort_keys=True,indent=2).encode()
    with zipfile.ZipFile(PRISM) as archive:
        template = archive.read('3D/Objects/cube10.stl_1.model').decode()
    components, relationships, parts, overrides = [], [], [], []
    for index, (scale_x, shift_x, config) in enumerate(modifiers):
        volume_id = index+3
        name = f'3D/Objects/fuzzy-mask-{volume_id}.model'
        members[name] = template.replace('id="1"',f'id="{volume_id}"').replace(
            '00010000-', f'000{volume_id}0000-').encode()
        z = 0.4 if base == OPEN else 0
        components.append(f'<component p:path="/{name}" objectid="{volume_id}" '
                          f'transform="{scale_x} 0 0 0 1 0 0 0 1 {shift_x} 0 {z}"/>')
        relationships.append(f'<Relationship Target="/{name}" Id="rel-fuzzy-{volume_id}" '
                             'Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>')
        # Equal wall settings are explicit even though fuzzy settings differ.
        config = {'wall_loops':settings['wall_loops']} | config
        part = f'<part id="{volume_id}" subtype="modifier_part"><metadata key="name" '
        part += f'value="fuzzy-mask-{volume_id}"/><metadata key="matrix" '
        part += f'value="{scale_x} 0 0 {shift_x} 0 1 0 0 0 0 1 {z} 0 0 0 1"/>'
        part += ''.join(f'<metadata key="{key}" value="{value}"/>' for key,value in config.items())
        parts.append(part+'</part>')
        overrides.append({'id':volume_id,'options':config,'scale_x':scale_x,'shift_x':shift_x})
    for name, close, insert in [
        ('3D/3dmodel.model','</components>',''.join(components)),
        ('3D/_rels/3dmodel.model.rels','</Relationships>',''.join(relationships)),
        ('Metadata/model_settings.config','</object>',''.join(parts)),
    ]:
        text = members[name].decode()
        assert text.count(close)==1
        members[name] = text.replace(close,insert+close).encode()
    path = ROOT/(label+'.3mf')
    with zipfile.ZipFile(path,'w') as archive:
        for name, data in members.items():
            assert not name.endswith('.gcode')
            info = zipfile.ZipInfo(name,(2026,1,1,0,0,0))
            info.compress_type=zipfile.ZIP_DEFLATED
            archive.writestr(info,data)
    manifest['cases'][label] = {
        'input_sha256':sha256(path.read_bytes()).hexdigest(),
        'base':str(base),'base_sha256':sha256(base.read_bytes()).hexdigest(),
        'settings':{key:settings[key] for key in settings if key.startswith('fuzzy_skin')
                    or key in ['wall_generator','wall_loops']},
        'modifiers':overrides,'volume_count':1+len(modifiers),
        'preflight_only':False,'official_contract':'strict','seeded_contract':'strict',
    }
(ROOT/'provenance.json').write_text(json.dumps(manifest,sort_keys=True)+'\n')
