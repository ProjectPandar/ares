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

import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO = join(dirname(fileURLToPath(import.meta.url)), '..');
const UPSTREAM = join(REPO, 'OrcaSlicer/src/libslic3r/PrintConfig.cpp');
const MATERIALS = join(REPO, 'OrcaSlicer/src/libslic3r/MaterialType.cpp');

const args = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const reportIdx = process.argv.indexOf('--report');
const reportPath =
    reportIdx >= 0 ? process.argv[reportIdx + 1] : 'target/configdef-verification.json';
const jsonPath = args[0];
if (!jsonPath) {
    console.error('usage: verify-configdef.mjs <printConfigDef.json> [--report path]');
    process.exit(2);
}

// ---------- shared-free upstream extraction (region-based) ----------

function stripComments(src) {
    let out = '';
    let i = 0;
    while (i < src.length) {
        const c = src[i];
        if (c === '"') {
            const s = i;
            i += 1;
            while (i < src.length && src[i] !== '"') {
                if (src[i] === '\\') i += 1;
                i += 1;
            }
            i += 1;
            out += src.slice(s, i);
        } else if (c === '/' && src[i + 1] === '/') {
            while (i < src.length && src[i] !== '\n') i += 1;
        } else if (c === '/' && src[i + 1] === '*') {
            i += 2;
            while (i < src.length && !(src[i] === '*' && src[i + 1] === '/')) {
                if (src[i] === '\n') out += '\n';
                i += 1;
            }
            i += 2;
        } else {
            out += c;
            i += 1;
        }
    }
    return out;
}

function decodeStr(text) {
    const re = /(?:u8)?"((?:[^"\\]|\\.)*)"/g;
    let m;
    let out = '';
    let found = false;
    while ((m = re.exec(text)) !== null) {
        found = true;
        out += m[1].replace(/\\(u[0-9a-fA-F]{4}|x[0-9a-fA-F]{1,4}|n|r|t|0|\\|"|')/g,
            (_, e) => {
                if (e === 'n') return '\n';
                if (e === 'r') return '\r';
                if (e === 't') return '\t';
                if (e === '0') return '\0';
                if (e === '\\') return '\\';
                if (e === '"') return '"';
                if (e === "'") return "'";
                if (e[0] === 'u' || e[0] === 'x') return String.fromCodePoint(parseInt(e.slice(1), 16));
                return e;
            });
    }
    return found ? out : null;
}

const src = stripComments(readFileSync(UPSTREAM, 'utf8'));

// enum constant -> int, from all libslic3r enum declarations + s_keys_map.
function enumTables() {
    const vals = new Map();
    for (const file of readdirSync(join(REPO, 'OrcaSlicer/src/libslic3r')).filter((f) => f.endsWith('.hpp'))) {
        const h = stripComments(readFileSync(join(REPO, 'OrcaSlicer/src/libslic3r', file), 'utf8'));
        const re = /enum(?:\s+class)?\s+([A-Za-z_]\w*)[^{]*\{([^}]*)\}/g;
        let m;
        while ((m = re.exec(h)) !== null) {
            let next = 0;
            for (const item of m[2].split(',')) {
                const t = item.trim();
                if (!t) continue;
                const eq = /^(\w+)\s*=\s*(-?\d+|0x[\da-fA-F]+)/.exec(t);
                if (eq) {
                    next = parseInt(eq[2]);
                    vals.set(eq[1], next);
                    vals.set(`${m[1]}::${eq[1]}`, next);
                } else if (/^\w+$/.test(t)) {
                    vals.set(t, next);
                    vals.set(`${m[1]}::${t}`, next);
                }
                next += 1;
            }
        }
    }
    return vals;
}
const ENUM_VALS = enumTables();

