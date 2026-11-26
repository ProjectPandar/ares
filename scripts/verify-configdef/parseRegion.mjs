// Per-region ConfigOptionDef field extraction and alias/override resolution.

import { decodeStr } from './upstream.mjs';

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
    color: 'color', select_open: 'select_open', slider: 'slider', legend: 'legend',
    plugin_config: 'plugin_config', plugin_picker: 'plugin_picker', one_string: 'one_string',
};
const OVERRIDE_SIMPLE = new Set([
    'filament_retraction_length', 'filament_z_hop',
    'filament_long_retractions_when_cut', 'filament_retraction_distances_when_cut',
]);
export const OVERRIDE_KIND = {
    floats: 'floats_nullable',
    percents: 'percents_nullable', ints: 'ints_nullable', bools: 'bools_nullable',
    enums: 'enums_nullable',
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

export function extractUpstreamDefs({ regions, keysMaps, overrideKeys }) {
    const aliasKey = new Map();
    for (const r of regions) if (r.alias) aliasKey.set(r.alias, r.key);

    const parsed = new Map();
    const orderKeys = [];
    function getField(key, field) {
        const r = parsed.get(key);
        if (!r) throw new Error(`alias target ${key} not parsed yet`);
        return r[field];
    }

    function parseRegion(region) {
        const t = region.text;
        const rec = { key: region.key, type: CO_TO_KIND[region.co], nullable: region.nullable };
        // Capture `def->name = <rhs>` where <rhs> runs to the first `;` not inside a string literal.
        const rhsText = (name) => {
            const re = new RegExp(`def->${name}\\s*=`, 'g');
            let m;
            let last = null;
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
            const m = new RegExp(`def->${name}\\s*=\\s*(true|false);`).exec(t);
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
            rec[f] = v !== null ? v : false;
        }
        {
            const m = /def->nullable\s*=\s*(true|false)/.exec(t);
            if (m) rec.nullable = m[1] === 'true';
            else if (region.co === 'coFloatsOrPercents' && /new\s+ConfigOptionFloatsOrPercentsNullable\b/.test(t)) {
                rec.nullable = true;
            }
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
                const entries = keysMaps.get(m[1] || m[2]);
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

    for (const region of regions) {
        const rec = parseRegion(region);
        if (!parsed.has(rec.key)) orderKeys.push(rec.key);
        parsed.set(rec.key, rec); // upstream add() overwrites by key
    }
    // filament_* nullable overrides copy fields from their unprefixed source.
    for (const key of overrideKeys) {
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
    return { parsed, orderKeys };
}
