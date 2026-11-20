// Live Native authority for the legend Custom G-code overview
// (docs/architecture/web-custom-gcode-overview.md). Inputs stay in Git; all Native
// outputs stay outside it. The plate list comes from the slice A probe
// (ARES_PLATE_CUSTOM_GCODES_BIN), the layers from the Ares G-code processed by
// ARES_CUSTOM_GCODE_OVERVIEW_BIN.
// `--write-fixture` writes the committed input tests/parity/custom-gcode-overview/overview.3mf:
// tests/parity/orca_cli_ender3.3mf, every entry copied byte for byte, plus one
// Metadata/custom_gcode_per_layer.xml (bbs_3mf.cpp:220, layout bbs_3mf.cpp:8462-8510).
// It also writes two-plates.3mf: the same cube, a second instance on plate 2 at the
// plate grid origin (PartPlate.hpp:38-48, 220 mm bed x 1.2), and different lists per
// plate, so slicing plate 2 must show plate 2's list (OrcaSlicer.cpp:5690).
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { crc32, deflateRawSync, inflateRawSync } from "node:zlib";
import { execFileSync } from "node:child_process";
import { runNative } from "./marker-time-runner.mjs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { resolve, sep } from "node:path";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const sha = value => createHash("sha256").update(value).digest("hex");
const required = name => {
  if (!process.env[name]) throw new Error(`${name} is required`);
  return process.env[name];
};

const typed = (top_z, type, extruder, color = "", extra = "") =>
  `<layer top_z="${top_z}" type="${type}" extruder="${extruder}" color="${color}" extra="${extra}"/>`;

// Plate 1, against the sliced layer zs (0.2 to 10, step 0.2, as f32), reaches every
// find_close_layer_idx case (GCodeViewer.cpp:108-125, eps 0.0011) and every label
// (GCodeViewer.cpp:4517-4527), in file order (not sorted by z).
const OVERVIEW_XML = `<?xml version="1.0" encoding="utf-8"?>
<custom_gcodes_per_layer><plate><plate_info id="1"/>${[
  typed("0.40000000596046448", 0, 1, "#FF0000"), // exact f32 layer z
  typed("1.1995", 1, 1, "", "upper layer nearer"), // within eps, nearer the upper layer
  typed("0.900000005960464477539062", 2, 2, "#00FF00"), // equidistant between two layers
  typed("0.1", 3, 1), // below the first layer, outside eps
  typed("10.0009", 4, 1, "", "G4 S1"), // above the last layer, within eps
  typed("0.2", 5, 1), // layer 0
  typed("1.6005", 1, 1, "", "lower layer nearer"), // within eps, nearer the lower layer
  typed("0.1995", 0, 1, "#0000FF"), // below the first layer, within eps
  typed("1000", 4, 1, "", "M117 above"), // above the last layer, outside eps
].join("")}</plate></custom_gcodes_per_layer>`.replace(/></g, ">\n<");

const PLATE_TWO = `<plate><plate_info id="2"/>${[
  typed("2", 1, 1, "", "plate 2 pause"),
  typed("5.0005", 3, 1),
  typed("0.5", 4, 1, "", "M117 plate 2"),
].join("")}</plate>`.replace(/></g, ">\n<");
const TWO_PLATES_XML = OVERVIEW_XML.replace("</custom_gcodes_per_layer>", `${PLATE_TWO}\n</custom_gcodes_per_layer>`);