const KEYS_MAPS = new Map();
for (const m of src.matchAll(/s_keys_map_(\w+)\s*=?\s*\{([\s\S]*?)\};/g)) {
    const entries = [];
    for (const e of m[2].matchAll(/\{\s*"([^"]+)"\s*,\s*([^}]+?)\s*\}/g)) {
        let tok = e[2].trim();
        const cast = /^\(\s*[\w:]+\s*\)\s*(.+)$/.exec(tok);
        if (cast) tok = cast[1].trim();
        const im = /^int\s*\(\s*([\w:]+)\s*\)$/.exec(tok);
        if (im) tok = im[1];
        tok = tok.replace(/\s+/g, '');
        const v = ENUM_VALS.get(tok);
        if (v === undefined) throw new Error(`keys-map const ${tok} unresolved`);
        entries.push([e[1], v]);
    }
    KEYS_MAPS.set(m[1], entries);
}

const MATERIAL_NAMES = (() => {
    const msrc = readFileSync(MATERIALS, 'utf8');
    const sec = msrc.slice(msrc.indexOf('material_types = {'), msrc.indexOf('};', msrc.indexOf('material_types = {')));
    return [...sec.matchAll(/\{\s*"([^"]+)"/g)].map((x) => x[1]);
})();

const OVERRIDE_KEYS = (() => {
    const m = /filament_extruder_override_keys = \{([\s\S]*?)\};/.exec(src);
    return [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]);
})();

// Locate each init fn body and slice it.
function fnBody(name) {
    const start = src.indexOf(`PrintConfigDef::${name}()`);
    const open = src.indexOf('{', start);
    let depth = 0;
    for (let i = open; i < src.length; i += 1) {
        const c = src[i];
        if (c === '"') {
            i += 1;
            while (i < src.length && src[i] !== '"') {
                if (src[i] === '\\') i += 1;
                i += 1;
            }
        } else if (c === '{') depth += 1;
        else if (c === '}') {
            depth -= 1;
            if (depth === 0) return src.slice(open + 1, i);
        }
    }
    throw new Error(`unbalanced ${name}`);
}

// Split into per-def regions: each `add(` starts a new region; text belongs to
// the most recent def. Loop bodies are expanded separately below.
const bodyCommon = fnBody('init_common_params');
const bodyFff = fnBody('init_fff_params');
const regionSrc = bodyCommon + '\n// __SPLIT__\n' + bodyFff;

// Expand loop-generated defs: replace the for-loop bodies with one synthetic
// region per generated key.
function expandRegions(body) {
    // Axes loop: expand per axis; format strings use the (void)L() pool order.
    const axes = {
        x: { feed: '500,200', accel: '1000,1000', jerk: '10,10' },
        y: { feed: '500,200', accel: '1000,1000', jerk: '10,10' },
        z: { feed: '12,12', accel: '500,200', jerk: '0.2,0.4' },
        e: { feed: '120,120', accel: '5000,5000', jerk: '2.5,2.5' },
    };
    const out = [];
    const axisLoop = /for \(const AxisDefault &axis : axes\) \{([\s\S]*?)\n        \}/.exec(body);
    let loopBody = axisLoop ? axisLoop[1] : null;
    let segments = [];
    if (axisLoop) {
        // split the loop body into the three def regions by `this->add(`
        const parts = loopBody.split(/(?=def\s*=\s*this->add\()/).filter((p) => p.includes('this->add('));
        for (const axis of Object.keys(axes)) {
            const upper = axis.toUpperCase();
            for (const part of parts) {
                let p = part
                    .replace(/"([^"]+)"\s*\+\s*axis\.name/g, `"$1${axis}"`)
                    .replace(/\(boost::format\(("(?:[^"\\]|\\.)*")\)\s*%\s*axis_upper\)\.str\(\)/g,
                        (_, lit) => `L(${lit.replace('%1%', upper)})`);
                p = p.replace(/axis\.(max_feedrate|max_acceleration|max_jerk)/g,
                    (_, f) => `std::vector<double>{${axes[axis][f === 'max_feedrate' ? 'feed' : f === 'max_acceleration' ? 'accel' : 'jerk']}}`);
                out.push(p);
            }
        }
        body = body.replace(axisLoop[0], '');
    }
    // MaterialType::all() loop inside filament_type's region: inline-expand.
    body = body.replace(
        /for \(const auto& filament : MaterialType::all\(\)\) \{\s*def->enum_values\.push_back\(filament\.name\);\s*\}/g,
        MATERIAL_NAMES.map((n) => `def->enum_values.push_back("${n}");`).join('\n'));
    // filament override loop: expand to per-key regions.
    const ovLoop = /for \(auto& opt_key : filament_extruder_override_keys\) \{([\s\S]*?)\n    \}/.exec(body);
    if (ovLoop) {
        body = body.replace(ovLoop[0], '');
        for (const key of OVERRIDE_KEYS) {
            out.push(`__OVERRIDE__:${key}`);
        }
    }
    return { body, extra: out };
}

const expCommon = expandRegions(bodyCommon);
const expFff = expandRegions(bodyFff);
const allText = expCommon.body + '\n' + expFff.body
    + '\n' + expCommon.extra.join('\n') + '\n' + expFff.extra.join('\n');

// Split into regions, each starting at a `this->add(` occurrence.
const regions = [];
{
    const re = /(?:auto\s+(\w+)\s*=\s*)?def\s*=\s*this->(add|add_nullable)\s*\(\s*"([^"]+)"\s*,\s*(co\w+)/g;
    const marks = [];
    let m;
    while ((m = re.exec(allText)) !== null) {
        marks.push({ pos: m.index, alias: m[1] || null, nullable: m[2] === 'add_nullable', key: m[3], co: m[4] });
    }
    for (let i = 0; i < marks.length; i += 1) {
        const end = i + 1 < marks.length ? marks[i + 1].pos : allText.length;
        regions.push({ ...marks[i], text: allText.slice(marks[i].pos, end) });
    }
}

