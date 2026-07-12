import { deriveExportRules, deriveMetadata } from "./source.mjs";

export function verifySourceMutations(sources, definitions, keys) {
  const replaceOnce = (source, needle, replacement) => {
    const first = source.indexOf(needle);
    if (first < 0 || first !== source.lastIndexOf(needle)) {
      throw new Error(`mutation anchor is not unique: ${needle}`);
    }
    return source.slice(0, first) + replacement + source.slice(first + needle.length);
  };
  const expectExportRejection = (needle, replacement) => {
    const mutated = replaceOnce(sources.gcode, needle, replacement);
    try {
      deriveExportRules(mutated, keys);
    } catch {
      return;
    }
    throw new Error(`export mutation was accepted: ${needle}`);
  };
  expectExportRejection(
    'std::vector<double> temp_flush_volumes_matrix = cfg.option<ConfigOptionFloats>("flush_volumes_matrix")->values',
    'std::vector<double> temp_flush_volumes_matrix = cfg.option<ConfigOptionFloats>("flush_multiplier")->values',
  );
  expectExportRejection('cfg.option(key))->get_at', 'cfg.option("flush_multiplier"))->get_at');
  expectExportRejection('<< "; " << key << " = " << dynamic_cast', '<< "; wrong = " << dynamic_cast');
  expectExportRejection('cfg.opt_serialize(key)', 'cfg.opt_serialize("flush_multiplier")');
  expectExportRejection(
    'if (!is_banned(key) && !cfg.option(key)->is_nil())',
    'if (!is_banned(key))',
  );
  expectExportRejection(
    'if (!is_banned(key) && !cfg.option(key)->is_nil())',
    'if (!cfg.option(key)->is_nil())',
  );
  expectExportRejection(
    'if(key == "extruder_colour")',
    'else if(key == "extruder_colour")',
  );
  expectExportRejection(
    'if (key == "wipe_tower_x" || key == "wipe_tower_y")',
    'ss << key; if (key == "wipe_tower_x" || key == "wipe_tower_y")',
  );
  expectExportRejection(
    '"compatible_printers"sv,',
    '"adaptive_bed_mesh_margin"sv,\n        "compatible_printers"sv,',
  );
  expectExportRejection(
    'ss << "; " << key << " = " << cfg.opt_serialize(key) << "\\n";',
    'ss << "; " << key << " = " << cfg.opt_serialize(key) << "\\n"; if (key == "adaptive_bed_mesh") ss << key;',
  );
  const metadataMutation = replaceOnce(
    sources.config,
    'j[BBL_JSON_KEY_FROM] = from;',
    'j[BBL_JSON_KEY_NAME] = from;',
  );
  try {
    deriveMetadata(new Set(definitions.keys()), keys, metadataMutation, sources.presetHeader);
  } catch {
    return;
  }
  throw new Error("metadata mutation was accepted");
}