/** `orca_cli_ender3.3mf` entries with object 3, an instance of the cube, on plate 2. */
function twoPlates(entries) {
  const edit = (name, from, to) => {
    const entry = entries.find(([entryName]) => entryName === name);
    const text = entry[1].toString("utf8");
    if (text.split(from).length !== 2) throw new Error(`${name}: expected one ${from}`);
    entry[1] = Buffer.from(text.replace(from, to));
  };
  edit("3D/3dmodel.model", " </resources>", `  <object id="3" p:UUID="00000003-61cb-4c03-9d28-80fed5dfa1dc" type="model">
   <components>
    <component p:path="/3D/Objects/cube10.stl_1.model" objectid="1" p:UUID="00030000-b206-40ff-9872-83e8017abed1" transform="1 0 0 0 1 0 0 0 1 0 0 0"/>
   </components>
  </object>
 </resources>`);
  edit("3D/3dmodel.model", " </build>", `  <item objectid="3" p:UUID="00000004-b1ec-4553-aec9-835e5b724bb4" transform="1 0 0 0 1 0 0 0 1 374 110 5" printable="1" auto_drop="1"/>
 </build>`);
  const config = entries.find(([name]) => name === "Metadata/model_settings.config")[1].toString("utf8");
  const object = config.slice(config.indexOf("  <object id=\"2\">"), config.indexOf("  <plate>"));
  const plate = config.slice(config.indexOf("  <plate>"), config.indexOf("  <assemble>"));
  // _add_model_config_file_to_archive writes every object, then every plate (bbs_3mf.cpp:7800-7916).
  edit("Metadata/model_settings.config", "  <plate>", `${object.replace('<object id="2">', '<object id="3">')}  <plate>`);
  edit("Metadata/model_settings.config", "  <assemble>", `${plate
    .replace('"plater_id" value="1"', '"plater_id" value="2"')
    .replace('"object_id" value="2"', '"object_id" value="3"')
    .replace('"identify_id" value="14"', '"identify_id" value="15"')}  <assemble>`);
  edit("Metadata/filament_sequence.json", "}}", '},"plate_2":{"nozzle_sequence":[],"optimal_assignment":[],"sequence":[]}}');
  return entries;
}

function unzip(bytes) {
  const end = bytes.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  const count = bytes.readUInt16LE(end + 10);
  let at = bytes.readUInt32LE(end + 16);
  const entries = [];
  for (let i = 0; i < count; i++) {
    const method = bytes.readUInt16LE(at + 10);
    const size = bytes.readUInt32LE(at + 20);
    const nameLength = bytes.readUInt16LE(at + 28);
    const skip = nameLength + bytes.readUInt16LE(at + 30) + bytes.readUInt16LE(at + 32);
    const local = bytes.readUInt32LE(at + 42);
    const name = bytes.toString("utf8", at + 46, at + 46 + nameLength);
    const start = local + 30 + bytes.readUInt16LE(local + 26) + bytes.readUInt16LE(local + 28);
    const body = bytes.subarray(start, start + size);
    entries.push([name, method === 0 ? Buffer.from(body) : inflateRawSync(body)]);
    at += 46 + skip;
  }
  return entries;
}

function zip(entries) {
  const local = [];
  const central = [];
  let offset = 0;
  for (const [name, data] of entries) {
    const nameBytes = Buffer.from(name);
    const body = deflateRawSync(data);
    const header = Buffer.alloc(30);
    header.writeUInt32LE(0x04034b50, 0);
    header.writeUInt16LE(20, 4);
    header.writeUInt16LE(8, 8);
    header.writeUInt32LE(crc32(data), 14);
    header.writeUInt32LE(body.length, 18);
    header.writeUInt32LE(data.length, 22);
    header.writeUInt16LE(nameBytes.length, 26);
    const entry = Buffer.alloc(46);
    entry.writeUInt32LE(0x02014b50, 0);
    entry.writeUInt16LE(20, 4);
    entry.writeUInt16LE(20, 6);
    entry.writeUInt16LE(8, 10);
    entry.writeUInt32LE(crc32(data), 16);
    entry.writeUInt32LE(body.length, 20);
    entry.writeUInt32LE(data.length, 24);
    entry.writeUInt16LE(nameBytes.length, 28);
    entry.writeUInt32LE(offset, 42);
    local.push(header, nameBytes, body);
    central.push(entry, nameBytes);
    offset += header.length + nameBytes.length + body.length;
  }
  const directory = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(entries.length, 8);
  end.writeUInt16LE(entries.length, 10);
  end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...local, directory, end]);
}

function writeFixture() {
  const source = unzip(readFileSync(`${repo}tests/parity/orca_cli_ender3.3mf`));
  mkdirSync(`${repo}tests/parity/custom-gcode-overview`, { recursive: true });
  const entry = ["Metadata/custom_gcode_per_layer.xml", Buffer.from(OVERVIEW_XML)];
  writeFileSync(`${repo}tests/parity/custom-gcode-overview/overview.3mf`, zip([...source, entry]));
  const two = twoPlates(unzip(readFileSync(`${repo}tests/parity/orca_cli_ender3.3mf`)));
  writeFileSync(`${repo}tests/parity/custom-gcode-overview/two-plates.3mf`, zip([...two, ["Metadata/custom_gcode_per_layer.xml", Buffer.from(TWO_PLATES_XML)]]));
}

