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

import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const REPO = join(dirname(fileURLToPath(import.meta.url)), '..');
const UPSTREAM = join(REPO, 'OrcaSlicer/src/libslic3r/PrintConfig.cpp');
const MATERIALS = join(REPO, 'OrcaSlicer/src/libslic3r/MaterialType.cpp');
const OUT_DIR = join(REPO, 'crates/ares-core/src/options/registry/definitions');

const args = process.argv.slice(2);
const CHECK = args.includes('--check');
const DUMP = args.includes('--dump');

// ---------- helpers ----------

function stripComments(src) {
    // Remove // and /* */ comments outside of string literals.
    let out = '';
    let i = 0;
    const n = src.length;
    while (i < n) {
        const c = src[i];
        if (c === '"') {
            const start = i;
            i += 1;
            while (i < n && src[i] !== '"') {
                if (src[i] === '\\') i += 1;
                i += 1;
            }
            i += 1;
            out += src.slice(start, i);
        } else if (c === '/' && src[i + 1] === '/') {
            while (i < n && src[i] !== '\n') i += 1;
        } else if (c === '/' && src[i + 1] === '*') {
            i += 2;
            while (i < n && !(src[i] === '*' && src[i + 1] === '/')) {
                if (src[i] === '\n') out += '\n';
                i += 1;
            }
            i += 2;
        } else {
            out += c;
            i += 1;
        }
    }
    // Drop dead `#if 0` blocks and directive lines so they cannot glue onto
    // the following statement.
    out = out.replace(/#if\s+0[\s\S]*?#endif/g, '\n');
    out = out.replace(/^\s*#(include|define|if|ifdef|ifndef|elif|else|endif)\b[^\n]*/gm, '');
    return out;
}

// Decode a C++ string literal sequence such as `"abc" "def"` or u8"..." into
// the UTF-8 string. Adjacent literals concatenate.
function decodeStringLiterals(text) {
    const re = /(?:u8)?"((?:[^"\\]|\\.)*)"/g;
    let m;
    let out = '';
    let found = false;
    while ((m = re.exec(text)) !== null) {
        found = true;
        out += decodeCStringBody(m[1]);
    }
    return found ? out : null;
}

function decodeCStringBody(body) {
    let out = '';
    let i = 0;
    while (i < body.length) {
        const c = body[i];
        if (c !== '\\') {
            out += c;
            i += 1;
            continue;
        }
        i += 1;
        const e = body[i];
        switch (e) {
            case 'n': out += '\n'; i += 1; break;
            case 'r': out += '\r'; i += 1; break;
            case 't': out += '\t'; i += 1; break;
            case '0': out += '\0'; i += 1; break;
            case '\\': out += '\\'; i += 1; break;
            case '"': out += '"'; i += 1; break;
            case "'": out += "'"; i += 1; break;
            case 'x': {
                const m = /^[0-9a-fA-F]{1,4}/.exec(body.slice(i + 1));
                out += String.fromCodePoint(parseInt(m[0], 16));
                i += 1 + m[0].length;
                break;
            }
            case 'u': {
                out += String.fromCodePoint(parseInt(body.slice(i + 1, i + 5), 16));
                i += 5;
                break;
            }
            case 'U': {
                out += String.fromCodePoint(parseInt(body.slice(i + 1, i + 9), 16));
                i += 9;
                break;
            }
            default:
                out += e;
                i += 1;
        }
    }
    return out;
}

// Extract the argument list of a call starting at index `open` (position of '(').
function callArgs(src, open) {
    let depth = 0;
    let i = open;
    const start = open + 1;
    for (; i < src.length; i += 1) {
        const c = src[i];
        if (c === '"') {
            i += 1;
            while (i < src.length && src[i] !== '"') {
                if (src[i] === '\\') i += 1;
                i += 1;
            }
        } else if (c === '(') depth += 1;
        else if (c === ')') {
            depth -= 1;
            if (depth === 0) return [src.slice(start, i), i];
        }
    }
    throw new Error('unbalanced parens');
}

function splitArgs(text) {
    const parts = [];
    let depth = 0;
    let cur = '';
    for (let i = 0; i < text.length; i += 1) {
        const c = text[i];
        if (c === '"') {
            const start = i;
            i += 1;
            while (i < text.length && text[i] !== '"') {
                if (text[i] === '\\') i += 1;
                i += 1;
            }
            cur += text.slice(start, i + 1);
            continue;
        }
        if (c === '(' || c === '{' || c === '[') depth += 1;
        if (c === ')' || c === '}' || c === ']') depth -= 1;
        if (c === ',' && depth === 0) {
            parts.push(cur.trim());
            cur = '';
        } else {
            cur += c;
        }
    }
    if (cur.trim() !== '') parts.push(cur.trim());
    return parts;
}

// ---------- upstream enum int resolution ----------

// Parse `enum class Name { A, B = 4, ... }` / `enum Name {...}` declarations in
// libslic3r headers so constants in s_keys_map_* tables resolve to ints.
function collectEnumValues() {
    const headers = readdirSync(join(REPO, 'OrcaSlicer/src/libslic3r'))
        .filter((f) => f.endsWith('.hpp'));
    const values = new Map(); // unqualified constant -> int ; qualified "Name::C" -> int
    for (const file of headers) {
        const src = stripComments(readFileSync(join(REPO, 'OrcaSlicer/src/libslic3r', file), 'utf8'));
        const re = /enum(?:\s+class)?\s+([A-Za-z_][A-Za-z0-9_]*)[^{]*\{([^}]*)\}/g;
        let m;
        while ((m = re.exec(src)) !== null) {
            const name = m[1];
            const body = m[2];
            let next = 0;
            for (const item of body.split(',')) {
                const t = item.trim();
                if (!t) continue;
                const eq = /^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(-?\d+|0x[0-9a-fA-F]+)\s*$/.exec(t);
                if (eq) {
                    next = parseInt(eq[2]);
                    values.set(eq[1], next);
                    values.set(`${name}::${eq[1]}`, next);
                } else if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(t)) {
                    values.set(t, next);
                    values.set(`${name}::${t}`, next);
                }
                next += 1;
            }
        }
    }
    return values;
}

