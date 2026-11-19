"""Runtime Native-first rectangle qualification. Missing tools are failures, never skips."""
from pathlib import Path
import argparse
import collections
import datetime
import hashlib
import json
import os
import re
import subprocess
import sys
import traceback
import zipfile

from raw import verify

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
PARSER = argparse.ArgumentParser()
PARSER.add_argument('out', type=Path)
PARSER.add_argument('--case', choices=['painted-leaves', 'painted-split'], default='painted-leaves')
PARSER.add_argument('--medial-axis-input', type=Path)
PARSER.add_argument('--search', action='store_true')
ARGS = PARSER.parse_args()
OUT = ARGS.out.resolve()
ELF = Path(os.environ['ARES_ORCA_NATIVE_BIN']).resolve()
RESOURCES = Path(os.environ['ARES_ORCA_RESOURCES_DIR']).resolve()
DEBUG = Path(os.environ['ARES_ORCA_DEBUG_DIR']).resolve()
DEBUG_FILE = Path(os.environ['ARES_ORCA_DEBUG_FILE']).resolve()
BUILT_SOURCE = Path(os.environ['ARES_ORCA_SOURCE_DIR']).resolve()
UPSTREAM = Path(os.environ['ARES_ORCA_UPSTREAM_DIR']).resolve()
NATIVE_SHA = os.environ['ARES_ORCA_NATIVE_SHA256']
DEBUG_SHA = os.environ['ARES_ORCA_DEBUG_SHA256']
PIN = '40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19'
# These digests bind the fixed observer addresses to the release seed-replay recipe.
RECIPE_NATIVE_SHA = '84a7780653be3b46fe909e43ae98b3576907c561e8ff337c208ff97c2b23a84a'
RECIPE_DEBUG_SHA = '2f9478d7c10321cbf200eb254d1c94bace69118408b6fdf5e381c9d06981bf3c'
sha = lambda b: hashlib.sha256(b).hexdigest()
load = lambda p: json.loads(p.read_text())
canonical = lambda x: json.dumps(x, sort_keys=True, separators=(',', ':'))


def save(path, value):
    path.write_text(json.dumps(value, indent=2))


def git(*args):
    return subprocess.check_output(['git', '-C', str(REPO), *args], text=True).strip()


def activity(data, initial_tool=None):
    tool = initial_tool
    relative = True
    position = collections.defaultdict(float)
    counts = collections.Counter()
    layers = []
    feature = ''
    roles = {'Outer wall', 'Inner wall', 'Sparse infill', 'Internal solid infill',
             'Top surface', 'Bottom surface', 'Gap infill'}
    for line in data.decode().splitlines():
        if line == ';LAYER_CHANGE':
            layers.append(collections.Counter())
        if line.startswith(';TYPE:'):
            feature = line[6:]
        if re.fullmatch(r'T\d+', line):
            tool = int(line[1:])
        words = line.split(';')[0].split()
        if not words:
            continue
        if words[0] in ['M82', 'M83']:
            relative = words[0] == 'M83'
        if words[0] not in ['G0', 'G1', 'G92']:
            continue
        e = next((float(w[1:]) for w in words[1:] if w.startswith('E')), None)
        if e is None:
            continue
        delta = e if relative else e - position[tool]
        position[tool] = position[tool] + e if relative else e
        if words[0] == 'G92':
            position[tool] = e
        elif delta > 0 and layers and feature in roles and any(
                w.startswith(('X', 'Y')) for w in words[1:]):
            assert tool is not None
            counts[tool] += 1
            layers[-1][tool] += 1
    return {'positive_model_XYE_by_tool':dict(counts),
            'positive_model_XYE_by_layer':list(map(dict, layers)),
            'both_material_positive_layers':sum(0 in x and 1 in x for x in layers)}


