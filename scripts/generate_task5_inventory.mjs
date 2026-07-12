import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { deriveConsumerCitations, deriveExportRules, deriveMetadata } from "./task5_inventory/source.mjs";
import { verifySourceMutations } from "./task5_inventory/mutations.mjs";

const commit = "8500fcdccaa10b5099ac20d252af3a7c560046f1";
const repo = process.env.ORCA_SLICER_REPO ?? "OrcaSlicer";
const show = path => execFileSync("git", ["-C", repo, "show", `${commit}:${path}`], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
const sources = {
  config: show("src/libslic3r/Config.cpp"),
  header: show("src/libslic3r/PrintConfig.hpp"),
  print: show("src/libslic3r/PrintConfig.cpp"),
  preset: show("src/libslic3r/Preset.cpp"),
  presetHeader: show("src/libslic3r/Preset.hpp"),
  gcode: show("src/libslic3r/GCode.cpp"),
  model: show("src/libslic3r/Format/bbs_3mf.cpp"),
  constants: show("src/libslic3r/PrintConfigConstants.hpp"),
};
const fixture = JSON.parse(execFileSync("tar", ["-xOf", "tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf", "Metadata/project_settings.config"], { encoding: "utf8" }));
const keys = Object.keys(fixture).sort();

function stripComments(source) {
  return source.replace(/\/\*[\s\S]*?\*\//g, match => match.replace(/[^\n]/g, " ")).replace(/\/\/.*$/gm, "");
}

function vector(name) {
  const clean = stripComments(sources.preset);
  const start = clean.indexOf(name);
  if (start < 0) throw new Error(`missing ${name}`);
  const open = clean.indexOf("{", start);
  const close = clean.indexOf("};", open);
  return new Set([...clean.slice(open, close).matchAll(/"([^"]+)"/g)].map(match => match[1]));
}

const printerKeys = new Set([...vector("s_Preset_printer_options"), ...vector("s_Preset_machine_limits_options")]);
const processKeys = vector("s_Preset_print_options");
const filamentKeys = vector("s_Preset_filament_options");
filamentKeys.add("filament_colour");

function classFields(name) {
  const clean = stripComments(sources.header);
  const start = clean.indexOf(`${name},`);
  if (start < 0) throw new Error(`missing class ${name}`);
  const tail = clean.slice(start);
  const closing = /^\s*\)\s*$/m.exec(tail);
  if (!closing) throw new Error(`missing class close ${name}`);
  const next = start + closing.index;
  const fields = new Map();
  for (const match of clean.slice(start, next).matchAll(/\(\(ConfigOption([^,]+),\s*([A-Za-z0-9_]+)\)\)/g)) {
    fields.set(match[2], {
      type: cppType(match[1].trim()),
      cppType: match[1].trim(),
      line: clean.slice(0, start + match.index).split("\n").length,
    });
  }
  return fields;
}

function cppType(type) {
  if (type.startsWith("Enum<")) return "coEnum";
  if (type.startsWith("Enums")) return "coEnums";
  const base = type.replace("Nullable", "");
  return ({ Bool: "coBool", Bools: "coBools", Float: "coFloat", Floats: "coFloats", FloatOrPercent: "coFloatOrPercent", FloatOrPercents: "coFloatOrPercent", Int: "coInt", Ints: "coInts", Percent: "coPercent", Percents: "coPercents", Point: "coPoint", Points: "coPoints", PointsGroups: "coPointsGroups", String: "coString", Strings: "coStrings" })[base] ?? (() => { throw new Error(`unknown type ${type}`); })();
}

const owners = [
  ["MachineEnvelopeConfig", classFields("MachineEnvelopeConfig")],
  ["GCodeConfig", classFields("GCodeConfig")],
  ["PrintConfig", classFields("PrintConfig")],
  ["PrintObjectConfig", classFields("PrintObjectConfig")],
  ["PrintRegionConfig", classFields("PrintRegionConfig")],
];

const constants = new Map([...sources.constants.matchAll(/^#define\s+([A-Z0-9_]+)\s+([^\s/]+)/gm)].map(match => [match[1], match[2]]));
const enumValues = new Map();
for (const match of sources.print.matchAll(/\{\s*"([^"]+)"\s*,\s*([^}\n]+)\}/g)) {
  const symbol = normalizeEnum(match[2]);
  if (symbol && !enumValues.has(symbol)) enumValues.set(symbol, match[1]);
}

function normalizeEnum(value) {
  return value.replace(/\(int\)|int\(|\)|\s/g, "").split("::").at(-1);
}

const registrations = [...stripComments(sources.print).matchAll(/add\("([^"]+)",\s*(co[A-Za-z]+)\)/g)];
const definitions = new Map();
for (let index = 0; index < registrations.length; index++) {
  const match = registrations[index];
  if (!(match[1] in fixture)) continue;
  const block = stripComments(sources.print).slice(match.index, registrations[index + 1]?.index ?? sources.print.length);
  const expression = /set_default_value\(([\s\S]*?)\);/.exec(block)?.[1];
  if (!expression) throw new Error(`missing default expression ${match[1]}`);
  const definition = {
    type: match[2],
    defaultValue: evaluateDefault(match[2], expression),
    nullable: /nullable\s*=\s*true/.test(block),
  };
  const previous = definitions.get(match[1]);
  if (previous && JSON.stringify(previous) !== JSON.stringify(definition)) throw new Error(`conflicting duplicate ${match[1]}`);
  definitions.set(match[1], definition);
}
if (definitions.size !== 622) throw new Error(`expected 622 literal definitions, got ${definitions.size}`);

const axes = /std::vector<AxisDefault> axes\s*\{([\s\S]*?)\n\s*\};/.exec(stripComments(sources.print))?.[1];
if (!axes) throw new Error("missing axis defaults");
for (const match of axes.matchAll(/\{\s*"([xyze])"\s*,\s*\{([^}]+)\}\s*,\s*\{([^}]+)\}\s*,\s*\{([^}]+)\}\s*\}/g)) {
  for (const [prefix, values] of [["machine_max_speed_", match[2]], ["machine_max_acceleration_", match[3]], ["machine_max_jerk_", match[4]]]) {
    definitions.set(`${prefix}${match[1]}`, { type: "coFloats", defaultValue: numberList(values).join(",") });
  }
}

const overrideBlock = /filament_extruder_override_keys\s*=\s*\{([\s\S]*?)\};/.exec(stripComments(sources.print))?.[1];
if (!overrideBlock) throw new Error("missing filament override list");
if (!/for\s*\(auto& opt_key : filament_extruder_override_keys\)\s*\{[\s\S]*?this->add_nullable\(opt_key, it_opt->second\.type\)/.test(stripComments(sources.print))) {
  throw new Error("filament override keys are not registered through add_nullable");
}
const nullable = new Set();
for (const match of overrideBlock.matchAll(/"([^"]+)"/g)) {
  const key = match[1];
  const base = key.slice("filament_".length);
  const source = definitions.get(base);
  if (!source) throw new Error(`missing override base ${key}/${base}`);
  definitions.set(key, { ...source });
  nullable.add(key);
}
for (const key of keys) {
  const ownerField = owners.find(([, fields]) => fields.has(key))?.[1].get(key);
  if (ownerField && /Nullable/.test(ownerField.cppType)) nullable.add(key);
}
for (const [key, definition] of definitions) if (definition.nullable) nullable.add(key);
if (nullable.size !== 31) throw new Error(`expected 31 nullable definitions, got ${nullable.size}`);

function evaluateDefault(type, expression) {
  let value = expression.trim();
  for (const [name, replacement] of constants) value = value.replaceAll(name, replacement);
  if (value.includes("nil_value()") || /ntUndefine/.test(value)) return "nil";
  if (type === "coString" || type === "coStrings") {
    const decode = part => [...part.matchAll(/"((?:\\.|[^"\\])*)"/g)].map(match => JSON.parse(`"${match[1]}"`)).join("");
    if (type === "coString") return escapeCstyle(decode(value));
    const initializer = /\{([\s\S]*)\}/.exec(value)?.[1];
    if (initializer === undefined) return "";
    const strings = splitInitializer(initializer).map(decode);
    return escapeStringsCstyle(strings);
  }
  value = value.replace(/\s+/g, " ");
  if (type === "coEnum" || type === "coEnums") {
    const opening = value.indexOf(type === "coEnum" ? ">" : "Generic");
    const body = value.slice(opening + 1).replace(/^[^{(]*[{(]/, "").replace(/[})]\s*$/, "");
    const names = body.split(",").map(normalizeEnum).filter(Boolean).map(name => enumValues.get(name) ?? (() => { throw new Error(`unknown enum ${name} in ${value}`); })());
    return names.join(",");
  }
  if (type === "coBool" || type === "coBools") {
    const bools = [...value.matchAll(/\b(true|false|[01])\b/g)].map(match => match[1] === "true" || match[1] === "1" ? "1" : "0");
    if (!bools.length) throw new Error(`missing bool in ${value}`);
    return bools.join(",");
  }
  if (type === "coPoint" || type === "coPoints" || type === "coPointsGroups") {
    const points = [...value.matchAll(/Vec2d\s*\(\s*([^,]+),\s*([^)]+)\)/g)].map(match => `${number(match[1])}x${number(match[2])}`);
    if (type === "coPoint") return points[0]?.replace("x", ",") ?? "";
    return points.join(",");
  }
  if (type === "coFloatOrPercent") {
    const values = numberList(value.replace(/ConfigOptionFloatOrPercent/g, ""));
    const percent = /,\s*true\s*\)/.test(value);
    return `${values[0]}${percent ? "%" : ""}`;
  }
  const values = numberList(value.replace(/ConfigOption[A-Za-z]+/g, ""));
  if (!values.length) return "";
  if (type === "coPercent" || type === "coPercents") return values.map(item => `${item}%`).join(",");
  return values.join(",");
}

function numberList(value) {
  return [...value.matchAll(/[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?f?/g)].map(match => number(match[0]));
}

function number(value) {
  const parsed = Number(value.replace(/f$/, ""));
  if (!Number.isFinite(parsed)) throw new Error(`invalid number ${value}`);
  return formatNumber(parsed);
}

function formatNumber(value) {
  if (value === 0) return Object.is(value, -0) ? "-0" : "0";
  const [mantissa, rawExponent] = value.toExponential(5).split("e");
  const exponent = Number(rawExponent);
  if (exponent < -4 || exponent >= 6) {
    const trimmed = trimDecimal(mantissa);
    return `${trimmed}e${exponent >= 0 ? "+" : "-"}${Math.abs(exponent).toString().padStart(2, "0")}`;
  }
  const decimals = Math.max(0, 5 - exponent);
  return trimDecimal(value.toFixed(decimals));
}

function trimDecimal(value) {
  return value.includes(".") ? value.replace(/0+$/, "").replace(/\.$/, "") : value;
}

function splitInitializer(value) {
  const parts = [];
  let start = 0;
  let depth = 0;
  let quoted = false;
  let escaped = false;
  for (let index = 0; index < value.length; index++) {
    const character = value[index];
    if (quoted) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === '"') quoted = false;
      continue;
    }
    if (character === '"') quoted = true;
    else if (character === "(" || character === "{") depth++;
    else if (character === ")" || character === "}") depth--;
    else if (character === "," && depth === 0) {
      parts.push(value.slice(start, index));
      start = index + 1;
    }
  }
  parts.push(value.slice(start));
  return parts;
}

function escapeCstyle(value) {
  return value.replaceAll("\\", "\\\\").replaceAll('"', '\\"').replaceAll("\r", "\\r").replaceAll("\n", "\\n");
}

function escapeStringsCstyle(values) {
  return values.map(value => {
    const quote = (values.length === 1 && value === "") || /[ \t\\"\r\n]/.test(value);
    const escaped = escapeCstyle(value);
    return quote ? `"${escaped}"` : escaped;
  }).join(";");
}

function rawScope(key) {
  if (printerKeys.has(key)) return "printer";
  if (processKeys.has(key)) return "process";
  if (filamentKeys.has(key)) return "filament";
  return "residual";
}

function owner(key) {
  for (const [name, fields] of owners) if (fields.has(key)) return ({ MachineEnvelopeConfig: "machine_envelope_config", GCodeConfig: "g_code_config", PrintConfig: "print_config", PrintObjectConfig: "print_object_config", PrintRegionConfig: "print_region_config" })[name];
  return "unowned";
}

function projections(key) {
  const result = [];
  if (owners[3][1].has(key)) result.push("object");
  if (owners[4][1].has(key)) result.push("region");
  if (owners[1][1].has(key)) result.push("g_code");
  return result;
}

function lineOf(source, needle) {
  const offset = source.indexOf(needle);
  if (offset < 0) return 0;
  return source.slice(0, offset).split("\n").length;
}

function definitionCitation(key) {
  if (metadata.has(key)) return metadata.get(key);
  const literal = `add(\"${key}\"`;
  const line = lineOf(sources.print, literal);
  if (line) return { path: "src/libslic3r/PrintConfig.cpp", line, symbol: key };
  const generatedLine = lineOf(sources.print, `\"${key}\"`);
  if (generatedLine) return { path: "src/libslic3r/PrintConfig.cpp", line: generatedLine, symbol: key };
  const headerLine = lineOf(sources.header, `,               ${key})`) || lineOf(sources.header, `,  ${key})`) || lineOf(sources.header, key);
  if (!headerLine) throw new Error(`no definition citation ${key}`);
  return { path: "src/libslic3r/PrintConfig.hpp", line: headerLine, symbol: key };
}

const legacyRecords = new Map();
{
  const start = sources.print.indexOf("void PrintConfigDef::handle_legacy");
  const end = sources.print.indexOf("void PrintConfigDef::handle_legacy_composite", start);
  const body = sources.print.slice(start, end);
  for (const match of body.matchAll(/opt_key\s*==\s*"([^"]+)"/g)) {
    const oldKey = match[1];
    const conditionEnd = body.indexOf(")", match.index);
    const brace = body.indexOf("{", conditionEnd);
    const semicolon = body.indexOf(";", conditionEnd);
    let branch;
    if (brace >= 0 && brace < semicolon) {
      let depth = 1;
      let cursor = brace + 1;
      while (depth && cursor < body.length) {
        if (body[cursor] === "{") depth++;
        if (body[cursor] === "}") depth--;
        cursor++;
      }
      branch = body.slice(brace + 1, cursor - 1);
    } else {
      branch = body.slice(conditionEnd + 1, semicolon + 1);
    }
    const renamed = /opt_key\s*=\s*"([^"]+)"/.exec(branch)?.[1];
    const target = renamed && renamed in fixture ? renamed : oldKey in fixture ? oldKey : null;
    if (!target) continue;
    const line = sources.print.slice(0, start + match.index).split("\n").length;
    const records = legacyRecords.get(target) ?? [];
    const conversions = [];
    if (renamed) conversions.push("rename");
    if (/\bvalue\s*=(?!=)|\bvalue\.(?:clear|erase)|ReplaceString\(value/.test(branch)) conversions.push("value_conversion");
    if (/opt_key\s*=\s*""/.test(branch) && oldKey in fixture) conversions.push("value_conversion");
    for (const conversion of conversions) {
      const record = { key: oldKey, conversion, citation: { path: "src/libslic3r/PrintConfig.cpp", line, symbol: oldKey } };
      if (!records.some(item => item.key === record.key && item.conversion === record.conversion)) records.push(record);
    }
    if (!records.length) continue;
    legacyRecords.set(target, records);
  }
  const line = lineOf(sources.print, "perimeter_feed_rate");
  legacyRecords.set("inner_wall_speed", [...(legacyRecords.get("inner_wall_speed") ?? []), {
    key: "perimeter_feed_rate",
    conversion: "rename",
    citation: { path: "src/libslic3r/PrintConfig.cpp", line, symbol: "perimeter_feed_rate" },
  }]);
}