// ---------- parse PrintConfig.cpp ----------

const raw = readFileSync(UPSTREAM, 'utf8');
const src = stripComments(raw);

// Constants used in numeric assignments. max_temp is a `const int` inside
// init_fff_params; INITIAL_* are #define'd in PrintConfigConstants.hpp.
const constNumbers = { max_temp: 1500 };
{
    const cc = readFileSync(join(REPO, 'OrcaSlicer/src/libslic3r/PrintConfigConstants.hpp'), 'utf8');
    for (const m of cc.matchAll(/#define\s+([A-Z_][A-Z0-9_]*)\s+(-?[0-9.]+|true|false)/g)) {
        constNumbers[m[1]] = m[2] === 'true' ? true : m[2] === 'false' ? false : Number(m[2]);
    }
}

// MaterialType::all() names from MaterialType.cpp.
function materialNames() {
    const msrc = readFileSync(MATERIALS, 'utf8');
    const sec = msrc.slice(msrc.indexOf('material_types = {'));
    const names = [];
    const re = /\{\s*"([^"]+)"/g;
    let m;
    while ((m = re.exec(sec)) !== null) {
        names.push(m[1]);
        if (sec.indexOf('};', m.index) < 0 || m.index > sec.indexOf('};')) break;
    }
    return names;
}

// Parse s_keys_map_<Name> tables in PrintConfig.cpp -> Map name -> [{key, value}]
function keysMaps(enumValues) {
    const maps = new Map();
    const re = /s_keys_map_([A-Za-z_][A-Za-z0-9_]*)\s*=?\s*\{([\s\S]*?)\};/g;
    let m;
    while ((m = re.exec(src)) !== null) {
        const entries = [];
        const ere = /\{\s*"([^"]+)"\s*,\s*([^}]+?)\s*\}/g;
        let e;
        while ((e = ere.exec(m[2])) !== null) {
            let tok = e[2].trim();
            let qualified = /^int\s*\(\s*([A-Za-z_][A-Za-z0-9_:]*)::([A-Za-z_][A-Za-z0-9_]*)\s*\)$/.exec(tok);
            let val;
            if (qualified) {
                val = enumValues.get(`${qualified[1]}::${qualified[2]}`);
            } else {
                val = enumValues.get(tok);
            }
            if (val === undefined) {
                throw new Error(`cannot resolve enum constant ${tok} in s_keys_map_${m[1]}`);
            }
            entries.push({ key: e[1], value: val });
        }
        maps.set(m[1], entries);
    }
    return maps;
}

const CO_TO_KIND = {
    coFloat: 'Float',
    coFloats: 'Floats',
    coFloatsNullable: 'FloatsNullable',
    coFloatOrPercent: 'FloatOrPercent',
    coFloatsOrPercents: 'FloatsOrPercents',
    coFloatsOrPercentsNullable: 'FloatsOrPercentsNullable',
    coPercent: 'Percent',
    coPercents: 'Percents',
    coPercentsNullable: 'PercentsNullable',
    coInt: 'Int',
    coInts: 'Ints',
    coIntsNullable: 'IntsNullable',
    coBool: 'Bool',
    coBools: 'Bools',
    coBoolsNullable: 'BoolsNullable',
    coEnum: 'Enum',
    coEnums: 'Enums',
    coEnumsNullable: 'EnumsNullable',
    coString: 'String',
    coStrings: 'Strings',
    coPoint: 'Point',
    coPoint3: 'Point3',
    coPoints: 'Points',
    coPointsGroups: 'PointsGroups',
};

const NULLABLE_MAP = {
    coFloats: 'coFloatsNullable',
    coFloatsOrPercents: 'coFloatsOrPercentsNullable',
    coPercents: 'coPercentsNullable',
    coInts: 'coIntsNullable',
    coBools: 'coBoolsNullable',
    coEnums: 'coEnumsNullable',
};

const MODE_MAP = {
    comSimple: 'simple',
    comAdvanced: 'advanced',
    comExpert: 'expert',
    comDevelop: 'develop',
};

// C++ ostream default formatting (%.6g).
function fmtDouble(v) {
    if (!Number.isFinite(v)) return 'nan';
    let s = v.toPrecision(6);
    // toPrecision keeps trailing zeros; strip like %g
    if (s.includes('e') || s.includes('E')) {
        const [mant, exp] = s.split(/e/i);
        let mm = mant.replace(/\.?0+$/, '');
        let e = parseInt(exp, 10);
        // C++ uses e+NN/e-NN with at least two exponent digits
        const sign = e < 0 ? '-' : '+';
        s = `${mm}e${sign}${String(Math.abs(e)).padStart(2, '0')}`;
        return s;
    }
    s = s.replace(/\.?0+$/, '');
    if (s === '-0') s = '0';
    return s === '' ? '0' : s;
}

function fmtNum(tok) {
    tok = tok.trim();
    let neg = false;
    if (tok.startsWith('-')) {
        neg = true;
        tok = tok.slice(1).trim();
    }
    tok = tok.replace(/f$/, '');
    tok = tok.replace(/^\(int\)\s*\(([A-Za-z_][A-Za-z0-9_:]*)\)$/, '$1');
    if (/nil_value/.test(tok)) return 'nil';
    let v;
    if (ENUM_VALUES.has(tok)) {
        v = ENUM_VALUES.get(tok);
    } else if (tok in constNumbers) {
        v = constNumbers[tok];
        if (typeof v === 'boolean') v = v ? 1 : 0;
    } else {
        v = Number(tok);
        if (Number.isNaN(v)) throw new Error(`unparsed number token: ${tok}`);
    }
    if (neg) v = -v;
    return Number.isInteger(v) ? String(v) : fmtDouble(v);
}

