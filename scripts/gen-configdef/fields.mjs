import { decodeStringLiterals, splitArgs } from './cpp.mjs';

export function createFieldAssigner(KEYS_MAPS, aliases, state, fmtNum) {
    const MODE_MAP = {
        comSimple: 'simple',
        comAdvanced: 'advanced',
        comExpert: 'expert',
        comDevelop: 'develop',
    };

    function fieldAssign(rec, field, rhs) {
        rhs = state.argumentBindings?.[rhs.trim()] ?? rhs;
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
                    rec[field] = state.ctxCopySource[it[1]];
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
                    rec[field] = state.ctxCopySource[it[1]];
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
                    rec[field] = state.ctxCopySource[it[1]].slice();
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

    return fieldAssign;
}