// alias name -> key (def_x = def = add)
const aliasKey = new Map();
for (const r of regions) if (r.alias) aliasKey.set(r.alias, r.key);

const CO_TO_KIND = {
    coFloat: 'float', coFloats: 'floats', coFloatsNullable: 'floats_nullable',
    coFloatOrPercent: 'float_or_percent', coFloatsOrPercents: 'floats_or_percents',
    coFloatsOrPercentsNullable: 'floats_or_percents_nullable',
    coPercent: 'percent', coPercents: 'percents', coPercentsNullable: 'percents_nullable',
    coInt: 'int', coInts: 'ints', coIntsNullable: 'ints_nullable',
    coBool: 'bool', coBools: 'bools', coBoolsNullable: 'bools_nullable',
    coEnum: 'enum', coEnums: 'enums', coEnumsNullable: 'enums_nullable',
    coString: 'string', coStrings: 'strings',
    coPoint: 'point', coPoint3: 'point3', coPoints: 'points', coPointsGroups: 'points_groups',
};
const MODE = { comSimple: 'simple', comAdvanced: 'advanced', comExpert: 'expert', comDevelop: 'develop' };
const GUI = {
    undefined: 'undefined', i_enum_open: 'i_enum_open', f_enum_open: 'f_enum_open',
    color: 'color', select_open: 'select_open', slider: 'slider', legend: 'legend', one_string: 'one_string',
};

function numOf(tok) {
    let t = tok.trim().replace(/f$/, '');
    if (t === 'max_temp') return 1500;
    const neg = t.startsWith('-');
    if (neg) t = t.slice(1).trim();
    if (t === 'max_temp') return neg ? -1500 : 1500;
    const v = Number(t);
    if (Number.isNaN(v)) throw new Error(`num token ${tok}`);
    return neg ? -v : v;
}