// Serialize a default value from `new ConfigOptionX{...}` per upstream rules.
function serializeDefault(ctor, argText, ctx) {
    const t = ctor.trim();
    const args = splitArgs(argText);
    const numList = () => args.length === 0 ? '' : args.map(fmtNum).join(',');
    const strList = () => args.map((a) => decodeStringLiterals(a) ?? a.replace(/^"|"$/g, ''));
    switch (t) {
        case 'ConfigOptionBool':
        case 'ConfigOptionBoolStatic': {
            let v = args[0].trim();
            if (v in constNumbers) v = String(constNumbers[v]);
            return v === 'true' ? 'true' : v === 'false' ? 'false' : v;
        }
        case 'ConfigOptionBools':
            return args.map((a) => (a.trim() === 'true' ? '1' : '0')).join(',');
        case 'ConfigOptionBoolsNullable':
            return args.map((a) => (a.trim() === 'true' ? '1' : '0')).join(',');
        case 'ConfigOptionInt':
            return fmtNum(args[0]);
        case 'ConfigOptionInts':
        case 'ConfigOptionIntsNullable':
            return numList();
        case 'ConfigOptionFloat':
            return fmtNum(args[0]);
        case 'ConfigOptionFloats':
        case 'ConfigOptionFloatsNullable':
            return numList();
        case 'ConfigOptionPercent':
            return `${fmtNum(args[0])}%`;
        case 'ConfigOptionPercents':
        case 'ConfigOptionPercentsNullable':
            return args.map((a) => {
                const n = fmtNum(a);
                return n === 'nil' ? 'nil' : `${n}%`;
            }).join(',');
        case 'ConfigOptionFloatOrPercent': {
            // (value, percent)
            const v = args[0].trim();
            if (/^(true|false)$/.test(v) && args.length === 1) return v; // bool ctor? unlikely
            const pct = args.length > 1 && args[1].trim() === 'true';
            return `${fmtNum(v)}${pct ? '%' : ''}`;
        }
        case 'ConfigOptionFloatsOrPercents':
        case 'ConfigOptionFloatsOrPercentsNullable': {
            // Elements may be numbers or FloatOrPercent(v, pct)
            return args.map((a) => {
                const fm = /^FloatOrPercent\s*\(\s*([^,]+)\s*,\s*(true|false)\s*\)$/.exec(a);
                if (fm) return `${fmtNum(fm[1])}${fm[2] === 'true' ? '%' : ''}`;
                return fmtNum(a);
            }).join(',');
        }
        case 'ConfigOptionString':
        case 'ConfigOptionStrings':
            return strList().join(';');
        case 'ConfigOptionStringsNullable':
            return strList().join(';');
        case 'ConfigOptionPoint': {
            const m = /^Vec2d\s*\(([^,]+),\s*([^)]+)\)$/.exec(args[0]);
            if (m) return `${fmtNum(m[1])}x${fmtNum(m[2])}`;
            return args.join(',');
        }
        case 'ConfigOptionPoints': {
            // { Vec2d(x,y), ... }
            const pts = [];
            const re = /Vec2d\s*\(([^,]+),\s*([^)]+)\)/g;
            let m;
            while ((m = re.exec(argText)) !== null) {
                pts.push(`${fmtNum(m[1])}x${fmtNum(m[2])}`);
            }
            return pts.join(',');
        }
        case 'ConfigOptionPointsGroups':
        case 'ConfigOptionPointsGroup':
            return '';
        case 'ConfigOptionEnum': {
            // resolve enum constant to its string key via the current def's keys map
            const tok = args.length === 2 ? args[1].trim() : args[0].trim();
            return resolveEnumKey(ctx, tok);
        }
        case 'ConfigOptionEnumsGeneric':
        case 'ConfigOptionEnumsGenericNullable':
            // Elements are either string literals or enum constants resolved
            // through the def's keys map.
            return args.map((a) => {
                const s = decodeStringLiterals(a);
                if (s !== null) return s;
                return resolveEnumKey(ctx, a.trim());
            }).join(',');
        default:
            throw new Error(`unknown default ctor ${t} (${ctx.key})`);
    }
}

function resolveEnumKey(ctx, tok) {
    tok = argumentBindings?.[tok.trim()] ?? tok;
    let name = tok.trim();
    // Strip a leading C-style cast: (int)X, (BedType)X, ...
    const cast = /^\(\s*[A-Za-z_][A-Za-z0-9_:]*\s*\)\s*(.+)$/.exec(name);
    if (cast) name = cast[1].trim();
    name = name.replace(/\s+/g, '');
    if (!/^[A-Za-z_][A-Za-z0-9_:]*$/.test(name)) {
        throw new Error(`unparsed enum constant ${tok} for ${ctx.key}`);
    }
    const entries = ctx.enumKeysEntries || [];
    const v = ENUM_VALUES.get(name);
    if (v === undefined) throw new Error(`unresolved default enum constant ${tok} for ${ctx.key}`);
    const hit = entries.find((e) => e.value === v);
    if (!hit) {
        // The map may not cover the constant (e.g. values pushed without a
        // map entry); serialize() upstream would throw, so fail loudly.
        throw new Error(`default enum constant ${tok} not in keys map for ${ctx.key}`);
    }
    return hit.key;
}

