// Native processor dump and source-based GCodeViewer.cpp:4221,4230 index audit.
// Does not execute the disabled GUI table or manufacture missing volumes.
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, realpathSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const fixtures = join(root, 'tests/parity/color-change');
const { values } = parseArgs({ options: {
  oracle: { type: 'string' },
  output: { type: 'string', default: join(root, 'target/color-change-oracle') },
} });
if (!values.oracle) throw new Error('--oracle is required');
const output = resolve(values.output);
const oracle = realpathSync(values.oracle);
mkdirSync(output, { recursive: true });
const sha256 = (path) => createHash('sha256').update(readFileSync(path)).digest('hex');
const report = {
  source_revision: '345efec64c7b95a05ce5cf96df0dde66a158835c',
  patch_sha256: sha256(join(root, 'tools/chain-probe/procgcode.patch')),
  binaries: Object.fromEntries([oracle, join(dirname(oracle), '.orca-slicer-wrapped')]
    .filter(existsSync).map((path) => [path, sha256(path)])),
  config: { tools: 1, diameter_mm: 1.75, density_g_cm3: 1.24,
    color: '#F2754E', flavor: 'MarlinLegacy', semm: false, tag_family: 'BBL' },
  audit_source: 'GCodeViewer.cpp:4221,4230: one volume read per ColorChange time record',
  fixtures: [],
};
for (const name of readdirSync(fixtures).filter((name) => name.endsWith('.gcode')).sort()) {
  const fixture = join(fixtures, name);
  const stem = name.slice(0, -'.gcode'.length);
  const raw = join(output, `${stem}.json`);
  const moves = join(output, `${stem}.csv`);
  const args = ['--process-gcode', fixture, '--colorchange-tools', '1',
    '--dump-color-change', raw, '--dump-moves', moves];
  const run = spawnSync(oracle, args, { encoding: 'utf8' });
  writeFileSync(join(output, `${stem}.log`), (run.stdout ?? '') + (run.stderr ?? ''));
  if (run.error) throw run.error;
  if (run.status !== 0) throw new Error(`${name}: oracle exit=${run.status} signal=${run.signal}`);
  const data = JSON.parse(readFileSync(raw, 'utf8'));
  const volumes = data.volumes_per_color_change;
  // Both matching-item and trailing-Print branches advance the volume index.
  const reads = data.custom_gcode_times.filter((record) => record.type === 0).length;
  const entry = {
    fixture: relative(root, fixture), sha256: sha256(fixture),
    command: [oracle, ...args], raw_json_sha256: sha256(raw), moves_csv_sha256: sha256(moves),
    event_count: data.custom_gcode_per_print_z.length,
    volume_count: volumes.length, color_time_count: reads,
    out_of_bounds_volume_indices: Array.from({ length: Math.max(0, reads - volumes.length) },
      (_, index) => volumes.length + index),
  };
  report.fixtures.push(entry);
  console.log(`${stem}: events=${entry.event_count} volumes=${volumes.length} `
    + `color_times=${reads} OOB=${JSON.stringify(entry.out_of_bounds_volume_indices)}`);
}
writeFileSync(join(output, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(`Evidence: ${join(output, 'report.json')}`);