function parseRegion(region) {
    const t = region.text;
    const rec = {
        key: region.key, type: CO_TO_KIND[region.co], nullable: region.nullable,
    };
    // Capture `def->name = <rhs>` where <rhs> runs to the first `;` that is
    // not inside a string literal (tooltips legitimately contain `;`).
    const rhsText = (name) => {
        const re = new RegExp(`def->${name}\\s*=`, 'g');
        let m; let last = null;
        while ((m = re.exec(t)) !== null) {
            let i = m.index + m[0].length;
            const start = i;
            while (i < t.length) {
                const c = t[i];
                if (c === '"') {
                    i += 1;
                    while (i < t.length && t[i] !== '"') {
                        if (t[i] === '\\') i += 1;
                        i += 1;
                    }
                } else if (c === ';') break;
                i += 1;
            }
            last = t.slice(start, i);
        }
        return last;
    };
    const strField = (name) => {
        const last = rhsText(name);
        if (last === null) return null;
        const s = decodeStr(last);
        if (s !== null) return s;
        const cm = /^(\w+)->(\w+)$/.exec(last.trim());
        if (cm && aliasKey.has(cm[1])) return getField(aliasKey.get(cm[1]), cm[2]);
        return `__UNPARSED__${last}`;
    };
    const boolField = (name) => {
        const re = new RegExp(`def->${name}\\s*=\\s*(true|false);`);
        const m = re.exec(t);
        return m ? m[1] === 'true' : null;
    };
    const numField = (name) => {
        const last = rhsText(name);
        if (last === null) return null;
        const cm = /^(\w+)->(\w+)$/.exec(last.trim());
        if (cm && aliasKey.has(cm[1])) return getField(aliasKey.get(cm[1]), cm[2]);
        return numOf(last);
    };
    rec.label = strField('label');
    rec.full_label = strField('full_label');
    rec.category = strField('category');
    rec.tooltip = strField('tooltip');
    rec.sidetext = strField('sidetext');
    rec.gui_flags = strField('gui_flags') ?? '';
    rec.ratio_over = strField('ratio_over');
    rec.min = numField('min');
    rec.max = numField('max');
    rec.max_literal = numField('max_literal');
    rec.height = numField('height');
    rec.width = numField('width');
    for (const f of ['multiline', 'full_width', 'is_code', 'readonly']) {
        const v = boolField(f);
        if (v !== null) rec[f] = v; else rec[f] = false;
    }
    {
        const m = /def->nullable\s*=\s*(true|false)/.exec(t);
        if (m) rec.nullable = m[1] === 'true';
    }
    {
        const m = /def->mode\s*=\s*(com\w+)/.exec(t);
        rec.mode = m ? MODE[m[1]] : 'simple';
    }
    {
        const m = /def->gui_type\s*=\s*ConfigOptionDef::GUIType::(\w+)/.exec(t);
        const cm = /def->gui_type\s*=\s*(\w+)->gui_type/.exec(t);
        if (m) rec.gui_type = GUI[m[1]];
        else if (cm && aliasKey.has(cm[1])) rec.gui_type = getField(aliasKey.get(cm[1]), 'gui_type');
        else rec.gui_type = 'undefined';
    }
    rec.enum_values = [];
    rec.enum_labels = [];
    for (const m of t.matchAll(/def->enum_values\.(?:push_back|emplace_back)\s*\(([^;]*)\);/g)) {
        rec.enum_values.push(decodeStr(m[1]));
    }
    for (const m of t.matchAll(/def->enum_labels\.(?:push_back|emplace_back)\s*\(([^;]*)\);/g)) {
        rec.enum_labels.push(decodeStr(m[1]));
    }
    for (const m of t.matchAll(/def->(enum_values|enum_labels)\s*=\s*\{([\s\S]*?)\};/g)) {
        rec[m[1]] = [...m[2].matchAll(/(?:L\()?"((?:[^"\\]|\\.)*)"\)?/g)].map((x) => x[1]);
    }
    for (const m of t.matchAll(/def->(enum_values|enum_labels)\s*=\s*(\w+)->\1/g)) {
        rec[m[1]] = getField(aliasKey.get(m[2]), m[1]);
    }
    for (const m of t.matchAll(/append\(def->(enum_values|enum_labels),\s*(\w+)->\1\)/g)) {
        rec[m[1]].push(...getField(aliasKey.get(m[2]), m[1]));
    }
    {
        const m = /def->enum_keys_map\s*=\s*&(?:ConfigOptionEnum<(\w+)>::get_enum_values\(\)|s_keys_map_(\w+))/.exec(t);
        if (m) {
            const entries = KEYS_MAPS.get(m[1] || m[2]);
            rec.enum_keys = rec.enum_values.map((v) => {
                const hit = entries.find(([k]) => k === v);
                return hit ? hit[1] : -1;
            });
            rec._map = entries;
        } else rec.enum_keys = [];
    }
    {
        const m = /def->aliases\s*=\s*\{([^}]*)\}/.exec(t);
        rec.aliases = m ? [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : [];
    }
    return rec;
}