const ENUM_VALUES = collectEnumValues();
const KEYS_MAPS = keysMaps(ENUM_VALUES);
const MATERIAL_NAMES = materialNames();

// ---------- statement stream ----------

const defs = new Map(); // key -> record
const order = [];
const aliases = new Map(); // def variable name -> record
const groups = {};

let phase = 'fff'; // printer_technology tagging: common defs get 'unknown'->'fff' unless in sla

function newDef(key, type, nullable) {
    if (!CO_TO_KIND[type]) throw new Error(`unmapped ConfigOptionType ${type} for ${key}`);
    const rec = {
        key,
        type: CO_TO_KIND[type],
        nullable,
            printer_technology: 'unknown',
        label: null,
        full_label: null,
        category: null,
        tooltip: null,
        sidetext: null,
        enum_values: [],
        enum_labels: [],
        enum_keys: [],
        min: null,
        max: null,
        max_literal: null,
        mode: 'simple',
        gui_type: 'undefined',
        gui_flags: '',
        multiline: false,
        full_width: false,
        is_code: false,
        readonly: false,
        height: null,
        width: null,
        ratio_over: null,
        aliases: [],
        shortcut: [],
        default_value: null,
        _typeCo: type,
    };
    if (!defs.has(key)) order.push(key);
    defs.set(key, rec);
    return rec;
}

function fieldAssign(rec, field, rhs) {
    rhs = argumentBindings?.[rhs.trim()] ?? rhs;
    const str = decodeStringLiterals(rhs);
    switch (field) {
        case 'label':
        case 'full_label':
        case 'category':
        case 'tooltip':
        case 'sidetext':
        case 'gui_flags':
        case 'ratio_over': {
            if (str !== null) {
                rec[field] = str;
                return;
            }
            const cm = /^([A-Za-z0-9_]+)->([a-z_]+)$/.exec(rhs.trim());
            if (cm && aliases.has(cm[1])) {
                rec[field] = aliases.get(cm[1])[cm[2]];
                return;
            }
            const it = /^it_opt->second\.([a-z_]+)$/.exec(rhs.trim());
            if (it) {
                rec[field] = ctxCopySource[it[1]];
                return;
            }
            throw new Error(`string field ${field} unparsed rhs: ${rhs} (${rec.key})`);
        }
        case 'min':
        case 'max':
        case 'max_literal':
        case 'height':
        case 'width': {
            const cm = /^([A-Za-z0-9_]+)->([a-z_]+)$/.exec(rhs.trim());
            if (cm && aliases.has(cm[1])) {
                rec[field] = aliases.get(cm[1])[cm[2]];
                return;
            }
            const it = /^it_opt->second\.([a-z_]+)$/.exec(rhs.trim());
            if (it) {
                rec[field] = ctxCopySource[it[1]];
                return;
            }
            rec[field] = Number(fmtNum(rhs));
            return;
        }
        case 'mode': {
            const trimmed = rhs.trim();
            if (MODE_MAP[trimmed]) {
                rec.mode = MODE_MAP[trimmed];
                return;
            }
            throw new Error(`mode unparsed: ${rhs} (${rec.key})`);
        }
        case 'gui_type': {
            const m = /GUIType::([a-z_]+)/.exec(rhs);
            if (m) {
                rec.gui_type = m[1];
                return;
            }
            const cm = /^([A-Za-z0-9_]+)->([a-z_]+)$/.exec(rhs.trim());
            if (cm && aliases.has(cm[1])) {
                rec.gui_type = aliases.get(cm[1])[cm[2]];
                return;
            }
            throw new Error(`gui_type unparsed: ${rhs} (${rec.key})`);
        }
        case 'nullable':
        case 'multiline':
        case 'full_width':
        case 'is_code':
        case 'readonly': {
            const trimmed = rhs.trim();
            if (trimmed === 'true' || trimmed === 'false') {
                rec[field] = trimmed === 'true';
                return;
            }
            const cm = /^([A-Za-z0-9_]+)->([a-z_]+)$/.exec(trimmed);
            if (cm && aliases.has(cm[1])) {
                rec[field] = aliases.get(cm[1])[cm[2]];
                return;
            }
            throw new Error(`bool field ${field} unparsed: ${rhs} (${rec.key})`);
        }
        case 'aliases': {
            const list = /^\{([\s\S]*)\}$/.exec(rhs.trim());
            if (list) {
                rec.aliases = splitArgs(list[1]).map((a) => decodeStringLiterals(a));
                return;
            }
            throw new Error(`aliases unparsed: ${rhs} (${rec.key})`);
        }
        case 'enum_values':
        case 'enum_labels':
        case 'shortcut': {
            const list = /^\{([\s\S]*)\}$/.exec(rhs.trim());
            if (list) {
                rec[field] = splitArgs(list[1]).map((a) => decodeStringLiterals(a));
                return;
            }
            const cm = /^([A-Za-z0-9_]+)->([a-z_]+)$/.exec(rhs.trim());
            if (cm && aliases.has(cm[1])) {
                rec[field] = aliases.get(cm[1])[cm[2]].slice();
                return;
            }
            const it = /^it_opt->second\.([a-z_]+)$/.exec(rhs.trim());
            if (it) {
                rec[field] = ctxCopySource[it[1]].slice();
                return;
            }
            throw new Error(`list field ${field} unparsed: ${rhs} (${rec.key})`);
        }
        case 'enum_keys_map': {
            let name = /ConfigOptionEnum<([A-Za-z0-9_]+)>::get_enum_values/.exec(rhs);
            if (!name) name = /s_keys_map_([A-Za-z0-9_]+)/.exec(rhs);
            if (!name) throw new Error(`enum_keys_map unparsed: ${rhs} (${rec.key})`);
            const entries = KEYS_MAPS.get(name[1]);
            if (!entries) throw new Error(`unknown keys map ${name[1]} (${rec.key})`);
            rec.enumKeysEntries = entries;
            return;
        }
        case 'cli':
        case 'cli_params':
        case 'plugin_type':
        case 'printer_technology':
            return; // deferred
        default:
            throw new Error(`unhandled field ${field} = ${rhs} (${rec.key})`);
    }
}