function legacyInputs(key) {
  return legacyRecords.get(key) ?? [];
}

function canonicalDefault(type, value) {
  if (type === "coBool" || type === "coBools") {
    return value === "true" ? "1" : value === "false" ? "0" : value;
  }
  if ((type === "coPercent" || type === "coPercents") && value !== "nil") {
    return value.split(",").map(item => item === "nil" || item.endsWith("%") ? item : `${item}%`).join(",");
  }
  if (type === "coPoint" && !value.includes(",")) {
    const point = /^([^x]+)x([^x]+)$/.exec(value);
    return point ? `${point[1]},${point[2]}` : value;
  }
  return value;
}

const metadata = deriveMetadata(new Set(definitions.keys()), keys, sources.config, sources.presetHeader);
const exportRules = deriveExportRules(sources.gcode, keys);
for (const key of exportRules.keys()) if (nullable.has(key)) throw new Error(`overlapping special/nullable export ${key}`);
if (process.argv.includes("--verify-mutations")) {
  verifySourceMutations(sources, definitions, keys);
  process.stdout.write("verified 11 source-semantics mutations\n");
  process.exit(0);
}
const consumerCitations = deriveConsumerCitations(keys, sources, new Set(metadata.keys()));

const rows = keys.map(key => {
  const isMetadata = metadata.has(key);
  const declared = definitions.get(key);
  if (!isMetadata && !declared) throw new Error(`missing definition ${key}`);
  const citation = definitionCitation(key);
  const special = exportRules.get(key);
  const upstreamConsumers = consumerCitations.get(key);
  if (!upstreamConsumers?.length) throw new Error(`missing consumer citation ${key}`);
  return {
    key,
    raw_scope: rawScope(key),
    static_owner: owner(key),
    option_type: isMetadata ? "Metadata" : declared.type,
    nullable: nullable.has(key),
    default_serialized: isMetadata ? "" : canonicalDefault(declared.type, declared.defaultValue),
    wire_shape: Array.isArray(fixture[key]) ? "array" : "scalar_string",
    effective_projections: projections(key),
    legacy_inputs: legacyInputs(key),
    config_export: isMetadata ? { kind: "metadata_exclusion" } : special ? { kind: "fixed_tag_special", rule: special } : nullable.has(key) ? { kind: "omit_when_nil" } : { kind: "canonical" },
    upstream_definition: citation,
    upstream_consumers: upstreamConsumers,
  };
});

