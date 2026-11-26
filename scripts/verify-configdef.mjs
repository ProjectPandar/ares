#!/usr/bin/env node
// Independent per-key audit of the PrintConfigDef export.
//
//   cargo run -p ares-core --release --example print_config_def > /tmp/pcd.json
//   node scripts/verify-configdef.mjs /tmp/pcd.json [--report artifacts/configdef-verification.json]
//
// Extracts every `ConfigOptionDef` field independently from the upstream
// PrintConfig.cpp (region-based scan, not shared code with the generator) and
// compares each field against printConfigDef()'s JSON output. Exits nonzero on
// any field mismatch and writes a JSON report.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { extractUpstreamDefs, OVERRIDE_KIND } from './verify-configdef/parseRegion.mjs';
import { fnBody, loadUpstreamRegions } from './verify-configdef/upstream.mjs';

const REPO = join(dirname(fileURLToPath(import.meta.url)), '..');

const args = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const reportIdx = process.argv.indexOf('--report');
const reportPath =
    reportIdx >= 0 ? process.argv[reportIdx + 1] : 'target/configdef-verification.json';
const jsonPath = args[0];
if (!jsonPath) {
    console.error('usage: verify-configdef.mjs <printConfigDef.json> [--report path]');
    process.exit(2);
}

const upstream = loadUpstreamRegions(REPO);
const { parsed, orderKeys } = extractUpstreamDefs(upstream);
const { src, bodyCommon, bodyFff, overrideKeys, regions } = upstream;

// ---------- compare ----------

const exported = JSON.parse(readFileSync(jsonPath, 'utf8'));
const byKey = new Map(exported.options.map((o) => [o.key, o]));

const FIELDS = [
    'label', 'full_label', 'category', 'tooltip', 'sidetext', 'enum_values',
    'enum_labels', 'enum_keys', 'min', 'max', 'max_literal', 'mode', 'gui_type',
    'gui_flags', 'multiline', 'full_width', 'is_code', 'readonly', 'nullable',
    'height', 'width', 'ratio_over', 'aliases', 'type',
];

function eq(a, b) {
    if (Array.isArray(a) || Array.isArray(b)) {
        return Array.isArray(a) && Array.isArray(b) && a.length === b.length && a.every((x, i) => x === b[i]);
    }
    return a === b;
}

const mismatches = [];
const checked = [];
for (const key of orderKeys) {
    const up = parsed.get(key);
    const got = byKey.get(key);
    if (!got) {
        mismatches.push({ key, field: '*', upstream: 'present', exported: 'missing' });
        continue;
    }
    for (const f of FIELDS) {
        let want = up[f];
        if (f === 'type') {
            // Upstream keeps the base coType plus a separate `nullable` flag;
            // the export folds nullability into OptionValueKind.
            want = up.nullable ? (OVERRIDE_KIND[up.type] ?? up.type) : up.type;
        }
        const have = got[f];
        if (!eq(want, have)) {
            mismatches.push({ key, field: f, upstream: want, exported: have });
        }
    }
    checked.push(key);
}
const missingInUpstream = [...byKey.keys()].filter((k) => !parsed.has(k));

// ---------- counts / exclusion accounting ----------

const countAdds = (body) => (body.match(/this->add\s*\(/g) || []).length;
const totalAddCallsInFile = (src.match(/this->add\s*\(/g) || []).length;
const slaBody = fnBody(src, 'init_sla_params');
const extruderBody = fnBody(src, 'init_extruder_option_keys');
const filamentKeysBody = fnBody(src, 'init_filament_option_keys');
const counts = {
    add_calls_total_in_file: totalAddCallsInFile,
    add_calls_init_common_params: countAdds(bodyCommon),
    add_calls_init_fff_params: countAdds(bodyFff),
    add_calls_init_extruder_option_keys: countAdds(extruderBody),
    add_calls_init_filament_option_keys: countAdds(filamentKeysBody),
    add_calls_init_sla_params: countAdds(slaBody),
    add_calls_outside_parsed_fns:
        totalAddCallsInFile - countAdds(bodyCommon) - countAdds(bodyFff)
        - countAdds(extruderBody) - countAdds(filamentKeysBody) - countAdds(slaBody),
    unique_keys_extracted: orderKeys.length,
    keys_exported: exported.options.length,
    duplicate_add_keys: [...parsed.keys()].filter((k) =>
        regions.filter((r) => r.key === k).length > 1),
};

// Reconcile upstream add-call count vs exported key count:
//   outside_parsed_fns = add() calls in sibling ConfigDef classes — not PrintConfigDef, out of scope.
//   init_sla_params    = SLA technology defs — out of scope for this slice.
//   duplicate_add_keys = upstream add() overwrites by key (last wins).
//   axis expansion     = the `machine_max_*` for-loop: 3 call sites -> 12 keys.
//   filament overrides = filament_extruder_override_keys loop adds nullable copies of existing keys.
const axisExpansion = 12 - 3;
counts.reconciliation = {
    print_config_def_add_calls:
        counts.add_calls_init_common_params + counts.add_calls_init_fff_params
        + counts.add_calls_init_extruder_option_keys + counts.add_calls_init_filament_option_keys,
    minus_duplicate_readds: counts.duplicate_add_keys.length,
    plus_axis_loop_expansion: axisExpansion,
    plus_filament_nullable_overrides: overrideKeys.length,
    equals_exported_keys:
        counts.add_calls_init_common_params + counts.add_calls_init_fff_params
        - counts.duplicate_add_keys.length + axisExpansion + overrideKeys.length,
};

const report = {
    generated_at: new Date().toISOString(),
    upstream_file: 'OrcaSlicer/src/libslic3r/PrintConfig.cpp',
    counts,
    excluded_keys: missingInUpstream.length
        ? { unexpected_exported_keys: missingInUpstream } : undefined,
    fields_compared_per_key: FIELDS,
    keys_checked: checked.length,
    mismatches,
    result: mismatches.length === 0 && missingInUpstream.length === 0 ? 'PASS' : 'FAIL',
};

mkdirSync(dirname(join(REPO, reportPath)), { recursive: true });
writeFileSync(join(REPO, reportPath), JSON.stringify(report, null, 2));
console.log(`checked ${checked.length} keys, ${mismatches.length} mismatches, report -> ${reportPath}`);
if (counts.add_calls_init_sla_params === 0) console.error('warning: SLA section not found');
process.exit(mismatches.length === 0 && missingInUpstream.length === 0 ? 0 : 1);
