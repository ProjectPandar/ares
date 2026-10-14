// Writes the plate custom G-code emission fixtures (docs/architecture/plate-custom-gcode-emission.md)
// into tests/parity/plate-custom-gcode-emission/: tests/parity/orca_cli_ender3.3mf (cube10, one
// filament, layers 0.2 .. 10.0 step 0.2, machine_pause_gcode "M25") plus one
// Metadata/custom_gcode_per_layer.xml entry, and for two cases one Metadata/project_settings.config
// edit. Every other entry is copied byte for byte.
// Type ids (CustomGCode.hpp:14-22): 0 ColorChange, 1 PausePrint, 2 ToolChange, 3 Template, 4 Custom.
// Usage: node tools/chain-probe/plate-custom-gcode-emission-fixtures.mjs
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { crc32, deflateRawSync, inflateRawSync } from "node:zlib";

const SOURCE = "tests/parity/orca_cli_ender3.3mf";
const ENTRY = "Metadata/custom_gcode_per_layer.xml";
const SETTINGS = "Metadata/project_settings.config";
const OUT = "tests/parity/plate-custom-gcode-emission";

const item = (top_z, type, extruder, color = "", extra = "") =>
  `<layer top_z="${top_z}" type="${type}" extruder="${extruder}" color="${color}" extra="${extra}"/>`;
const mode = (value) => `<mode value="${value}"/>`;
const entry = (...children) =>
  `<?xml version="1.0" encoding="utf-8"?>\n<custom_gcodes_per_layer><plate><plate_info id="1"/>${children.join("")}</plate></custom_gcodes_per_layer>`.replace(
    /></g,
    ">\n<",
  );

const FIXTURES = [
  // PausePrint on layer 3: tag + machine_pause_gcode; the message is not emitted (GCode.cpp:4233-4241).
  { name: "pause", xml: entry(item("0.6", 1, 1, "", "Insert magnet")) },
  // Custom on layer 5: tag + raw multi-line extra (GCode.cpp:4242-4250).
  { name: "custom", xml: entry(item("1", 4, 1, "", "G4 S1&#10;M117 custom [layer_num]")) },
  // Template on layer 7: tag + template_custom_gcode through the placeholder parser (GCode.cpp:4245-4246).
  {
    name: "template",
    xml: entry(item("1.4", 3, 1)),
    settings: { template_custom_gcode: "M117 template T[current_extruder] H[layer_height]\nG4 P10" },
  },
  // ColorChange on layer 9: the M600 block is #if 0, so tag + empty extra (GCode.cpp:4203-4231, :4242-4250).
  { name: "color-change", xml: entry(item("1.8", 0, 1, "#00FF00")) },
  // Two events inside layer 11's span (2.1, 2.3]: the last one wins (ToolOrdering.cpp:1513).
  { name: "same-layer", xml: entry(item("2.15", 4, 1, "", "M117 first"), item("2.2", 4, 1, "", "M117 second")) },
  // Below the first layer's midpoint 0.1: never assigned (ToolOrdering.cpp:1495-1498).
  // Above the top layer 10.0: lands on the top layer (print_z_above = lowest(), :1478, :1486).
  { name: "bounds", xml: entry(item("0.05", 4, 1, "", "M117 below"), item("12", 4, 1, "", "M117 above")) },
  // Model mode MultiExtruder vs print mode SingleExtruder: the color change is ignored, the pause kept (:1473).
  { name: "mode-mismatch", xml: entry(item("0.6", 0, 1, "#00FF00"), item("1", 1, 1), mode("MultiExtruder")) },
  // Every ToolChange is skipped (:1485-1487); only the Custom on layer 7 is emitted.
  { name: "tool-change", xml: entry(item("0.6", 2, 1), item("1.4", 4, 1, "", "M117 after tool change"), mode("MultiAsSingle")) },
  // Sequential printing never assigns or emits (GCode.cpp:2758-2760, :4825).
  { name: "by-object", xml: entry(item("0.6", 1, 1)), settings: { print_sequence: "by object" } },
];

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

const source = unzip(readFileSync(SOURCE));
mkdirSync(OUT, { recursive: true });
for (const { name, xml, settings } of FIXTURES) {
  const entries = source.map(([path, data]) => {
    if (path !== SETTINGS || settings === undefined) return [path, data];
    const config = JSON.parse(data.toString("utf8"));
    for (const [key, value] of Object.entries(settings)) {
      if (!(key in config)) throw new Error(`${name}: ${key} is not in ${SETTINGS}`);
      config[key] = value;
    }
    return [path, Buffer.from(`${JSON.stringify(config, null, 4)}\n`)];
  });
  writeFileSync(`${OUT}/${name}.3mf`, zip([...entries, [ENTRY, Buffer.from(xml)]]));
}
