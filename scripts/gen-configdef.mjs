#!/usr/bin/env node
// Generates the PrintConfigDef metadata table for ares-core from the upstream
// OrcaSlicer/src/libslic3r/PrintConfig.cpp definitions.
//
//   node scripts/gen-configdef.mjs            -> rewrite generated table files
//   node scripts/gen-configdef.mjs --check    -> exit 1 if files differ
//   node scripts/gen-configdef.mjs --dump     -> print the exported JSON to stdout
//
// Upstream boundary: PrintConfigDef::init_common_params, init_fff_params,
// init_extruder_option_keys and init_filament_option_keys in
// OrcaSlicer/src/libslic3r/PrintConfig.cpp, plus the material name table in
// OrcaSlicer/src/libslic3r/MaterialType.cpp (filament_type enum values) and
// enum class declarations in libslic3r headers (enum_keys_map int values).

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import { readUpstream } from './gen-configdef/upstream.mjs';
import { parseDefinitions } from './gen-configdef/parser.mjs';
import { projectDefinitions, renderMetadata } from './gen-configdef/output.mjs';

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

const outPath = join(OUT_DIR, 'metadata.rs');
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
    console.log('configdef metadata up to date');
    process.exit(0);
}

writeFileSync(outPath, formatted);
console.log(`wrote ${outPath} (${jsonDefs.length} options)`);
