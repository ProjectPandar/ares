import { decodeStringLiterals, splitArgs } from './cpp.mjs';

export function createDefaultSerializer({ constNumbers, ENUM_VALUES }, state) {
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
        tok = state.argumentBindings?.[tok.trim()] ?? tok;
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

    return { fmtNum, serializeDefault };
}
