from pathlib import Path
import hashlib
import json
import zipfile

root = Path(__file__).parent
source = root.parent / 'm140-initial-bed/unused-hotter-filament.3mf'
with zipfile.ZipFile(source) as archive:
    entries = {name: archive.read(name) for name in archive.namelist()}
config = json.loads(entries['Metadata/project_settings.config'])
config['curr_bed_type'] = 'High Temp Plate'
for plate, initial, ordinary in [('cool', 35, 30), ('hot', 55, 50), ('textured', 61, 59)]:
    config[f'{plate}_plate_temp_initial_layer'] = [str(initial)] * 2
    config[f'{plate}_plate_temp'] = [str(ordinary)] * 2
entries['Metadata/project_settings.config'] = json.dumps(config, indent=2).encode()
manifest = {
    'upstream_commit': '40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19',
    'source': '../m140-initial-bed/unused-hotter-filament.3mf',
    'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
    'cases': [],
}
for name, bed, initial, ordinary in [
    ('omitted', None, 55, 50),
    ('textured-pei', 'Textured PEI Plate', 61, 59),
]:
    current = dict(entries)
    if bed is not None:
        current['Metadata/model_settings.config'] = current['Metadata/model_settings.config'].replace(
            b'<metadata key="plater_id" value="1"/>',
            f'<metadata key="plater_id" value="1"/>\n    <metadata key="bed_type" value="{bed}"/>'.encode(),
        )
    path = root / f'{name}.3mf'
    with zipfile.ZipFile(path, 'w') as archive:
        for entry, data in current.items():
            info = zipfile.ZipInfo(entry, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)
    manifest['cases'].append({
        'input': path.name,
        'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'project_bed_type': 'High Temp Plate',
        'plate_bed_type': bed,
        'initial_temperature': initial,
        'ordinary_temperature': ordinary,
    })
(root / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps(manifest, indent=2))
