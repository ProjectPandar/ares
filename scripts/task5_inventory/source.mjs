export function deriveExportRules(source, fixtureKeys) {
  const start = source.indexOf("void GCode::append_full_config");
  if (start < 0) throw new Error("missing append_full_config");
  const body = balancedBlock(source, source.indexOf("{", start));
  const rules = new Map();

  const multiplierRead = /std::vector<double>\s+(\w+)\s*=\s*cfg\.option<ConfigOptionFloats>\("([^"]+)"\)->values/.exec(body);
  const matrixRead = /std::vector<double>\s+(temp_flush_volumes_matrix)\s*=\s*cfg\.option<ConfigOptionFloats>\("([^"]+)"\)->values/.exec(body);
  const matrixWrite = /cfg\.option<ConfigOptionFloats>\("([^"]+)"\)->values\s*=\s*(temp_flush_volumes_matrix)/.exec(body);
  if (!multiplierRead
      || !matrixRead
      || !matrixWrite
      || multiplierRead[1] !== "temp_cfg_flush_multiplier"
      || multiplierRead[2] !== "flush_multiplier"
      || matrixRead[2] !== matrixWrite[1]
      || matrixRead[1] !== matrixWrite[2]
      || !body.includes("temp_cfg_flush_multiplier_idx = temp_cfg_flush_multiplier[idx]")
      || !/std::transform\(\s*temp_flush_volumes_matrix\.begin\(\)\s*\+\s*temp_begin_t\s*,\s*temp_flush_volumes_matrix\.begin\(\)\s*\+\s*temp_end_t\s*,\s*temp_flush_volumes_matrix\.begin\(\)\s*\+\s*temp_begin_t\s*,\s*\[temp_cfg_flush_multiplier_idx\]\(double\s+inputx\)\s*\{\s*return std::round\(inputx\s*\*\s*temp_cfg_flush_multiplier_idx\);\s*\}\s*\);/.test(body)) {
    throw new Error("missing flush matrix multiplier/round export semantics");
  }
  const sizeGuard = /if\s*\(filament_count_tmp \* filament_count_tmp \* heads_count_tmp == temp_flush_volumes_matrix\.size\(\)\)\s*\{/.exec(body);
  const guardedFlush = sizeGuard
    ? balancedBlock(body, sizeGuard.index + sizeGuard[0].lastIndexOf("{"))
    : "";
  if (!guardedFlush.includes(matrixWrite[0])) throw new Error("flush write is outside its size guard");
  rules.set(matrixRead[2], "scaled_flush_matrix");

  const banned = /static const std::set<std::string_view>\s+(\w+)\s*\(\s*\{([\s\S]*?)\}\s*\);/.exec(body);
  if (!banned) throw new Error("missing banned config-key set");
  const bannedKeys = new Set([...banned[2].matchAll(/"([^"]+)"sv/g)].map(match => match[1]));
  if (!bannedKeys.size
      || fixtureKeys.some(key => bannedKeys.has(key))
      || !new RegExp(`return\\s+${banned[1]}\\.find\\(key\\)\\s*!=\\s*${banned[1]}\\.end\\(\\)\\s*;`).test(body)) {
    throw new Error("invalid or fixture-overlapping banned config-key semantics");
  }
  const loop = /for\s*\(const std::string\s*&\s*key\s*:\s*cfg\.keys\(\)\)\s*\{/.exec(body);
  if (!loop) throw new Error("missing cfg.keys export loop");
  const loopBody = balancedBlock(body, loop.index + loop[0].lastIndexOf("{"));
  const guard = /if\s*\(\s*!is_banned\(key\)\s*&&\s*!cfg\.option\(key\)->is_nil\(\)\s*\)\s*\{/.exec(loopBody);
  if (!guard) throw new Error("missing banned-and-nil export guard");
  const guardRange = blockRange(loopBody, guard.index + guard[0].lastIndexOf("{"));
  if (loopBody.slice(0, guard.index).trim() || loopBody.slice(guardRange.closing + 1).trim()) {
    throw new Error("unexpected statement outside export guard");
  }
  const guardBody = guardRange.body;

  const wipeCondition = /if\s*\(key == "([^"]+)" \|\| key == "([^"]+)"\)\s*\{/.exec(guardBody);
  const wipeRange = wipeCondition
    ? blockRange(guardBody, wipeCondition.index + wipeCondition[0].lastIndexOf("{"))
    : null;
  const wipeBody = wipeCondition
    ? wipeRange.body
    : "";
  if (!wipeCondition
      || guardBody.slice(0, wipeCondition.index).trim()
      || !wipeBody.includes("std::setprecision(3)")
      || !wipeBody.includes('<< "; " << key << " = "')
      || !/ConfigOptionFloats\s*\*>\s*\(cfg\.option\(key\)\)->get_at\(print\.get_plate_index\(\)\)/.test(wipeBody)) {
    throw new Error("missing plate-indexed fixed-3 wipe tower export semantics");
  }
  rules.set(wipeCondition[1], "plate_coordinate_duplicate");
  rules.set(wipeCondition[2], "plate_coordinate_duplicate");

  const substitution = /if\s*\(\s*key == "([^"]+)"\s*\)\s*ss\s*<<\s*"; "\s*<<\s*key\s*<<\s*" = "\s*<<\s*cfg\.opt_serialize\("([^"]+)"\)\s*<<\s*"\\n"\s*;\s*else\s*ss\s*<<\s*"; "\s*<<\s*key\s*<<\s*" = "\s*<<\s*cfg\.opt_serialize\(key\)\s*<<\s*"\\n"\s*;/.exec(guardBody);
  if (!substitution
      || substitution[1] === substitution[2]
      || guardBody.slice(wipeRange.closing + 1, substitution.index).trim()
      || guardBody.slice(substitution.index + substitution[0].length).trim()
      || topLevelIfCount(guardBody) !== 2) {
    throw new Error("missing cross-key colour export substitution");
  }
  rules.set(substitution[1], "filament_colour_substitution");
  if (rules.size !== 4) throw new Error(`expected 4 fixed export rules, got ${rules.size}`);
  return rules;
}

export function deriveMetadata(registeredKeys, fixtureKeys, configSource, presetHeader) {
  const start = configSource.indexOf("void ConfigBase::save_to_json");
  if (start < 0) throw new Error("missing ConfigBase::save_to_json");
  const opening = configSource.indexOf("{", start);
  const body = balancedBlock(configSource, opening);
  const loop = body.search(/for\s*\([^)]*:\s*this->keys\(\)\s*\)/);
  if (loop < 0) throw new Error("missing save_to_json config-key loop");
  const headers = [...body.slice(0, loop).matchAll(/j\[([A-Z][A-Z0-9_]*)\]\s*=\s*[A-Za-z_][A-Za-z0-9_]*\s*;/g)];
  const macros = new Map([...presetHeader.matchAll(/^#define\s+([A-Z][A-Z0-9_]*)\s+"([^"]+)"\s*$/gm)].map(match => [match[1], match[2]]));
  const metadata = new Map(headers.map(match => {
    const key = macros.get(match[1]);
    if (!key) throw new Error(`missing JSON header macro ${match[1]}`);
    return [key, {
      path: "src/libslic3r/Config.cpp",
      line: configSource.slice(0, opening + 1 + match.index).split("\n").length,
      symbol: match[1],
    }];
  }));
  if (metadata.size !== headers.length) throw new Error("duplicate save_to_json metadata key");
  const unregistered = fixtureKeys.filter(key => !registeredKeys.has(key));
  if (unregistered.length !== metadata.size || unregistered.some(key => !metadata.has(key))) {
    throw new Error("fixture/config registration partition differs from save_to_json metadata");
  }
  return metadata;
}

export function deriveConsumerCitations(keys, loadedSources, metadataKeys) {
  const citations = new Map();
  const genericLine = loadedSources.gcode
    .split("\n")
    .findIndex(line => line.includes("!cfg.option(key)->is_nil()")) + 1;
  if (!genericLine) throw new Error("missing generic config guard consumer");
  const metadataLine = loadedSources.model
    .split("\n")
    .findIndex(line => line.includes("config.save_to_json(temp_file")) + 1;
  if (!metadataLine) throw new Error("missing metadata save consumer");
  for (const key of keys) {
    citations.set(key, [{
      path: metadataKeys.has(key)
        ? "src/libslic3r/Format/bbs_3mf.cpp"
        : "src/libslic3r/GCode.cpp",
      line: metadataKeys.has(key) ? metadataLine : genericLine,
      symbol: metadataKeys.has(key) ? "save_to_json" : "cfg.option(key)->is_nil()",
    }]);
  }
  if (citations.size !== keys.length) throw new Error("incomplete consumer citation set");
  return citations;
}

function topLevelIfCount(source) {
  const code = maskCommentsAndStrings(source);
  let depth = 0;
  let count = 0;
  for (const match of code.matchAll(/[{}]|\bif\b/g)) {
    if (match[0] === "{") depth++;
    else if (match[0] === "}") depth--;
    else if (depth === 0) count++;
  }
  return count;
}

function maskCommentsAndStrings(source) {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, match => match.replace(/[^\n]/g, " "))
    .replace(/\/\/.*$/gm, match => match.replace(/[^\n]/g, " "))
    .replace(/"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g, match => match.replace(/[^\n]/g, " "));
}

function blockRange(source, opening) {
  if (opening < 0 || source[opening] !== "{") throw new Error("missing block opening");
  const code = maskCommentsAndStrings(source);
  let depth = 1;
  let cursor = opening + 1;
  while (depth && cursor < code.length) {
    if (code[cursor] === "{") depth++;
    if (code[cursor] === "}") depth--;
    cursor++;
  }
  if (depth) throw new Error("unterminated block");
  return { body: source.slice(opening + 1, cursor - 1), closing: cursor - 1 };
}

function balancedBlock(source, opening) {
  return blockRange(source, opening).body;
}
