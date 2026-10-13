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

# Import controls follow bbs_3mf.cpp:2824-2883; none encode Native outputs.
step = PROFILES['step']
boundaries = {
    'empty': '',
    'four-values': 'object_id=1|0;0.3;0.6;0.3\n',
    'odd-count': 'object_id=1|0;0.3;0.3;0.15;0.6\n',
    'duplicate': step + PROFILES['ramp'],
    'unknown-ordinal': step.replace('object_id=1', 'object_id=9'),
    'malformed-record': 'not-a-record\n',
    'malformed-number': step.replace('|0;', '|not-a-number;'),
    'numeric-prefix': 'object_id= +1tail| 0; +3e-1; .3suffix;1.5e-1tail;0.6 trailing;0.15\n',
    'hex-number': 'object_id=1|0x0;0x1.3333333333333p-2;0.3;0x1.3333333333333p-3;0.6;0.15\n',
    'wrong-top': step.replace('0.6;0.15', '0.7;0.15'),
    'wrong-first': step.replace('|0;0.3;', '|0;0.2;'),
    'repeat-z': 'object_id=1|0;0.3;0.3;0.15;0.45;0.15;0.45;0.1;0.6;0.1\n',
}

def write_variant(label, members, contract):
    dest = DEST / f'{label}.3mf'
    with zipfile.ZipFile(dest, 'w') as output:
        for name, payload in members.items():
            info = zipfile.ZipInfo(name, date_time=(2026, 10, 13, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            output.writestr(info, payload)
    manifest['variants'][label] = dict(contract, sha256=hashlib.sha256(dest.read_bytes()).hexdigest())

with zipfile.ZipFile(DEST / 'absent.3mf') as source:
    absent = {name: source.read(name) for name in source.namelist()}
for label, profile in boundaries.items():
    write_variant(label, dict(absent, **{'Metadata/layer_heights_profile.txt': profile.encode()}), {'profile': profile})
write_variant('member-order', dict(absent, **{
    'metadata/LAYER_HEIGHTS_PROFILE.TXT': PROFILES['ramp'].encode(),
    'Metadata/layer_heights_profile.txt': step.encode(),
}), {'profiles_in_archive_order': [PROFILES['ramp'], step]})

import copy
import uuid
import xml.etree.ElementTree as ET
CORE = 'http://schemas.microsoft.com/3dmanufacturing/core/2015/02'
PRODUCTION = 'http://schemas.microsoft.com/3dmanufacturing/production/2015/06'
ET.register_namespace('', CORE)
ET.register_namespace('p', PRODUCTION)
ET.register_namespace('BambuStudio', 'http://schemas.bambulab.com/package/2021')

def xml_bytes(root):
    return ET.tostring(root, encoding='utf-8', xml_declaration=True)

members = absent.copy()
root = ET.fromstring(members['3D/3dmodel.model'])
resources = root.find(f'{{{CORE}}}resources')
second = copy.deepcopy(resources.find(f'{{{CORE}}}object'))
second.set('id', '3')
for element in second.iter():
    if f'{{{PRODUCTION}}}UUID' in element.attrib:
        element.set(f'{{{PRODUCTION}}}UUID', str(uuid.uuid5(uuid.NAMESPACE_URL, ET.tostring(element).decode())))
second.find(f'{{{CORE}}}components/{{{CORE}}}component').set(f'{{{PRODUCTION}}}path', '/3D/Objects/boundary.stl_2.model')
resources.append(second)
build = root.find(f'{{{CORE}}}build')
first_item = build.find(f'{{{CORE}}}item')
first_item.set('transform', '1 0 0 0 1 0 0 0 1 90 110 0')
second_item = copy.deepcopy(first_item)
second_item.set('objectid', '3')
second_item.set('transform', '1 0 0 0 1 0 0 0 1 130 110 0')
second_item.set(f'{{{PRODUCTION}}}UUID', str(uuid.uuid5(uuid.NAMESPACE_URL, 'layer-profile-object-3')))
build.append(second_item)
members['3D/3dmodel.model'] = xml_bytes(root)
members['3D/Objects/boundary.stl_2.model'] = members['3D/Objects/boundary.stl_1.model']
relations = ET.fromstring(members['3D/_rels/3dmodel.model.rels'])
relation = copy.deepcopy(list(relations)[0]); relation.set('Id', 'rel-2'); relation.set('Target', '/3D/Objects/boundary.stl_2.model'); relations.append(relation)
members['3D/_rels/3dmodel.model.rels'] = xml_bytes(relations).replace(b'ns0:', b'').replace(b'xmlns:ns0=', b'xmlns=')
settings = ET.fromstring(members['Metadata/model_settings.config'])
second_meta = copy.deepcopy(settings.find('object')); second_meta.set('id', '3'); settings.insert(1, second_meta)
plate = settings.find('plate'); second_instance = copy.deepcopy(plate.find('model_instance'))
for element in second_instance:
    if element.get('key') == 'object_id': element.set('value', '3')
    if element.get('key') == 'identify_id': element.set('value', '15')
plate.append(second_instance)
members['Metadata/model_settings.config'] = xml_bytes(settings)
write_variant('two-objects-absent', members, {'transform': 'clone model part to unique source path/id3; placements90,130'})
two_profiles = PROFILES['ramp'] + step.replace('object_id=1', 'object_id=2')
write_variant('two-objects', dict(members, **{'Metadata/layer_heights_profile.txt': two_profiles.encode()}), {'profile': two_profiles, 'baseline': 'two-objects-absent'})

# First source object is on unselected plate2; source ordinal2 remains plate1.
settings = ET.fromstring(members['Metadata/model_settings.config'])
plate = settings.find('plate'); first_instance = plate.find('model_instance'); plate.remove(first_instance)
plate2 = copy.deepcopy(plate)
for element in list(plate2):
    if element.tag == 'model_instance': plate2.remove(element)
    if element.get('key') == 'plater_id': element.set('value', '2')
plate2.append(first_instance); settings.insert(list(settings).index(plate) + 1, plate2)
members['Metadata/model_settings.config'] = xml_bytes(settings)
sequences = json.loads(members['Metadata/filament_sequence.json']); sequences['plate_2'] = sequences['plate_1'].copy()
members['Metadata/filament_sequence.json'] = json.dumps(sequences).encode()
write_variant('selected-plate-absent', members, {'selected_plate': 1, 'source_ordinal': 2})
write_variant('selected-plate', dict(members, **{'Metadata/layer_heights_profile.txt': two_profiles.encode()}), {'profile': two_profiles, 'selected_plate': 1, 'baseline': 'selected-plate-absent'})

(DEST / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
