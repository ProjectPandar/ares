// Shared-free upstream PrintConfig.cpp/MaterialType.cpp source scanner and loop expander.

import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

export function stripComments(src) {
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

export function decodeStr(text) {
    const ctx = /^\s*L_CONTEXT\s*\(\s*([\s\S]*)\)\s*$/.exec(text);
    if (ctx) {
        let depth = 0;
        let inStr = false;
        const inner = ctx[1];
        for (let i = 0; i < inner.length; i += 1) {
            const c = inner[i];
            if (inStr) {
                if (c === '\\') i += 1;
                else if (c === '"') inStr = false;
            } else if (c === '"') inStr = true;
            else if (c === '(') depth += 1;
            else if (c === ')') depth -= 1;
            else if (c === ',' && depth === 0) {
                text = inner.slice(0, i);
                break;
            }
        }
    }
    const re = /(?:u8)?"((?:[^"\\]|\\.)*)"/g;
    let m;
    let out = '';
    let found = false;
    while ((m = re.exec(text)) !== null) {
        found = true;
        out += m[1].replace(/\\(u[0-9a-fA-F]{4}|x[0-9a-fA-F]{1,4}|n|r|t|0|\\|"|')/g, (_, e) => {
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

// enum constant -> int, from all libslic3r enum declarations + s_keys_map.
function enumTables(repo) {
    const vals = new Map();
    for (const file of readdirSync(join(repo, 'OrcaSlicer/src/libslic3r')).filter((f) => f.endsWith('.hpp'))) {
        const h = stripComments(readFileSync(join(repo, 'OrcaSlicer/src/libslic3r', file), 'utf8'));
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

export function fnBody(src, name) {
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

// Expand loop-generated defs: replace the for-loop bodies with one synthetic region per generated key.
function expandRegions(body, materialNames, overrideKeys) {
    const axes = {
        x: { feed: '500,200', accel: '1000,1000', jerk: '10,10' },
        y: { feed: '500,200', accel: '1000,1000', jerk: '10,10' },
        z: { feed: '12,12', accel: '500,200', jerk: '0.2,0.4' },
        e: { feed: '120,120', accel: '5000,5000', jerk: '2.5,2.5' },
    };
    const out = [];
    const axisLoop = /for \(const AxisDefault &axis : axes\) \{([\s\S]*?)\n        \}/.exec(body);
    if (axisLoop) {
        const parts = axisLoop[1].split(/(?=def\s*=\s*this->add\()/).filter((p) => p.includes('this->add('));
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
    const pluginLoop = /for\s*\(\s*const char\*\s*key\s*:\s*\{([^}]*)\}\s*\)\s*\{([\s\S]*?)\n    \}/.exec(body);
    if (pluginLoop) {
        const keys = [...pluginLoop[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
        for (const key of keys) {
            out.push(pluginLoop[2].replace(/this->add\(\s*key\s*,/, `this->add("${key}",`));
        }
        body = body.replace(pluginLoop[0], '');
    }
    const beltLambda = /auto\s+add_belt_remap\s*=\s*\[this\]\([^)]*\)\s*\{([\s\S]*?)\n    \};/.exec(body);
    if (beltLambda) {
        const tmpl = beltLambda[1];
        body = body.replace(beltLambda[0], '');
        const callRe = /add_belt_remap\(\s*"([^"]+)"\s*,\s*L\(("(?:[^"\\]|\\.)*")\)\s*,\s*L\(("(?:[^"\\]|\\.)*")\)\s*,\s*([^,]+)\s*,\s*(com\w+)\s*\);/g;
        let cm;
        while ((cm = callRe.exec(body)) !== null) {
            const expanded = tmpl
                .replace(/auto\s+def\s*=\s*this->add\(\s*key\s*,/, `def = this->add("${cm[1]}",`)
                .replace(/def->label\s*=\s*label;/, `def->label = L(${cm[2]});`)
                .replace(/def->tooltip\s*=\s*tooltip;/, `def->tooltip = L(${cm[3]});`)
                .replace(/def->mode\s*=\s*mode;/, `def->mode = ${cm[5]};`);
            out.push(expanded);
        }
        body = body.replace(callRe, '');
    }
    body = body.replace(
        /for \(const auto& filament : MaterialType::all\(\)\) \{\s*def->enum_values\.push_back\(filament\.name\);\s*\}/g,
        materialNames.map((n) => `def->enum_values.push_back("${n}");`).join('\n'));
    const ovLoop = /for \(auto& opt_key : filament_extruder_override_keys\) \{([\s\S]*?)\n    \}/.exec(body);
    if (ovLoop) {
        body = body.replace(ovLoop[0], '');
        for (const key of overrideKeys) out.push(`__OVERRIDE__:${key}`);
    }
    return { body, extra: out };
}

export function loadUpstreamRegions(repo) {
    const src = stripComments(readFileSync(join(repo, 'OrcaSlicer/src/libslic3r/PrintConfig.cpp'), 'utf8'));
    const enumVals = enumTables(repo);
    const keysMaps = new Map();
    for (const m of src.matchAll(/s_keys_map_(\w+)\s*=?\s*\{([\s\S]*?)\};/g)) {
        const entries = [];
        for (const e of m[2].matchAll(/\{\s*"([^"]+)"\s*,\s*([^}]+?)\s*\}/g)) {
            let tok = e[2].trim();
            const cast = /^\(\s*[\w:]+\s*\)\s*(.+)$/.exec(tok);
            if (cast) tok = cast[1].trim();
            const im = /^int\s*\(\s*([\w:]+)\s*\)$/.exec(tok);
            if (im) tok = im[1];
            tok = tok.replace(/\s+/g, '');
            const v = enumVals.get(tok);
            if (v === undefined) throw new Error(`keys-map const ${tok} unresolved`);
            entries.push([e[1], v]);
        }
        keysMaps.set(m[1], entries);
    }
    const msrc = readFileSync(join(repo, 'OrcaSlicer/src/libslic3r/MaterialType.cpp'), 'utf8');
    const sec = msrc.slice(msrc.indexOf('material_types = {'), msrc.indexOf('};', msrc.indexOf('material_types = {')));
    const materialNames = [...sec.matchAll(/\{\s*"([^"]+)"/g)].map((x) => x[1]);
    const overrideMatch = /filament_extruder_override_keys = \{([\s\S]*?)\};/.exec(src);
    const overrideKeys = [...overrideMatch[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]);

    const bodyCommon = fnBody(src, 'init_common_params');
    const bodyFff = fnBody(src, 'init_fff_params');
    const expCommon = expandRegions(bodyCommon, materialNames, overrideKeys);
    const expFff = expandRegions(bodyFff, materialNames, overrideKeys);
    const allText = `${expCommon.body}\n${expFff.body}\n${expCommon.extra.join('\n')}\n${expFff.extra.join('\n')}`;

    const regions = [];
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
    return { src, bodyCommon, bodyFff, keysMaps, overrideKeys, regions };
}
