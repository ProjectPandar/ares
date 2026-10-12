// Writes the plate custom G-codes fixtures (docs/architecture/project-plate-custom-gcodes.md)
// into tests/parity/plate-custom-gcodes/: tests/parity/orca_cli_ender3.3mf, every entry
// copied byte for byte, plus one Metadata/custom_gcode_per_layer.xml entry
// (bbs_3mf.cpp:220) in the layout _BBS_3MF_Exporter writes (bbs_3mf.cpp:8462-8510).
// Usage: node tools/chain-probe/plate-custom-gcodes-fixtures.mjs
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { crc32, deflateRawSync, inflateRawSync } from "node:zlib";

const SOURCE = "tests/parity/orca_cli_ender3.3mf";
const ENTRY = "Metadata/custom_gcode_per_layer.xml";
const OUT = "tests/parity/plate-custom-gcodes";

const layer = (attributes) =>
  `<layer ${Object.entries(attributes).map(([key, value]) => `${key}="${value}"`).join(" ")}/>`;
const typed = (top_z, type, extruder, color = "", extra = "") => layer({ top_z, type, extruder, color, extra });
const plate = (id, ...children) =>
  `<plate>${id === undefined ? "" : `<plate_info id="${id}"/>`}${children.join("")}</plate>`;
const mode = (value) => `<mode value="${value}"/>`;
const document = (root, ...children) =>
  `<?xml version="1.0" encoding="utf-8"?>\n<${root}>${children.join("")}</${root}>`.replace(/></g, ">\n<");

const FIXTURES = [
  {
    // All six types, file order kept (not sorted by z), escaped strings, an unknown element
    // and attribute, a root-level layer that the plate branch ignores, a plate without
    // plate_info (skipped), a repeated id (appends), and two plate_info (the last wins).
    name: "plates",
    xml: document(
      "custom_gcodes_per_layer",
      plate(
        1,
        typed("0.40000000000000002", 0, 1, "#FF0000"),
        typed("1.2", 1, 1, "", "Insert &quot;magnet&quot; &amp; resume&#10;now"),
        typed("0.8", 2, 2, "#00FF00"),
        typed("2.8", 3, 1),
        `<ignored value="1"/>`,
        layer({ top_z: "3.6000000000000001", type: 4, extruder: 1, color: "", extra: "G4 S1&#10;M117 &lt;hi&gt;", unknown: "x" }),
        typed("4.4", 5, 1),
      ),
      plate(2, typed("1", 0, 3, "#0000FF"), mode("SingleExtruder")),
      plate(undefined, typed("9.9", 1, 1, "", "skipped")),
      plate(2, typed("5.5", 4, 1, "", "M118 appended")),
      plate(3, `<plate_info id="4"/>`, typed("0.6", 1, 1, "", "last plate_info")),
      typed("7.7", 1, 1, "", "root level"),
    ),
  },
  {
    // No <plate>: the root's children go to plate 1. The entry name differs in case (iequals).
    name: "no-plate",
    entry: "Metadata/Custom_GCode_Per_Layer.XML",
    xml: document("custom_gcodes_per_layer", typed("0.6", 0, 1, "#123456"), mode("MultiAsSingle"), typed("1.4", 4, 1, "", "M117 root")),
  },
  {
    // PrusaSlicer 2.2.0 and older: no type, the gcode attribute decides; extra is ignored.
    name: "legacy",
    xml: document(
      "custom_gcodes_per_layer",
      plate(
        1,
        layer({ top_z: "0.4", extruder: 1, color: "#ABCDEF", gcode: "M600", extra: "ignored" }),
        layer({ top_z: "0.8", extruder: 1, color: "Pause message", gcode: "M601" }),
        layer({ top_z: "1.2", extruder: 2, color: "", gcode: "tool_change" }),
        layer({ top_z: "1.6", extruder: 1, color: "", gcode: "G28 X" }),
      ),
    ),
  },
  {
    // check_mode_for_custom_gcode_per_print_z (CustomGCode.cpp:41-58) for plates without a
    // mode; an unknown mode string; an explicit mode that is kept; a plate with no codes.
    name: "modes",
    xml: document(
      "custom_gcodes_per_layer",
      plate(1, typed("0.4", 0, 1, "#FF0000")),
      plate(2, typed("0.4", 0, 2, "#FF0000")),
      plate(3, typed("0.4", 0, 2, "#FF0000"), typed("0.8", 2, 1)),
      plate(4, typed("0.4", 2, 1), mode("Bogus")),
      plate(5, typed("0.4", 0, 2, "#FF0000"), mode("MultiAsSingle")),
      plate(6),
      plate(7, typed("0.4", 1, 3, "", "pause only")),
    ),
  },
  { name: "wrong-root", xml: document("custom_gcodes_per_print_z", plate(1, typed("0.4", 0, 1, "#FF0000"))) },
  { name: "empty", xml: "" },
  { name: "missing-attr", xml: document("custom_gcodes_per_layer", plate(1, layer({ top_z: "0.4", type: 0, extruder: 1, extra: "" }))) },
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
for (const { name, entry = ENTRY, xml } of FIXTURES) {
  writeFileSync(`${OUT}/${name}.3mf`, zip([...source, [entry, Buffer.from(xml)]]));
}
