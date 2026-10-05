// Writes the `.orca_printer` fixture laid out as OrcaSlicer exports one
// (ExportConfigsDialog::archive_preset_bundle_to_file, CreatePresetsDialog.cpp:3931-4040):
// printer/filament/process JSON members plus bundle_structure.json. User presets
// inherit from the BBL X1C system presets. Members are deflated with node:zlib; no zip tool is needed.
// Usage: node tests/presets/make-orca-printer.mjs "tests/presets/My X1C 0.4 nozzle.orca_printer"
import { writeFileSync } from "node:fs";
import { crc32, deflateRawSync } from "node:zlib";
const printer = "My X1C 0.4 nozzle";
const process_ = "0.20mm Standard @BBL X1C - Mine";
const filament = "Bambu PLA Basic @BBL X1C - Mine";
const files = {
  [`printer/${printer}.json`]: { from: "User", inherits: "Bambu Lab X1 Carbon 0.4 nozzle", is_custom_defined: "0", name: printer, printable_height: "240", printer_settings_id: printer, version: "2.4.0.2" },
  [`filament/${filament}.json`]: { compatible_printers: [printer], filament_settings_id: [filament], from: "User", inherits: "Bambu PLA Basic @BBL X1C", is_custom_defined: "0", name: filament, nozzle_temperature: ["215"], version: "2.4.0.2" },
  [`process/${process_}.json`]: { from: "User", inherits: "0.20mm Standard @BBL X1C", is_custom_defined: "0", name: process_, print_settings_id: process_, sparse_infill_density: "20%", version: "2.4.0.2" },
};
const structure = { bundle_id: `offline_${printer}_20261005163000`, bundle_type: "printer config bundle", filament_config: [`filament/${filament}.json`], printer_config: [`printer/${printer}.json`], printer_preset_name: printer, process_config: [`process/${process_}.json`], version: "" };
const entries = [...Object.entries(files).map(([n, j]) => [n, JSON.stringify(j, null, 4) + "\n"]), ["bundle_structure.json", JSON.stringify(structure)]];
const locals = [], centrals = []; let offset = 0;
for (const [name, text] of entries) {
  const data = Buffer.from(text), packed = deflateRawSync(data), nameBuf = Buffer.from(name), crc = crc32(data);
  const h = Buffer.alloc(30); h.writeUInt32LE(0x04034b50, 0); h.writeUInt16LE(20, 4); h.writeUInt16LE(0x800, 6); h.writeUInt16LE(8, 8);
  h.writeUInt32LE(crc, 14); h.writeUInt32LE(packed.length, 18); h.writeUInt32LE(data.length, 22); h.writeUInt16LE(nameBuf.length, 26);
  const c = Buffer.alloc(46); c.writeUInt32LE(0x02014b50, 0); c.writeUInt16LE(20, 4); c.writeUInt16LE(20, 6); c.writeUInt16LE(0x800, 8); c.writeUInt16LE(8, 10);
  c.writeUInt32LE(crc, 16); c.writeUInt32LE(packed.length, 20); c.writeUInt32LE(data.length, 24); c.writeUInt16LE(nameBuf.length, 28); c.writeUInt32LE(offset, 42);
  locals.push(h, nameBuf, packed); centrals.push(c, nameBuf); offset += 30 + nameBuf.length + packed.length;
}
const central = Buffer.concat(centrals), end = Buffer.alloc(22);
end.writeUInt32LE(0x06054b50, 0); end.writeUInt16LE(entries.length, 8); end.writeUInt16LE(entries.length, 10); end.writeUInt32LE(central.length, 12); end.writeUInt32LE(offset, 16);
writeFileSync(process.argv[2], Buffer.concat([...locals, central, end]));