// Resolve aliases before comparing (alias target regions may appear earlier).
const parsed = new Map();
const orderKeys = [];
function getField(key, field) {
    const r = parsed.get(key);
    if (!r) throw new Error(`alias target ${key} not parsed yet`);
    return r[field];
}
for (const region of regions) {
    const rec = parseRegion(region);
    if (!parsed.has(rec.key)) orderKeys.push(rec.key);
    parsed.set(rec.key, rec); // upstream add() overwrites by key
}
// filament_* nullable overrides copy fields from their unprefixed source.
const OVERRIDE_SIMPLE = new Set([
    'filament_retraction_length', 'filament_z_hop',
    'filament_long_retractions_when_cut', 'filament_retraction_distances_when_cut',
]);
const OVERRIDE_KIND = {
    floats: 'floats_nullable', floats_or_percents: 'floats_or_percents_nullable',
    percents: 'percents_nullable', ints: 'ints_nullable', bools: 'bools_nullable',
    enums: 'enums_nullable',
};
for (const key of OVERRIDE_KEYS) {
    const src = parsed.get(key.replace(/^filament_/, ''));
    if (!src) throw new Error(`override source missing for ${key}`);
    const rec = { ...src, key, type: OVERRIDE_KIND[src.type] ?? src.type, nullable: true };
    rec.enum_values = src.enum_values.slice();
    rec.enum_labels = src.enum_labels.slice();
    rec.enum_keys = src.enum_keys.slice();
    rec.mode = OVERRIDE_SIMPLE.has(key) ? 'simple' : 'advanced';
    if (!parsed.has(key)) orderKeys.push(key);
    parsed.set(key, rec);
}

// ---------- compare ----------

const exported = JSON.parse(readFileSync(jsonPath, 'utf8'));
const byKey = new Map(exported.options.map((o) => [o.key, o]));

const FIELDS = [
    'label', 'full_label', 'category', 'tooltip', 'sidetext', 'enum_values',
    'enum_labels', 'enum_keys', 'min', 'max', 'max_literal', 'mode', 'gui_type',
    'gui_flags', 'multiline', 'full_width', 'is_code', 'readonly', 'nullable',
    'height', 'width', 'ratio_over', 'aliases', 'type',
];
// upstream-side normalized comparisons
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
const slaBody = fnBody('init_sla_params');
const extruderBody = fnBody('init_extruder_option_keys');
const filamentKeysBody = fnBody('init_filament_option_keys');
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
//   outside_parsed_fns = add() calls in sibling ConfigDef classes
//   (TimestampsConfigDef, DimensionsConfigDef, OtherPresetsConfigDef,
//   CustomGcodeSpecificConfigDef, ...) — not PrintConfigDef, out of scope.
//   init_sla_params    = SLA technology defs — out of scope for this slice.
//   duplicate_add_keys = upstream add() overwrites by key (last wins).
//   axis expansion     = the `machine_max_*` for-loop: 3 call sites -> 12 keys.
//   filament overrides = filament_extruder_override_keys loop adds nullable
//   copies of existing keys.
const axisExpansion = 12 - 3;
counts.reconciliation = {
    print_config_def_add_calls:
        counts.add_calls_init_common_params + counts.add_calls_init_fff_params
        + counts.add_calls_init_extruder_option_keys + counts.add_calls_init_filament_option_keys,
    minus_duplicate_readds: counts.duplicate_add_keys.length,
    plus_axis_loop_expansion: axisExpansion,
    plus_filament_nullable_overrides: OVERRIDE_KEYS.length,
    equals_exported_keys:
        counts.add_calls_init_common_params + counts.add_calls_init_fff_params
        - counts.duplicate_add_keys.length + axisExpansion + OVERRIDE_KEYS.length,
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
