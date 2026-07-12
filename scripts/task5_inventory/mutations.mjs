import { deriveExportRules, deriveMetadata } from "./source.mjs";
import { axisDefinitions } from "./axis.mjs";
import { enumLookup, evaluateEnumDefault } from "./enums.mjs";

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

export function verifyOptionDefinitionMutations(source, numberList) {
  const printOrderMutation = replaceRequired(
    source,
    '"default",     int(PrintOrder::Default)',
    '"WRONG",       int(PrintOrder::Default)',
  );
  const inputExpression = "ConfigOptionEnum<InputShaperType>(InputShaperType::Default)";
  if (evaluateEnumDefault(inputExpression, enumLookup(printOrderMutation)) !== "Default") {
    throw new Error("unrelated enum mutation changed InputShaperType");
  }
  const inputShaperMutation = replaceRequired(
    source,
    '{"Default", int(InputShaperType::Default)}',
    '{"WRONG", int(InputShaperType::Default)}',
  );
  if (evaluateEnumDefault(inputExpression, enumLookup(inputShaperMutation)) !== "WRONG") {
    throw new Error("InputShaperType mutation was accepted");
  }
  const nozzleExpression = "ConfigOptionEnumsGenericNullable({ ntUndefine })";
  if (evaluateEnumDefault(nozzleExpression, enumLookup(source)) !== "undefine") {
    throw new Error("NozzleType undefine default was not resolved");
  }
  const nozzleMutation = replaceRequired(
    source,
    '{ "undefine",       int(NozzleType::ntUndefine) }',
    '{ "WRONG",          int(NozzleType::ntUndefine) }',
  );
  if (evaluateEnumDefault(nozzleExpression, enumLookup(nozzleMutation)) !== "WRONG") {
    throw new Error("NozzleType mutation was accepted");
  }
  for (const [needle, replacement] of [
    ['this->add("machine_max_speed_" + axis.name, coFloats)', 'this->add("wrong_speed_" + axis.name, coFloats)'],
    ["ConfigOptionFloats(axis.max_acceleration)", "ConfigOptionFloats(axis.max_feedrate)"],
    ['{ "z", {  12.,  12. }, {   500.,  200. }, {  0.2,  0.4 } }', '{ "z", {  12.,  12. }, {   500.,  200. }, {  9.9,  9.9 } }'],
  ]) {
    const mutated = replaceRequired(source, needle, replacement);
    let rejected = false;
    try {
      const derived = axisDefinitions(mutated, numberList);
      rejected = needle.startsWith('{ "z"')
        && derived.get("machine_max_jerk_z").defaultValue !== "0.2,0.4";
    } catch {
      rejected = true;
    }
    if (!rejected) throw new Error(`axis mutation was accepted: ${needle}`);
  }
  const swapped = replaceRequired(
    replaceRequired(source, "ConfigOptionFloats(axis.max_feedrate)", "ConfigOptionFloats(axis.__swap__)"),
    "ConfigOptionFloats(axis.max_acceleration)",
    "ConfigOptionFloats(axis.max_feedrate)",
  ).replace("ConfigOptionFloats(axis.__swap__)", "ConfigOptionFloats(axis.max_acceleration)");
  try {
    axisDefinitions(swapped, numberList);
  } catch {
    const declarationSwapped = replaceRequired(
      replaceRequired(source, "std::vector<double> max_feedrate;", "std::vector<double> __swap__;"),
      "std::vector<double> max_acceleration;",
      "std::vector<double> max_feedrate;",
    ).replace("std::vector<double> __swap__;", "std::vector<double> max_acceleration;");
    try {
      axisDefinitions(declarationSwapped, numberList);
    } catch {
      return;
    }
    throw new Error("swapped AxisDefault declarations were accepted");
  }
  throw new Error("swapped axis member bindings were accepted");
}

function replaceRequired(source, needle, replacement) {
  const mutated = source.replace(needle, replacement);
  if (mutated === source) throw new Error(`missing mutation anchor: ${needle}`);
  return mutated;
}
