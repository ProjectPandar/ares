"""Build input projects from the synthetic STL and resolved stock settings.

Run from the repository root with stock X1C and K1 project paths as arguments.
Only project settings are read from those archives; no mesh or results are read.
"""
from hashlib import sha256
import json
from pathlib import Path
import sys
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED

ROOT = Path(__file__).parent
CORE = 'http://schemas.microsoft.com/3dmanufacturing/core/2015/02'
PRODUCTION = 'http://schemas.microsoft.com/3dmanufacturing/production/2015/06'
BAMBU = 'http://schemas.bambulab.com/package/2021'
MODEL_TYPE = 'http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel'
SETTINGS = 'Metadata/project_settings.config'
PART = '/3D/Objects/half.model'
XML = '<?xml version="1.0" encoding="UTF-8"?>\n'


def mesh():
    points = [tuple(map(float, row.split()[1:]))
              for row in Path('tests/parity/overhang.stl').read_text().splitlines()
              if row.lstrip().startswith('vertex ')]
    assert len(points) == 72
    vertices, triangles = [], []
    # Preserve the STL's two closed shells and triangle order within one part.
    for shell in (points[:36], points[36:]):
        indices = {}
        for i in range(0, len(shell), 3):
            face = []
            for x, y, z in shell[i:i + 3]:
                point = (x / 2, y / 2, z / 2 - 3.75)
                if point not in indices:
                    indices[point] = len(vertices)
                    vertices.append(point)
                face.append(indices[point])
            triangles.append(tuple(face))
    v = ''.join(f'<vertex x="{x:g}" y="{y:g}" z="{z:g}"/>' for x, y, z in vertices)
    t = ''.join(f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in triangles)
    return f'<mesh><vertices>{v}</vertices><triangles>{t}</triangles></mesh>'


def rels(target):
    return f'''<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="{target}" Id="model" Type="{MODEL_TYPE}"/>
</Relationships>'''


def project(label, center, settings):
    model = f'<model unit="millimeter" xmlns="{CORE}" xmlns:p="{PRODUCTION}" xmlns:BambuStudio="{BAMBU}" requiredextensions="p">'
    version = '<metadata name="BambuStudio:3mfVersion">1</metadata>'
    members = {
        '[Content_Types].xml': '''<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
 <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
 <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
 <Default Extension="config" ContentType="application/xml"/>
 <Default Extension="png" ContentType="image/png"/>
</Types>''',
        '_rels/.rels': rels('/3D/3dmodel.model'),
        '3D/_rels/3dmodel.model.rels': rels(PART),
        '3D/3dmodel.model': f'''{model}
 <metadata name="Application">BambuStudio-02.08.01.55</metadata>
 <metadata name="OrcaSlicer">2.5.0</metadata>
 {version}
 <resources><object id="2" type="model"><components>
  <component p:path="{PART}" objectid="1" transform="1 0 0 0 1 0 0 0 1 0 0 3.75"/>
 </components></object></resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 {center} {center} 0" printable="1"/></build>
</model>''',
        PART[1:]: f'''{model}
 {version}
 <resources><object id="1" type="model">{mesh()}</object></resources><build/>
</model>''',
        'Metadata/model_settings.config': '''<config>
 <object id="2" auto_drop="1">
  <metadata key="name" value="half.stl"/>
  <metadata key="extruder" value="1"/>
  <part id="1" subtype="normal_part">
   <metadata key="name" value="half.stl"/>
   <metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 3.75 0 0 0 1"/>
  </part>
 </object>
 <plate>
  <metadata key="plater_id" value="1"/>
  <metadata key="filament_map_mode" value="Auto For Flush"/>
  <model_instance>
   <metadata key="object_id" value="2"/>
   <metadata key="instance_id" value="0"/>
   <metadata key="identify_id" value="1"/>
  </model_instance>
 </plate>
</config>''',
    }
    path = ROOT / f'{label}.3mf'
    with ZipFile(path, 'w') as out:
        for name, data in members.items():
            out.writestr(ZipInfo(name), XML + data + '\n', ZIP_DEFLATED)
        out.writestr(ZipInfo(SETTINGS), json.dumps(settings, sort_keys=True) + '\n', ZIP_DEFLATED)
    print(label, sha256(path.read_bytes()).hexdigest())


def stock(path, expected_sha):
    with ZipFile(path) as archive:
        settings = archive.read(SETTINGS)
    assert sha256(settings).hexdigest() == expected_sha
    return json.loads(settings)


if __name__ == '__main__':
    x1c = stock(sys.argv[1], '214f9ec02a67f05c99154ba989cc1c86dce17c14f56e113b5a17f16a959c8c4b')
    k1 = stock(sys.argv[2], 'e9927108a67dc1468109c7e9fe030e523b60e99a109f69f14051b24c5ac49491')
    common = {'enable_support': '1', 'support_type': 'normal(auto)', 'support_style': 'default'}
    x1c |= common | {'farthest_point_timelapse': '0'}
    k1 |= common
    project('x1c-grid-interface', 128, x1c | {'support_interface_pattern': 'grid'})
    project('x1c-two-layer-raft', 128, x1c | {'raft_layers': '2'})
    project('k1-rectilinear-grid-base', 110, k1 | {'support_base_pattern': 'rectilinear-grid'})
