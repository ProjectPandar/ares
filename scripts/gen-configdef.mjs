#!/usr/bin/env node
// Generates the PrintConfigDef runtime validation table and pre-compressed
// JSON export for ares-core from upstream OrcaSlicer/src/libslic3r/PrintConfig.cpp.
//
//   node scripts/gen-configdef.mjs            -> rewrite generated files
//   node scripts/gen-configdef.mjs --check    -> exit 1 if files differ
//   node scripts/gen-configdef.mjs --dump     -> print the exported JSON to stdout
//
// Upstream boundary: PrintConfigDef::init_common_params, init_fff_params,
// init_extruder_option_keys and init_filament_option_keys in
// OrcaSlicer/src/libslic3r/PrintConfig.cpp, plus the material name table in
// OrcaSlicer/src/libslic3r/MaterialType.cpp (filament_type enum values) and
// enum class declarations in libslic3r headers (enum_keys_map int values).

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deflateRawSync, inflateRawSync } from 'node:zlib';

import { renderPrintConfigDefJson } from './gen-configdef/export.mjs';
import { projectDefinitions, renderMetadata } from './gen-configdef/output.mjs';
import { parseDefinitions } from './gen-configdef/parser.mjs';
import { readUpstream } from './gen-configdef/upstream.mjs';

const REPO = join(dirname(fileURLToPath(import.meta.url)), '..');
const OUT_DIR = join(REPO, 'crates/ares-core/src/options/registry/definitions');

const args = process.argv.slice(2);
const CHECK = args.includes('--check');
const DUMP = args.includes('--dump');

const upstream = readUpstream(REPO);
const parsed = parseDefinitions(upstream);
const jsonDefs = projectDefinitions(parsed, upstream);
const { groups } = parsed;

if (DUMP) {
    console.log(JSON.stringify({ options: jsonDefs, groups }, null, 2));
    process.exit(0);
}

const formatted = renderMetadata(jsonDefs);
const exportJson = renderPrintConfigDefJson(jsonDefs, REPO);

const outPath = join(OUT_DIR, 'metadata.rs');
const deflatePath = join(OUT_DIR, 'print_config_def.json.deflate');

function existingDeflateMatches(path, expectedJson) {
    try {
        const compressed = readFileSync(path);
        if (compressed.length === 0 || compressed.length >= expectedJson.length) {
            return false;
        }
        return inflateRawSync(compressed).equals(expectedJson);
    } catch {
        return false;
    }
}

if (CHECK) {
    let existing = '';
    try {
        existing = readFileSync(outPath, 'utf8');
    } catch {
        existing = '';
    }
    if (existing !== formatted) {
        console.error(`configdef metadata out of date: ${outPath}`);
        console.error('run `node scripts/gen-configdef.mjs` to regenerate');
        process.exit(1);
    }
    if (!existingDeflateMatches(deflatePath, exportJson)) {
        console.error(`configdef compressed JSON out of date: ${deflatePath}`);
        console.error('run `node scripts/gen-configdef.mjs` to regenerate');
        process.exit(1);
    }
    console.log('configdef metadata up to date');
    process.exit(0);
}

writeFileSync(outPath, formatted);
let compressed;
if (existingDeflateMatches(deflatePath, exportJson)) {
    compressed = readFileSync(deflatePath);
} else {
    compressed = deflateRawSync(exportJson, { level: 9 });
    writeFileSync(deflatePath, compressed);
}
console.log(
    `wrote ${outPath} (${jsonDefs.length} options) and ${deflatePath} (${compressed.length} bytes deflated from ${exportJson.length} bytes)`,
);