/** `plate` is the zero-based key of the sliced plate. */
export async function generateCustomGCodeOverview(directory, project, gcode, plate = 0) {
  const probe = required("ARES_PLATE_CUSTOM_GCODES_BIN");
  const binary = required("ARES_CUSTOM_GCODE_OVERVIEW_BIN");
  const dir = resolve(directory);
  if (dir === resolve(repo) || dir.startsWith(`${resolve(repo)}${sep}`)) throw new Error("Native outputs must stay outside the repository");
  for (const name of ["items.json", "native.json"]) {
    if (existsSync(`${dir}/${name}`)) throw new Error(`stale Native artifact already exists: ${name}`);
  }
  mkdirSync(dir, { recursive: true });
  execFileSync(process.execPath, [`${repo}tools/chain-probe/extract-custom-gcode-overview.mjs`, "--check"], { stdio: "inherit" });
  copyFileSync(project, `${dir}/input.3mf`);
  copyFileSync(gcode, `${dir}/ares.gcode`);
  const workspacePackage = readFileSync(`${repo}Cargo.toml`, "utf8").split(/^\[workspace\.package\]\s*$/m)[1].split(/^\[/m)[0];
  const [, version] = workspacePackage.match(/^version\s*=\s*"([^"]+)"\s*$/m);
  const banner = `; generated by OrcaSlicer (Ares ${version}) on 1980-01-01 at 00:00:00`;
  const ares = readFileSync(gcode);
  const generators = ares.toString("utf8").split("\n").filter(line => line.startsWith("; generated by OrcaSlicer (Ares "));
  if (generators.length !== 1 || generators[0] !== banner) throw new Error(`expected one exact generator line: ${banner}`);
  // GCodeProcessor::detect_producer matches "generated by OrcaSlicer" as a substring (GCodeProcessor.cpp:5164–5175).
  writeFileSync(`${dir}/input.gcode`, ares);
  mkdirSync(`${dir}/probe`);
  const probeCommand = ["--slice", "0", "--outputdir", `${dir}/probe`, `${dir}/input.3mf`];
  execFileSync(probe, probeCommand, { env: { ...process.env, ARES_PLATE_CUSTOM_GCODES_DUMP: `${dir}/items.json` }, stdio: ["ignore", "ignore", "pipe"] });
  const command = ["--process-gcode", `${dir}/input.gcode`, "--dump-custom-gcode-overview", `${dir}/items.json`, String(plate), `${dir}/native.json`];
  const stdout = (await runNative(binary, command, `${dir}/native`, performance.now() + 300_000)).toString("utf8");
  writeFileSync(`${dir}/stdout.log`, stdout);
  const native = JSON.parse(readFileSync(`${dir}/native.json`, "utf8"));
  // Mechanical projection only; expected values remain C++ output.
  writeFileSync(`${dir}/layers.tsv`, native.zsBits.map((z, i) => `${z}\t${native.timesBits[i]}\n`).join(""));
  writeFileSync(`${dir}/rows.tsv`, native.rows.map(r => `${r.label}\t${r.layer}\t${r.timeBits}\t${r.time}\n`).join(""));
  const files = ["input.3mf", "ares.gcode", "input.gcode", "items.json", "native.json", "layers.tsv", "rows.tsv"];
  writeFileSync(`${dir}/oracle.json`, `${JSON.stringify({
    kind: native.oracle, upstream: native.upstream, plate,
    probe, probeSha256: sha(readFileSync(probe)), probeCommand: [probe, ...probeCommand],
    binary, binarySha256: sha(readFileSync(binary)), command: [binary, ...command], stdout,
    extractionManifestSha256: sha(readFileSync(`${repo}tools/chain-probe/custom-gcode-overview/extraction.json`)),
    rows: native.rows.length,
    files: Object.fromEntries(files.map(name => [name, sha(readFileSync(`${dir}/${name}`))])),
  }, null, 2)}\n`);
  return dir;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url) && process.argv[2] === "--write-fixture") {
  writeFixture();
} else if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [directory, project, gcode, plate = "0"] = process.argv.slice(2);
  if (!directory || !project || !gcode) throw new Error("usage: <external artifact directory> <project.3mf> <ares.gcode> [plate key]");
  console.log(`live native custom-gcode-overview artifacts: ${await generateCustomGCodeOverview(directory, project, gcode, Number(plate))}`);
}
