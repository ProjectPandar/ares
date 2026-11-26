import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

const KIND_JSON = {
    Float: 'float',
    FloatOrPercent: 'float_or_percent',
    FloatsOrPercents: 'floats_or_percents',
    Percent: 'percent',
    Percents: 'percents',
    PercentsNullable: 'percents_nullable',
    Int: 'int',
    Bool: 'bool',
    Bools: 'bools',
    BoolsNullable: 'bools_nullable',
    Enum: 'enum',
    Enums: 'enums',
    EnumsNullable: 'enums_nullable',
    Floats: 'floats',
    FloatsNullable: 'floats_nullable',
    IntsNullable: 'ints_nullable',
    Ints: 'ints',
    Strings: 'strings',
    String: 'string',
    Point: 'point',
    Points: 'points',
    PointsGroups: 'points_groups',
};

function readOptionDefinitions(repo) {
    const tableDir = join(repo, 'crates/ares-core/src/options/registry/definitions/table');
    const definitions = new Map();
    for (const file of readdirSync(tableDir).filter((name) => name.endsWith('.rs')).sort()) {
        const text = readFileSync(join(tableDir, file), 'utf8');
        for (const match of text.matchAll(/definition!\(\s*"([^"]+)",\s*(\w+),\s*("(?:[^"\\]|\\.)*")/g)) {
            const kind = KIND_JSON[match[2]];
            if (!kind) throw new Error(`unknown OptionValueKind ${match[2]} for ${match[1]}`);
            definitions.set(match[1], {
                kind,
                default_value: JSON.parse(match[3]),
            });
        }
    }
    return definitions;
}

function readKeyLists(repo) {
    const text = readFileSync(join(repo, 'crates/ares-core/src/options/registry/key_lists.rs'), 'utf8');
    const readList = (constName) => {
        const match = new RegExp(`const ${constName}: &\\[&str\\] = &\\[([\\s\\S]*?)\\];`).exec(text);
        if (!match) throw new Error(`missing ${constName} in key_lists.rs`);
        return [...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
    };
    return {
        extruder_option_keys: readList('EXTRUDER_OPTION_KEYS'),
        filament_option_keys: readList('FILAMENT_OPTION_KEYS'),
        extruder_retract_keys: readList('EXTRUDER_RETRACT_KEYS'),
        filament_retract_keys: readList('FILAMENT_RETRACT_KEYS'),
    };
}

function jsonF64(value) {
    if (value === null) return 'null';
    const text = String(value);
    return Number.isInteger(value) && !text.includes('e') && !text.includes('E')
        ? `${text}.0`
        : text;
}

function renderEntry(def, definitions) {
    const owned = definitions.get(def.key);
    const valueKind = owned ? owned.kind : 'unknown';
    const defaultValue = owned ? owned.default_value : '';
    return `{${[
        `"key":${JSON.stringify(def.key)}`,
        `"type":${JSON.stringify(valueKind)}`,
        `"default_value":${JSON.stringify(defaultValue)}`,
        `"nullable":${def.nullable}`,
        `"label":${JSON.stringify(def.label)}`,
        `"full_label":${JSON.stringify(def.full_label)}`,
        `"category":${JSON.stringify(def.category)}`,
        `"tooltip":${JSON.stringify(def.tooltip)}`,
        `"sidetext":${JSON.stringify(def.sidetext)}`,
        `"enum_values":${JSON.stringify(def.enum_values)}`,
        `"enum_labels":${JSON.stringify(def.enum_labels)}`,
        `"enum_keys":${JSON.stringify(def.enum_keys)}`,
        `"min":${jsonF64(def.min)}`,
        `"max":${jsonF64(def.max)}`,
        `"mode":${JSON.stringify(def.mode)}`,
        `"gui_type":${JSON.stringify(def.gui_type)}`,
        `"gui_flags":${JSON.stringify(def.gui_flags)}`,
        `"multiline":${def.multiline}`,
        `"full_width":${def.full_width}`,
        `"is_code":${def.is_code}`,
        `"readonly":${def.readonly}`,
        `"height":${jsonF64(def.height)}`,
        `"width":${jsonF64(def.width)}`,
        `"ratio_over":${JSON.stringify(def.ratio_over)}`,
        `"max_literal":${jsonF64(def.max_literal)}`,
        `"aliases":${JSON.stringify(def.aliases)}`,
        `"shortcut":${JSON.stringify(def.shortcut)}`,
        `"printer_technology":${JSON.stringify(def.printer_technology)}`,
    ].join(',')}}`;
}

// Builds the exact UTF-8 JSON bytes served by `ares_core::print_config_def_json()`.
export function renderPrintConfigDefJson(jsonDefs, repo) {
    const definitions = readOptionDefinitions(repo);
    const groups = readKeyLists(repo);
    const sortedDefs = [...jsonDefs].sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
    const json = `{"options":[${sortedDefs.map((d) => renderEntry(d, definitions)).join(',')}],"groups":${JSON.stringify(groups)}}`;
    return Buffer.from(json, 'utf8');
}
