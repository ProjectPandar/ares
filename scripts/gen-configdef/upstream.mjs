import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

import { stripComments } from './cpp.mjs';

export function readUpstream(REPO) {
    const UPSTREAM = join(REPO, 'OrcaSlicer/src/libslic3r/PrintConfig.cpp');
    const MATERIALS = join(REPO, 'OrcaSlicer/src/libslic3r/MaterialType.cpp');
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

    const ENUM_VALUES = collectEnumValues();
    const KEYS_MAPS = keysMaps(ENUM_VALUES);
    const MATERIAL_NAMES = materialNames();

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


    return { src, constNumbers, ENUM_VALUES, KEYS_MAPS, MATERIAL_NAMES, OVERRIDE_KEYS,
        defaultOwners, staticDefaultOwners, filamentDefaults, nilOverrides };
}
