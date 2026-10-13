"""F1 input-only 3MFs; no Native output is read or embedded.

Run from repo root. Native admission/closure/min-width preflight decides which
authored inputs are valid parity cases before the RED corpus is committed.
"""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).parent
BASE = Path('tests/parity/legacy-brim-ears/O0.3mf')
OPEN = Path('tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf')
COMMON = {'wall_generator':'arachne', 'wall_loops':'2',
          'sparse_infill_density':'0%', 'top_shell_layers':'0',
          'bottom_shell_layers':'0', 'top_shell_thickness':'0',
          'bottom_shell_thickness':'0', 'only_one_wall_top':'0',
          'only_one_wall_first_layer':'0', 'detect_overhang_wall':'0',
          'overhang_reverse':'0', 'enable_support':'0', 'enable_prime_tower':'0',
          'single_extruder_multi_material':'0', 'spiral_mode':'0',
          'brim_type':'no_brim', 'brim_width':'0', 'brim_ears':'0',
          'brim_use_efc_outline':'0', 'elefant_foot_compensation':'0',
          'skirt_loops':'0', 'skirt_height':'0', 'alternate_extra_wall':'0',
          'layer_height':'0.2', 'initial_layer_print_height':'0.2',
          'seam_gap':'0', 'seam_slope_type':'none',
          'fuzzy_skin':'external', 'fuzzy_skin_first_layer':'1',
          'fuzzy_skin_thickness':'0.2', 'fuzzy_skin_point_distance':'0.3',
          'fuzzy_skin_noise_type':'ripple', 'fuzzy_skin_mode':'displacement',
          'fuzzy_skin_scale':'1', 'fuzzy_skin_octaves':'4',
          'fuzzy_skin_persistence':'0.5', 'fuzzy_skin_ripples_per_layer':'4',
          'fuzzy_skin_ripple_offset':'0%',
          'fuzzy_skin_layers_between_ripple_offset':'1'}
CASES = []
for noise in ['classic','perlin','billow','ridgedmulti','voronoi','ripple']:
    for mode in ['displacement','extrusion','combined']:
        overrides = {'fuzzy_skin_noise_type':noise,'fuzzy_skin_mode':mode}
        if noise == 'ripple':
            overrides['fuzzy_skin_point_distance'] = '1'
        CASES.append((noise+'-'+mode, 'prism', overrides, False))
CASES += [('hole','hollow',{'fuzzy_skin':'hole'},False),
          ('all','hollow',{'fuzzy_skin':'all'},False),
          ('allwalls','hollow',{'fuzzy_skin':'allwalls'},False),
          ('disabled','prism',{'fuzzy_skin':'disabled_fuzzy'},False),
          ('none','prism',{'fuzzy_skin':'none'},False),
          ('first-layer-off','prism',{'fuzzy_skin_first_layer':'0'},False),
          ('zero-thickness','prism',{'fuzzy_skin_thickness':'0'},False),
          ('sub-threshold','prism',{'fuzzy_skin_thickness':'0.0005'},False),
          ('ripple-phase','prism',{'fuzzy_skin_ripple_offset':'25%',
              'fuzzy_skin_layers_between_ripple_offset':'2'},False),
          ('open-neck','open',{'wall_loops':'1','fuzzy_skin':'allwalls'},False),
          ('small-padding','tiny',{'wall_loops':'1','fuzzy_skin_noise_type':'classic',
              'fuzzy_skin_point_distance':'2','fuzzy_skin_first_layer':'0'},False)]


