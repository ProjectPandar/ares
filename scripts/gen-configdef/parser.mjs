import { callArgs, decodeStringLiterals, fnBody, iterStatements, splitArgs } from './cpp.mjs';
import { createDefaultSerializer } from './defaults.mjs';
import { createFieldAssigner } from './fields.mjs';

export function parseDefinitions(upstream) {
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

    const { src, KEYS_MAPS, MATERIAL_NAMES, OVERRIDE_KEYS } = upstream;
    const state = { ctxCopySource: null, argumentBindings: null };
    const { fmtNum, serializeDefault } = createDefaultSerializer(upstream, state);
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

    const fieldAssign = createFieldAssigner(KEYS_MAPS, aliases, state, fmtNum);

    let cur = null; // current record
    let curAlias = null;
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
            state.argumentBindings = Object.fromEntries(
                ['key', 'label', 'tooltip', 'default_axis', 'mode'].map((key, index) => [key, argumentsList[index]]),
            );
            processBlock(beltRemapBody);
            state.argumentBindings = null;
            return;
        }
        // alias capture: auto def_x = def = this->add(...)
        if ((m = /^auto\s+([A-Za-z0-9_]+)\s*=\s*def\s*=\s*this->add/.exec(s))) {
            curAlias = m[1];
        }
        if ((m = /this->add\s*\(/.exec(s))) {
            const [argText] = callArgs(s, s.indexOf('(', m.index));
            const a = splitArgs(argText);
            let key = decodeStringLiterals(state.argumentBindings?.[a[0].trim()] ?? a[0]);
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
                type = NULLABLE_MAP[state.ctxCopySource._typeCo];
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
                    cur.default_value = state.ctxCopySource.default_value;
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

    const AXIS_DEFAULTS = {
        x: { max_feedrate: [500, 200], max_acceleration: [1000, 1000], max_jerk: [10, 10] },
        y: { max_feedrate: [500, 200], max_acceleration: [1000, 1000], max_jerk: [10, 10] },
        z: { max_feedrate: [12, 12], max_acceleration: [500, 200], max_jerk: [0.2, 0.4] },
        e: { max_feedrate: [120, 120], max_acceleration: [5000, 5000], max_jerk: [2.5, 2.5] },
    };
    let curAxis = null;
    let curLoopKey = null;

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
                        state.ctxCopySource = defs.get(srcKey);
                        if (!state.ctxCopySource) throw new Error(`override source ${srcKey} missing`);
                        const simple = [
                            'filament_retraction_length',
                            'filament_z_hop',
                            'filament_long_retractions_when_cut',
                            'filament_retraction_distances_when_cut',
                        ].includes(key);
                        const rec = newDef(key, state.ctxCopySource._typeCo + 'Nullable', true);
                        for (const f of ['label', 'full_label', 'tooltip', 'sidetext', 'enum_labels', 'enum_values']) {
                            rec[f] = Array.isArray(state.ctxCopySource[f]) ? state.ctxCopySource[f].slice() : state.ctxCopySource[f];
                        }
                        rec.enumKeysEntries = state.ctxCopySource.enumKeysEntries;
                        rec.enum_keys = state.ctxCopySource.enum_keys.slice();
                        rec.min = state.ctxCopySource.min;
                        rec.max = state.ctxCopySource.max;
                        rec.mode = simple ? 'simple' : 'advanced';
                        rec.default_value = state.ctxCopySource.default_value;
                    }
                    curLoopKey = null;
                    state.ctxCopySource = null;
                } else if (/^if\s*\(/.test(head) || /^\}/.test(head)) {
                    processBlock(item.body);
                } else {
                    // bare block or unknown loop: recurse
                    processBlock(item.body);
                }
            }
        }
    }

    processBlock(fnBody(src, 'init_common_params'));
    // assign_printer_technology_to_unknown(common, ptAny)
    for (const rec of defs.values()) {
        if (rec.printer_technology === 'unknown') rec.printer_technology = 'any';
    }
    processBlock(fnBody(src, 'init_fff_params'));
    for (const rec of defs.values()) {
        if (rec.printer_technology === 'unknown') rec.printer_technology = 'fff';
    }

    // Key groups.
    function keyList(body, varName) {
        const m = new RegExp(`${varName}\\s*=\\s*\\{([\\s\\S]*?)\\};`).exec(body);
        return [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]);
    }
    groups.extruder_option_keys = keyList(fnBody(src, 'init_extruder_option_keys'), 'm_extruder_option_keys');
    groups.extruder_retract_keys = keyList(fnBody(src, 'init_extruder_option_keys'), 'm_extruder_retract_keys');
    groups.filament_option_keys = keyList(fnBody(src, 'init_filament_option_keys'), 'm_filament_option_keys');
    groups.filament_retract_keys = keyList(fnBody(src, 'init_filament_option_keys'), 'm_filament_retract_keys');

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

    return { defs, order, groups };
}