def run():
    affinity = os.sched_getaffinity(0)
    assert len(affinity) == 1 and affinity == os.sched_getaffinity(os.getppid()), \
        'Native driver and Rust test must share one CPU'
    cpu = next(iter(affinity))
    assert os.environ['ORCA_REPLAY_CPU'] == str(cpu), 'inherit actual singleton replay CPU'
    assert not ARGS.search or ARGS.medial_axis_input is not None
    assert sha(ELF.read_bytes()) == NATIVE_SHA == RECIPE_NATIVE_SHA
    assert sha(DEBUG_FILE.read_bytes()) == DEBUG_SHA == RECIPE_DEBUG_SHA
    assert git('-C', str(UPSTREAM), 'rev-parse', 'HEAD') == PIN
    assert not git('-C', str(UPSTREAM), 'status', '--porcelain')
    sources = ['MultiMaterialSegmentation.cpp', 'TriangleSelector.cpp', 'Model.cpp',
               'PrintApply.cpp', 'PrintObjectSlice.cpp', 'Geometry/Voronoi.hpp',
               'Geometry/Voronoi.cpp', 'Geometry/VoronoiUtils.hpp',
               'Geometry/VoronoiUtils.cpp', 'Geometry/VoronoiUtilsCgal.hpp',
               'Geometry/VoronoiUtilsCgal.cpp', 'EdgeGrid.cpp', 'EdgeGrid.hpp']
    hashes = {}
    for name in sources:
        b = (BUILT_SOURCE / name).read_bytes()
        assert b == (UPSTREAM / 'src/libslic3r' / name).read_bytes()
        hashes[name] = sha(b)
    consumers = {}
    for name in ['Geometry/MedialAxis.cpp', 'Point.hpp', 'Arachne/SkeletalTrapezoidation.cpp']:
        data = (BUILT_SOURCE / name).read_bytes()
        assert data == (UPSTREAM / 'src/libslic3r' / name).read_bytes()
        consumers[name] = sha(data)
    head = git('rev-parse', 'HEAD')
    save(OUT / 'provenance.json', {
        'HEAD':head, 'UTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'Native_ELF':str(ELF), 'Native_SHA256':NATIVE_SHA,
        'matching_debug_file':str(DEBUG_FILE), 'debug_SHA256':DEBUG_SHA,
        'upstream':PIN, 'upstream_directory':str(UPSTREAM),
        'build_source_directory':str(BUILT_SOURCE), 'actual_build_source13_SHA256':hashes,
        'actual_consumer_source_SHA256':consumers,
        'case':ARGS.case, 'medial_axis_input':str(ARGS.medial_axis_input), 'bounded_search':ARGS.search,
        'Native_resources_directory':str(RESOURCES),
        'observer_SHA256':sha((HERE / 'observe.py').read_bytes()),
        'driver_SHA256':sha(Path(__file__).read_bytes()),
        'raw_decoder_SHA256':sha((HERE / 'raw.py').read_bytes()),
        'Rust_test_CPU_affinity':sorted(os.sched_getaffinity(os.getppid())),
        'working_tree_status':git('status', '--porcelain')})
    if ARGS.medial_axis_input is None:
        fixture = REPO / 'tests/parity/paint-color'
        case = load(fixture / 'manifest.json')['cases'][ARGS.case]
        prep = subprocess.run([sys.executable, str(fixture / 'prepare.py'), str(OUT / 'input')],
                              capture_output=True)
        (OUT / 'prepare.stdout').write_bytes(prep.stdout)
        (OUT / 'prepare.stderr').write_bytes(prep.stderr)
        save(OUT / 'prepare-exit.json', {'exit':prep.returncode})
        assert prep.returncode == 0
        project = OUT / 'input' / (ARGS.case + '.3mf')
    else:
        project = ARGS.medial_axis_input.resolve()
        assert project.parent == REPO / 'tests/parity/gated-role-flow'
        assert project.stem in ['gap-unit', 'box-unit', 'support-unit']
        case = load(project.parent / 'manifest.json')['cases'][project.stem]
    assert sha(project.read_bytes()) == case['archive_sha256']
    with zipfile.ZipFile(project) as archive:
        members = {n:sha(archive.read(n)) for n in archive.namelist()}
        if ARGS.medial_axis_input:
            assert len(json.loads(archive.read('Metadata/project_settings.config'))['filament_diameter']) == 1
    assert members == case['member_sha256']
    save(OUT / 'input-hashes.json', {'archive_SHA256':sha(project.read_bytes()),
                                   'member_SHA256':members})
    groups = []
    records = []
    pids = []
    streams = []
    samples = []
    header = re.compile(rb'(?m)^; generated by OrcaSlicer 2\.5\.0 on \d{4}-\d{2}-\d{2} at \d{2}:\d{2}:\d{2}$')
    gdb = os.environ['ARES_GDB_BIN']
    for number in ([1] if ARGS.search else [1, 2]):
        folder = OUT / f'Native-{number}'
        folder.mkdir()
        env = dict(os.environ, ORCA_REPLAY_SEED='0',
                   ARES_VD_DIAG_DIR=str(folder / 'capture'),
                   ARES_VD_NATIVE_ELF=str(ELF), ARES_VD_DEBUG_DIR=str(DEBUG),
                   ARES_VD_NATIVE_SHA=NATIVE_SHA,
                   ARES_VD_CONSTRUCTOR='lines' if ARGS.medial_axis_input else 'mmu',
                   ARES_VD_SEARCH='1' if ARGS.search else '0',
                   ARES_VD_CAPTURE_LIMIT='64' if ARGS.search else '100000')
        cmd = [gdb, '--batch', '-nx', '-x', str(HERE / 'observe.py'), '--args',
               str(ELF), '--slice', '0',
               '--outputdir', str(folder / 'native'), str(project)]
        save(folder / 'launch.json', {'command':cmd, 'cwd':str(folder),
             'UTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),
             'seed':0, 'CPU':[cpu], 'resources':str(RESOURCES), 'HEAD':head, 'gdb_SHA256':sha(Path(gdb).read_bytes())})
        with (folder / 'gdb.log').open('w') as log:
            proc = subprocess.run(cmd, cwd=folder, env=env, stdout=log,
                                  stderr=subprocess.STDOUT, timeout=240)
        save(folder / 'exit.json', {'exit':proc.returncode})
        assert proc.returncode == 0, 'Native observer failed; not backend RED'
        capture = folder / 'capture'
        outcome = load(capture / 'outcome.json')
        assert outcome['entries'] == outcome['finishes']
        if ARGS.medial_axis_input is None:
            assert outcome['entries'] == 50 and not outcome['limit_reached']
        assert outcome['exit_codes'] == [0] and not outcome['errors']
        observer = load(capture / 'observer.json')
        assert observer['ELF_SHA256'] == NATIVE_SHA
        pids.append(observer['pid'])
        samples += observer['initial_affinity']
        group = collections.defaultdict(list)
        files = sorted(capture.glob('call-*/record.json'))
        assert len(files) == outcome['entries']
        for path in files:
            row = load(path)
            assert row['pid'] == observer['pid'] and row['repair_enabled']
            vd = row['diagram']
            assert set(vd) == {'modified', 'state', 'issue', 'cells', 'vertices', 'edges'}
            if ARGS.medial_axis_input is None and ARGS.case == 'painted-leaves':
                assert not vd['modified'] and vd['state'] == vd['issue'] == 0
            assert row['events'][0]['kind'] == 'detect_original'
            assert all('returned_issue' in event for event in row['events'])
            if vd['state'] == 1:
                assert vd['modified'] and vd['issue'] == 0
                assert row['events'][0]['returned_issue'] != 0
                rotations = [e for e in row['events'] if e['kind'] == 'rotation']
                assert rotations and rotations[-1]['returned_issue'] == 0
            verify(path.parent, row, rectangle=(ARGS.medial_axis_input is None and ARGS.case == 'painted-leaves'))
            group[canonical(row['lines'])].append(canonical(vd))
            samples += row['entry_affinity'] + row['finish_affinity']
            records.append(str(path.relative_to(OUT)))
        if ARGS.medial_axis_input is None:
            assert len(group) == (1 if ARGS.case == 'painted-leaves' else 30)
        assert all(len(set(values)) == 1 for values in group.values())
        groups.append(dict(group))
        data = (folder / 'native/plate_1.gcode').read_bytes()
        assert len(header.findall(data)) == 1
        streams.append(header.sub(b'; generated by <SLICER> 2.5.0', data))
        active = activity(data, initial_tool=0 if ARGS.medial_axis_input else None)
        if ARGS.medial_axis_input is None:
            assert set(active['positive_model_XYE_by_tool']) == {0, 1}
            assert len(active['positive_model_XYE_by_layer']) == 50
            assert all(active['positive_model_XYE_by_layer'])
            assert active['both_material_positive_layers'] > 0
        else:
            assert active['positive_model_XYE_by_tool'] and active['positive_model_XYE_by_layer']
        save(folder / 'activity.json', active)
    assert len(set(pids)) == len(pids)
    if not ARGS.search:
        assert len(pids) == 2 and groups[0] == groups[1]
        assert streams[0] == streams[1], 'entire Native output differs beyond generator header'
    assert samples and all(row['mask'] == str(cpu) for row in samples)
    save(OUT / 'Native-qualified.json', {
        'status':'BOUNDED_SEARCH_ONLY' if ARGS.search else 'COMPLETE_NATIVE_FIRST', 'HEAD':head, 'completed_processes':len(pids),
        'Native_PIDs':pids, 'qualified_diagrams':len(records), 'records':records,
        'CPU':[cpu], 'thread_samples_single_CPU':len(samples), 'distinct_ordered_input_groups':len(groups[0]),
        'complete_effective_diagrams_repeat_exact':not ARGS.search,
        'complete_Gcode_repeat_generator_only_equal':not ARGS.search,
        'complete_Gcode_generator_only_SHA256':sha(streams[0]),
        'both_activity_checks_passed':not ARGS.search, 'independent_raw_decode_all_records':True,
        'repair_successful_records':sum(load(OUT / p)['diagram']['state'] == 1 for p in records),
        'scope':'bounded existing-input MedialAxis search' if ARGS.search else ARGS.case})


if __name__ == '__main__':
    try:
        run()
    except Exception:
        save(OUT / 'INFRASTRUCTURE-NON-GATE.json', {'status':'NOT_BACKEND_RED',
             'error':traceback.format_exc()})
        raise