let ctxCopySource = null; // record being copied in filament override loop
let cur = null; // current record
let curAlias = null;
let argumentBindings = null;
let beltRemapBody = null;

// Resolve the `(void)L("...")` literal pool for boost::format substitution.
const voidLPool = [];

function handleStatement(stmt) {
    const s = stmt.trim();
    if (!s) return;

    let m;
    if (/^auto\s+add_belt_remap\s*=/.test(s)) {
        beltRemapBody = s.slice(s.indexOf('{') + 1, s.lastIndexOf('}'));
        return;
    }
    if (/^add_belt_remap\s*\(/.test(s)) {
        const [argumentsText] = callArgs(s, s.indexOf('('));
        const argumentsList = splitArgs(argumentsText);
        argumentBindings = Object.fromEntries(
            ['key', 'label', 'tooltip', 'default_axis', 'mode'].map((key, index) => [key, argumentsList[index]]),
        );
        processBlock(beltRemapBody);
        argumentBindings = null;
        return;
    }
    // alias capture: auto def_x = def = this->add(...)
    if ((m = /^auto\s+([A-Za-z0-9_]+)\s*=\s*def\s*=\s*this->add/.exec(s))) {
        curAlias = m[1];
    }
    if ((m = /this->add\s*\(/.exec(s))) {
        const [argText] = callArgs(s, s.indexOf('(', m.index));
        const a = splitArgs(argText);
        let key = decodeStringLiterals(argumentBindings?.[a[0].trim()] ?? a[0]);
        if (key === null && a[0].trim() === 'key') key = curLoopKey;
        if (key !== null && /\+\s*axis\.name/.test(a[0])) key += curAxis;
        const type = a[1].trim();
        cur = newDef(key, type, false);
        if (curAlias) {
            aliases.set(curAlias, cur);
            curAlias = null;
        }
        return;
    }
    if ((m = /this->add_nullable\s*\(/.exec(s))) {
        const [argText] = callArgs(s, s.indexOf('(', m.index));
        const a = splitArgs(argText);
        let key = decodeStringLiterals(a[0]);
        let type = a[1].trim();
        if (key === null) {
            // add_nullable(opt_key, it_opt->second.type) in the override loop
            key = curLoopKey;
            type = NULLABLE_MAP[ctxCopySource._typeCo];
        }
        cur = newDef(key, type, true);
        return;
    }
    if ((m = /^\(void\)L\s*\(/.exec(s))) {
        const [argText] = callArgs(s, s.indexOf('(', m.index));
        voidLPool.push(decodeStringLiterals(argText));
        return;
    }
    if ((m = /^def->([a-z_]+)\s*=\s*(.+)$/s.exec(s))) {
        let rhs = m[2].trim();
        // boost::format("...%1%...")% axis_upper -> substitute loop value
        const bf = /^\(boost::format\(("(?:[^"\\]|\\.)*")\)\s*%\s*axis_upper\)\.str\(\)$/.exec(rhs);
        if (bf) {
            rhs = `L(${bf[1].replace('%1%', curAxis.toUpperCase())})`;
        }
        fieldAssign(cur, m[1], rhs);
        return;
    }
    if ((m = /^def->enum_values\.(?:push_back|emplace_back)\s*\(/.exec(s))) {
        const [argText] = callArgs(s, s.indexOf('(', m.index));
        cur.enum_values.push(decodeStringLiterals(argText));
        return;
    }
    if ((m = /^def->enum_labels\.(?:push_back|emplace_back)\s*\(/.exec(s))) {
        const [argText] = callArgs(s, s.indexOf('(', m.index));
        cur.enum_labels.push(decodeStringLiterals(argText));
        return;
    }
    if ((m = /^append\s*\(def->(enum_values|enum_labels),\s*([A-Za-z0-9_]+)->/.exec(s))) {
        cur[m[1]].push(...aliases.get(m[2])[m[1]]);
        return;
    }
    if ((m = /^def->set_default_value\s*\(/.exec(s))) {
        const [argText] = callArgs(s, s.indexOf('(', m.index));
        const varm = /^new\s+(ConfigOption[A-Za-z0-9_]+)\s*[\{(]\s*axis\.(max_feedrate|max_acceleration|max_jerk)/.exec(argText.trim());
        if (varm) {
            cur.default_value = AXIS_DEFAULTS[curAxis][varm[2]].join(',');
        } else {
            const cm = /^new\s+(ConfigOption[A-Za-z0-9_]+)(?:<([A-Za-z0-9_]+)>)?\s*[\{(]([\s\S]*)[\})]$/.exec(argText.trim());
            if (cm) {
                let inner = cm[3].trim();
                // The constructor may use braces ({1, 2}) or parens ((10)).
                // cm[3] already excludes the outermost bracket; an init-list
                // written as {{..}} keeps one extra layer.
                if (inner.startsWith('{') && inner.endsWith('}')) inner = inner.slice(1, -1);
                cur.default_value = serializeDefault(cm[1], inner, cur);
                cur.default_ctor = cm[1];
                cur.default_args = inner;
            } else if (/it_opt->second\.default_value/.test(argText)) {
                cur.default_value = ctxCopySource.default_value;
            } else {
                throw new Error(`set_default_value unparsed: ${argText} (${cur.key})`);
            }
        }
        return;
    }
    if (/^def->enum_values\.push_back\(filament\.name\)/.test(s)) {
        cur.enum_values.push(...MATERIAL_NAMES);
        return;
    }
    // ignore other statements (declarations, control flow handled by the block splitter)
}

// ---------- control-flow-aware walk over init functions ----------

const AXIS_DEFAULTS = {
    x: { max_feedrate: [500, 200], max_acceleration: [1000, 1000], max_jerk: [10, 10] },
    y: { max_feedrate: [500, 200], max_acceleration: [1000, 1000], max_jerk: [10, 10] },
    z: { max_feedrate: [12, 12], max_acceleration: [500, 200], max_jerk: [0.2, 0.4] },
    e: { max_feedrate: [120, 120], max_acceleration: [5000, 5000], max_jerk: [2.5, 2.5] },
};
let curAxis = null;
let curLoopKey = null;

// FILAMENT_OVERRIDE_KEYS parsed from the const vector at top of the file.
function filamentOverrideKeys() {
    const m = /const std::vector<std::string> filament_extruder_override_keys = \{([\s\S]*?)\};/.exec(src);
    return [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]);
}
const OVERRIDE_KEYS = filamentOverrideKeys();

// FullPrintConfig defaults, the default preset overlay and headless CLI
// materialization: PrintConfig.hpp, Preset.cpp/PresetBundle.cpp:503-518,
// OrcaSlicer.cpp:1347-1350. These are definition ownership, not dump keys.
const configHeader = stripComments(readFileSync(join(REPO, 'OrcaSlicer/src/libslic3r/PrintConfig.hpp'), 'utf8'));
const fullHeader = configHeader.slice(0, configHeader.indexOf('PRINT_CONFIG_CLASS_DEFINE(\n    SLAPrinterConfig'));
const presetSource = stripComments(readFileSync(join(REPO, 'OrcaSlicer/src/libslic3r/Preset.cpp'), 'utf8'));
const staticDefaultOwners = new Set([...fullHeader.matchAll(/\(\(\s*ConfigOption[^,\n]+,\s*(\w+)\s*\)\)/g)].map((match) => match[1]));
const defaultOwners = new Set(staticDefaultOwners);
let filamentDefaults;
for (const group of ['print', 'filament', 'printer', 'machine_limits']) {
    const body = new RegExp(`s_Preset_${group}_options\\s*\\{([\\s\\S]*?)\\};`).exec(presetSource)[1];
    const keys = [...body.matchAll(/"([^"]+)"/g)].map((match) => match[1]);
    if (group === 'filament') filamentDefaults = new Set(keys);
    else for (const key of keys) defaultOwners.add(key);
}
const cliSource = stripComments(readFileSync(join(REPO, 'OrcaSlicer/src/OrcaSlicer.cpp'), 'utf8'));
const bookkeepingBody = /bookkeeping_keys\s*=\s*\{([\s\S]*?)\};/.exec(cliSource)[1];
for (const match of bookkeepingBody.matchAll(/"([^"]+)"/g)) {
    if (!staticDefaultOwners.has(match[1])) defaultOwners.delete(match[1]);
}
for (const match of cliSource.matchAll(/if\s*\(!m_extra_config\.has\("([^"]+)"\)\)\s*m_extra_config\.set_key_value\("\1", new ConfigOptionBool\(false\)\)/g)) {
    defaultOwners.add(match[1]);
}
const overrideBody = src.slice(src.indexOf('bool is_filament_extruder_override_key('), src.indexOf('size_t get_extruder_index('));
const nilOverrides = new Set([...OVERRIDE_KEYS, ...[...overrideBody.matchAll(/opt_key\s*==\s*"([^"]+)"/g)].map((match) => match[1])]);


// Extract a function body by name.
function fnBody(name) {
    const start = src.indexOf(`PrintConfigDef::${name}()`);
    const open = src.indexOf('{', start);
    let depth = 0;
    let i = open;
    for (; i < src.length; i += 1) {
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
    throw new Error(`unbalanced body ${name}`);
}

// Split a block into top-level statements / nested blocks.
function* iterStatements(block) {
    let i = 0;
    let cur = '';
    while (i < block.length) {
        const c = block[i];
        if (c === '"') {
            const start = i;
            i += 1;
            while (i < block.length && block[i] !== '"') {
                if (block[i] === '\\') i += 1;
                i += 1;
            }
            cur += block.slice(start, i + 1);
            i += 1;
            continue;
        }
        if (c === '{') {
            // `{` opens a code block only in statement-leading position:
            // empty head (bare block) or right after for/if/else/while head.
            const head = cur.trim();
            const isBlock = head === '' || /(for|if|else|while|switch)\s*\([\s\S]*\)\s*$/.test(head) || /\belse\s*$/.test(head);
            let depth = 1;
            const start = i;
            i += 1;
            while (i < block.length && depth > 0) {
                if (block[i] === '"') {
                    i += 1;
                    while (i < block.length && block[i] !== '"') {
                        if (block[i] === '\\') i += 1;
                        i += 1;
                    }
                    i += 1; // step past the closing quote
                    continue;
                }
                if (block[i] === '{') depth += 1;
                if (block[i] === '}') depth -= 1;
                i += 1;
            }
            if (isBlock) {
                yield { type: 'block', head, body: block.slice(start + 1, i - 1) };
                cur = '';
            } else {
                // brace-initializer inside an expression: keep it inline
                cur += block.slice(start, i);
            }
            continue;
        }
        if (c === ';') {
            yield { type: 'stmt', text: cur + ';' };
            cur = '';
            i += 1;
            continue;
        }
        cur += c;
        i += 1;
    }
    if (cur.trim()) yield { type: 'stmt', text: cur };
}

function processBlock(block) {
    for (const item of iterStatements(block)) {
        if (item.type === 'stmt') {
            // an `if (...) X else Y` may arrive as stmt without braces
            const ifm = /^if\s*\(([^)]*)\)\s*([\s\S]*?)(?:\belse\b([\s\S]*))?$/.exec(item.text.trim());
            if (ifm && !ifm[0].startsWith('if (opt_key')) {
                // generic: eval both branches? Only override-loop `if` matters; handled below.
            }
            if (/^if\s*\(/.test(item.text.trim()) && /opt_key/.test(item.text)) {
                // handled inside override loop processing
            }
            handleStatement(item.text.replace(/;\s*$/, ''));
        } else {
            const head = item.head;
            if (/^for\s*\(const auto& filament : MaterialType::all\(\)\)/.test(head)) {
                for (const s of iterStatements(item.body)) {
                    if (s.type === 'stmt' && /enum_values\.push_back\(filament\.name\)/.test(s.text)) {
                        cur.enum_values.push(...MATERIAL_NAMES);
                    } else if (s.type === 'stmt') {
                        handleStatement(s.text.replace(/;\s*$/, ''));
                    }
                }
            } else if (/^for\s*\(const AxisDefault &axis : axes\)/.test(head)) {
                for (const axis of Object.keys(AXIS_DEFAULTS)) {
                    curAxis = axis;
                    voidLPool.length = 0;
                    processBlock(item.body);
                }
                curAxis = null;
            } else if (/^for\s*\(const char\* key :/.test(head)) {
                for (const match of head.matchAll(/"([^"]+)"/g)) {
                    curLoopKey = match[1];
                    processBlock(item.body);
                }
                curLoopKey = null;
            } else if (/^for\s*\(auto& opt_key : filament_extruder_override_keys\)/.test(head)) {
                for (const key of OVERRIDE_KEYS) {
                    curLoopKey = key;
                    const srcKey = key.replace(/^filament_/, '');
                    ctxCopySource = defs.get(srcKey);
                    if (!ctxCopySource) throw new Error(`override source ${srcKey} missing`);
                    const simple = [
                        'filament_retraction_length',
                        'filament_z_hop',
                        'filament_long_retractions_when_cut',
                        'filament_retraction_distances_when_cut',
                    ].includes(key);
                    const rec = newDef(key, ctxCopySource._typeCo + 'Nullable', true);
                    for (const f of ['label', 'full_label', 'tooltip', 'sidetext', 'enum_labels', 'enum_values']) {
                        rec[f] = Array.isArray(ctxCopySource[f]) ? ctxCopySource[f].slice() : ctxCopySource[f];
                    }
                    rec.enumKeysEntries = ctxCopySource.enumKeysEntries;
                    rec.enum_keys = ctxCopySource.enum_keys.slice();
                    rec.min = ctxCopySource.min;
                    rec.max = ctxCopySource.max;
                    rec.mode = simple ? 'simple' : 'advanced';
                    rec.default_value = ctxCopySource.default_value;
                }
                curLoopKey = null;
                ctxCopySource = null;
            } else if (/^if\s*\(/.test(head) || /^\}/.test(head)) {
                processBlock(item.body);
            } else {
                // bare block or unknown loop: recurse
                processBlock(item.body);
            }
        }
    }
}

processBlock(fnBody('init_common_params'));
// assign_printer_technology_to_unknown(common, ptAny)
for (const rec of defs.values()) {
    if (rec.printer_technology === 'unknown') rec.printer_technology = 'any';
}
processBlock(fnBody('init_fff_params'));
for (const rec of defs.values()) {
    if (rec.printer_technology === 'unknown') rec.printer_technology = 'fff';
}

// Key groups.
function keyList(body, varName) {
    const m = new RegExp(`${varName}\\s*=\\s*\\{([\\s\\S]*?)\\};`).exec(body);
    return [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]);
}
groups.extruder_option_keys = keyList(fnBody('init_extruder_option_keys'), 'm_extruder_option_keys');
groups.extruder_retract_keys = keyList(fnBody('init_extruder_option_keys'), 'm_extruder_retract_keys');
groups.filament_option_keys = keyList(fnBody('init_filament_option_keys'), 'm_filament_option_keys');
groups.filament_retract_keys = keyList(fnBody('init_filament_option_keys'), 'm_filament_retract_keys');

// ---------- emit JSON ----------

function defaultWire(rec) {
    const escape = (value) => value.replace(/[\\"\r\n]/g, (character) => ({'\\': '\\\\', '"': '\\"', '\r': '\\r', '\n': '\\n'})[character]);
    if (rec.type === 'Bool') return rec.default_value === 'true' ? '1' : rec.default_value === 'false' ? '0' : rec.default_value;
    if (rec.type === 'String') return escape(rec.default_value ?? '');
    if (rec.type === 'Strings') {
        const values = splitArgs(rec.default_args ?? '').map((value) => decodeStringLiterals(value));
        return values.map((value) => ((values.length === 1 && value === '') || /[\s\\"]/.test(value)) ? `"${escape(value)}"` : value).join(';');
    }
    return rec.default_value ?? '0';
}

function toJson(rec) {
    const wire = defaultWire(rec);
    const numericVector = /^(Floats|Ints|Bools|Percents)/.test(rec.type);
    return {
        key: rec.key,
        type: rec.type,
        default_value: rec.default_value,
        default_wire_value: wire,
        full_config_default: defaultOwners.has(rec.key),
        default_is_nil: (rec.nullable && !staticDefaultOwners.has(rec.key) && filamentDefaults.has(rec.key) && nilOverrides.has(rec.key)) ||
            (numericVector && (wire === '' || (rec.nullable && wire.split(',').every((value) => value === 'nil')))),
        nullable: rec.nullable,
        label: rec.label,
        full_label: rec.full_label,
        category: rec.category,
        tooltip: rec.tooltip,
        sidetext: rec.sidetext,
        enum_values: rec.enum_values,
        enum_labels: rec.enum_labels,
        enum_keys: rec.enum_keys,
        min: rec.min,
        max: rec.max,
        mode: rec.mode,
        gui_type: rec.gui_type,
        gui_flags: rec.gui_flags,
        multiline: rec.multiline,
        full_width: rec.full_width,
        is_code: rec.is_code,
        readonly: rec.readonly,
        height: rec.height,
        width: rec.width,
        ratio_over: rec.ratio_over,
        max_literal: rec.max_literal,
        aliases: rec.aliases,
        shortcut: rec.shortcut,
        printer_technology: rec.printer_technology,
    };
}

// Align enum_keys with enum_values: for each pushed enum value string, the
// upstream enum_keys_map entry with the same key supplies the int code.
// Values without a keys-map entry (e.g. filament_map_mode's "Default") emit -1.
for (const rec of defs.values()) {
    if (rec.enumKeysEntries && rec.enum_values.length) {
        rec.enum_keys = rec.enum_values.map((v) => {
            const hit = rec.enumKeysEntries.find((e) => e.key === v);
            return hit ? hit.value : -1;
        });
    }
}

const jsonDefs = order.map((k) => toJson(defs.get(k)));

if (DUMP) {
    console.log(JSON.stringify({ options: jsonDefs, groups }, null, 2));
    process.exit(0);
}

// ---------- emit Rust table ----------

function rsStr(s) {
    if (s === null) return 'None';
    return `Some(${JSON.stringify(s)})`;
}
function rsStrList(a) {
    if (!a || a.length === 0) return '&[]';
    return `&[${a.map((x) => JSON.stringify(x)).join(', ')}]`;
}
function rsNum(v) {
    if (v === null) return 'None';
    // f64 fields always need a float literal.
    return Number.isInteger(v) ? `Some(${v}.0)` : `Some(${v})`;
}
function rsIntList(a) {
    if (!a || a.length === 0) return '&[]';
    return `&[${a.join(', ')}]`;
}

// Emit sorted by key so option_metadata() can binary-search.
const sortedDefs = [...jsonDefs].sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));

const MODE_RS = { simple: 'Simple', advanced: 'Advanced', expert: 'Expert', develop: 'Develop' };
const GUI_RS = {
    undefined: 'Undefined',
    i_enum_open: 'IEnumOpen',
    f_enum_open: 'FEnumOpen',
    color: 'Color',
    select_open: 'SelectOpen',
    slider: 'Slider',
    legend: 'Legend',
    plugin_config: 'PluginConfig',
    plugin_picker: 'PluginPicker',
    one_string: 'OneString',
};

function rsEnum(qual, v) {
    return `${qual}::${v}`;
}

let modRs = '// Generated by scripts/gen-configdef.mjs from OrcaSlicer/src/libslic3r/PrintConfig.cpp. Do not edit.\n';
modRs += '// Re-run `node scripts/gen-configdef.mjs` after upstream PrintConfig.cpp changes.\n\n';
modRs += 'use super::super::{OptionGuiType, OptionMetadata, OptionMode};\n\n';
modRs += 'pub(crate) const METADATA: &[OptionMetadata] = &[\n';
for (const d of sortedDefs) {
    modRs += '    OptionMetadata {\n';
    modRs += `        key: ${JSON.stringify(d.key)},\n`;
    modRs += `        label: ${rsStr(d.label)},\n`;
    modRs += `        full_label: ${rsStr(d.full_label)},\n`;
    modRs += `        category: ${rsStr(d.category)},\n`;
    modRs += `        tooltip: ${rsStr(d.tooltip)},\n`;
    modRs += `        sidetext: ${rsStr(d.sidetext)},\n`;
    modRs += `        enum_values: ${rsStrList(d.enum_values)},\n`;
    modRs += `        enum_labels: ${rsStrList(d.enum_labels)},\n`;
    modRs += `        enum_keys: ${rsIntList(d.enum_keys)},\n`;
    modRs += `        min: ${rsNum(d.min)},\n`;
    modRs += `        max: ${rsNum(d.max)},\n`;
    modRs += `        max_literal: ${rsNum(d.max_literal)},\n`;
    modRs += `        mode: OptionMode::${MODE_RS[d.mode]},\n`;
    modRs += `        gui_type: OptionGuiType::${GUI_RS[d.gui_type]},\n`;
    modRs += `        gui_flags: ${JSON.stringify(d.gui_flags)},\n`;
    modRs += `        multiline: ${d.multiline},\n`;
    modRs += `        full_width: ${d.full_width},\n`;
    modRs += `        is_code: ${d.is_code},\n`;
    modRs += `        readonly: ${d.readonly},\n`;
    modRs += `        nullable: ${d.nullable},\n`;
    modRs += `        height: ${rsNum(d.height)},\n`;
    modRs += `        width: ${rsNum(d.width)},\n`;
    modRs += `        ratio_over: ${rsStr(d.ratio_over)},\n`;
    modRs += `        aliases: ${rsStrList(d.aliases)},\n`;
    modRs += `        shortcut: ${rsStrList(d.shortcut)},\n`;
    modRs += `        printer_technology: ${JSON.stringify(d.printer_technology)},\n`;
    modRs += '    },\n';
}
modRs += '];\n';

// Normalize through rustfmt so `--check` compares against the committed
// (formatted) file rather than raw generator output.
function rustfmt(code) {
    return execFileSync('rustfmt', ['--edition', '2024'], { input: code }).toString();
}

const formatted = rustfmt(modRs);

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
