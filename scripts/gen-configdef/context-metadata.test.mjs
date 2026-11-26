import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { closeSync, mkdirSync, openSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { inflateRawSync } from 'node:zlib';

import { renderPrintConfigDefJson } from './export.mjs';

const repo = fileURLToPath(new URL('../../', import.meta.url));
const targets = [
    'fan_kickstart', 'fan_speedup_time', 'filament_toolchange_delay',
    'input_shaping_damp_x', 'input_shaping_damp_y',
    'input_shaping_freq_x', 'input_shaping_freq_y',
    'machine_load_filament_time', 'machine_tool_change_time',
    'machine_unload_filament_time', 'preheat_time', 'slow_down_layer_time',
];
const hash = (value) => createHash('sha256').update(value).digest('hex');
const byKey = (jsonBuffer) => new Map(
    JSON.parse(jsonBuffer.toString('utf8')).options.map((entry) => [entry.key, JSON.stringify(entry)]),
);

test('pinned source generation reproduces all twelve committed contextual records', () => {
    assert(process.env.ARES_CONFIGDEF_TEST_ARTIFACTS, 'set ARES_CONFIGDEF_TEST_ARTIFACTS to a fresh directory');
    const artifacts = resolve(process.env.ARES_CONFIGDEF_TEST_ARTIFACTS);
    mkdirSync(artifacts);
    const source = spawnSync('git', ['-C', join(repo, 'OrcaSlicer'), 'rev-parse', 'HEAD']);
    assert.equal(source.status, 0, source.stderr.toString());
    assert.equal(source.stdout.toString().trim(), '40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19');
    const stdoutPath = join(artifacts, 'stdout.json');
    const stdout = openSync(stdoutPath, 'w');
    const result = spawnSync(process.execPath, [join(repo, 'scripts/gen-configdef.mjs'), '--dump'], {
        stdio: ['ignore', stdout, 'pipe'],
    });
    closeSync(stdout);
    writeFileSync(join(artifacts, 'stderr.log'), result.stderr);
    assert.equal(result.status, 0, result.stderr.toString());
    const options = JSON.parse(readFileSync(stdoutPath, 'utf8')).options;
    const generated = renderPrintConfigDefJson(options, repo);
    writeFileSync(join(artifacts, 'generated-print-config-def.json'), generated);
    const committedDeflate = readFileSync(
        join(repo, 'crates/ares-core/src/options/registry/definitions/print_config_def.json.deflate'),
    );
    const committed = inflateRawSync(committedDeflate);
    const committedEntries = byKey(committed);
    const generatedEntries = byKey(generated);
    const selected = Object.fromEntries(targets.map((key) => [key, {
        committed: hash(committedEntries.get(key)), generated: hash(generatedEntries.get(key)),
    }]));
    writeFileSync(join(artifacts, 'target-blocks.json'), JSON.stringify(selected, null, 2) + '\n');
    for (const key of targets) {
        assert.equal(generatedEntries.get(key), committedEntries.get(key), key);
    }
    const mixed = options.find((entry) => entry.key === 'fan_cooling_layer_time');
    assert.equal(mixed.sidetext, 's');
    writeFileSync(join(artifacts, 'verification.json'), JSON.stringify({
        source: source.stdout.toString().trim(), verifiedBlocks: targets.length,
        generatedSHA256: hash(generated), committedSHA256: hash(committed), mixedUnit: mixed.sidetext,
    }, null, 2) + '\n');
});