const counts = Object.fromEntries(rows.reduce((map, row) => map.set(row.raw_scope, 1 + (map.get(row.raw_scope) ?? 0)), new Map()));
const expectedCounts = { process: 352, filament: 122, printer: 132, residual: 47 };
if (JSON.stringify(counts) !== JSON.stringify(expectedCounts)) throw new Error(`wrong scope counts ${JSON.stringify(counts)}`);
const projectionCounts = Object.fromEntries(["object", "region", "g_code"].map(projection => [projection, rows.filter(row => row.effective_projections.includes(projection)).length]));
if (JSON.stringify(projectionCounts) !== JSON.stringify({ object: 126, region: 153, g_code: 149 })) throw new Error(`wrong projection counts ${JSON.stringify(projectionCounts)}`);
if (rows.filter(row => row.nullable).length !== 31) throw new Error("wrong nullable count");
const exportCounts = Object.fromEntries(rows.reduce((map, row) => map.set(row.config_export.kind, 1 + (map.get(row.config_export.kind) ?? 0)), new Map()));
if (JSON.stringify(exportCounts) !== JSON.stringify({ canonical: 615, fixed_tag_special: 4, omit_when_nil: 31, metadata_exclusion: 3 })) throw new Error(`wrong export counts ${JSON.stringify(exportCounts)}`);
const output = `${JSON.stringify(rows, null, 2)}\n`;
if (process.argv.includes("--stdout")) {
  process.stdout.write(output);
} else if (process.argv.includes("--write")) {
  writeFileSync("tests/ksr_fdmtest_v4/options-v242.json", output);
} else {
  throw new Error("pass --stdout or --write");
}