def mesh(shape):
    xs = [-15,-6,6,15] if shape == 'hollow' else [-15,15]
    if shape == 'tiny':
        xs = [-0.4,0.4]
    ys = xs
    cells = {(i,j) for i in range(len(xs)-1) for j in range(len(ys)-1)
             if shape != 'hollow' or (i,j) != (1,1)}
    vertices, ids, triangles = [], {}, []

    def vertex(point):
        if point not in ids:
            ids[point] = len(vertices)
            vertices.append(point)
        return ids[point]

    def quad(points):
        a,b,c,d = [vertex(p) for p in points]
        triangles.extend([(a,b,c),(a,c,d)])

    for i,j in sorted(cells):
        a,b,c,d = [(xs[i],ys[j]),(xs[i+1],ys[j]),
                   (xs[i+1],ys[j+1]),(xs[i],ys[j+1])]
        quad([(*p,0.4) for p in [a,b,c,d]])
        quad([(*p,-0.4) for p in [d,c,b,a]])
        for p,q,neighbor in [(a,b,(i,j-1)),(b,c,(i+1,j)),
                             (c,d,(i,j+1)),(d,a,(i-1,j))]:
            if neighbor not in cells:
                quad([(*p,-0.4),(*q,-0.4),(*q,0.4),(*p,0.4)])
    edges = Counter((a,b) for t in triangles for a,b in zip(t,t[1:]+t[:1]))
    assert all(n==1 and edges[b,a]==1 for (a,b),n in edges.items())
    volume = sum(vertices[a][0]*(vertices[b][1]*vertices[c][2]-vertices[b][2]*vertices[c][1])
                 +vertices[a][1]*(vertices[b][2]*vertices[c][0]-vertices[b][0]*vertices[c][2])
                 +vertices[a][2]*(vertices[b][0]*vertices[c][1]-vertices[b][1]*vertices[c][0])
                 for a,b,c in triangles)/6
    assert volume>0
    xml = '<mesh><vertices>'+''.join(
        f'<vertex x="{x}" y="{y}" z="{z}"/>' for x,y,z in vertices)
    xml += '</vertices><triangles>'+''.join(
        f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a,b,c in triangles)
    return xml+'</triangles></mesh>',len(vertices),len(triangles),volume


manifest = {'source':'FuzzySkin.cpp:223-291,345-412,415-492,685-694',
            'spec':'308583e3a with approved F1/F2 split',
            'scope':'F1 single group whole coverage; no multi-group/LineSplit claim',
            'common_settings':COMMON, 'cases':{}}
for label,shape,overrides,preflight_only in CASES:
    base = OPEN if shape == 'open' else BASE
    with zipfile.ZipFile(base) as z:
        members = {n:z.read(n) for n in z.namelist()}
    geometry = {'reused_open_input':str(OPEN)}
    if shape != 'open':
        model,vertices,triangles,volume = mesh(shape)
        name = '3D/Objects/cube10.stl_1.model'
        members[name] = re.sub(r'<mesh>.*?</mesh>',model,members[name].decode(),flags=re.S).encode()
        members['3D/3dmodel.model'] = members['3D/3dmodel.model'].replace(
            b'1 110 110 5',b'1 110 110 0.4')
        geometry = {'vertices':vertices,'triangles':triangles,'signed_volume_mm3':volume,
                    'directed_edges_once_with_reverse':True,'height_mm':0.8}
    settings = json.loads(members['Metadata/project_settings.config'])
    settings.update(COMMON | overrides)
    members['Metadata/project_settings.config'] = json.dumps(settings,sort_keys=True,indent=2).encode()
    path = ROOT / (label+'.3mf')
    with zipfile.ZipFile(path,'w') as out:
        for n,data in members.items():
            info = zipfile.ZipInfo(n,(2026,1,1,0,0,0))
            info.compress_type = zipfile.ZIP_DEFLATED
            out.writestr(info,data)
    inactive = settings['fuzzy_skin'] in ['none','disabled_fuzzy']
    manifest['cases'][label] = {'input_sha256':sha256(path.read_bytes()).hexdigest(),
        'base':str(base),'base_sha256':sha256(base.read_bytes()).hexdigest(),
        'geometry':geometry,'settings':COMMON | overrides,'preflight_only':preflight_only,
        'official_contract':'strict' if inactive or settings['fuzzy_skin_noise_type']=='ripple'
                             else 'diagnostic only; never parity PASS',
        'seeded_contract':'strict' if not preflight_only else 'Native min-width diagnostic',
        'single_object_volume_group':True}
(ROOT/'provenance.json').write_text(json.dumps(manifest,sort_keys=True)+'\n')
