// Writes the Summary (`update_print_estimated_stats`, GCode.cpp:1904-1940)
// fixtures into tests/parity/summary/: existing 3MFs whose
// project_settings.config gets the listed keys replaced. Every other entry
// is copied byte for byte. Usage: node tools/chain-probe/summary-fixtures.mjs
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { crc32, deflateRawSync } from "node:zlib";

const SETTINGS = "Metadata/project_settings.config";
const FIXTURES = [
  {
    // Two used filaments (object-level sparse_infill_filament_id=2), distinct
    // densities and costs, time_cost > 0.
    name: "time-cost",
    source: "tests/parity/validation/multi-filament-bridge-anchor-narrowed.3mf",
    edits: { filament_density: ["1.24", "1.31"], filament_cost: ["24.99", "31.7"], time_cost: "1.7" },
  },
  {
    // Two configured filaments, only filament 1 used by the object.
    name: "unused-filament",
    source: "tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf",
    edits: { filament_density: ["1.24", "1.31"], filament_cost: ["24.99", "31.7"], time_cost: "1.7" },
  },
];

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

mkdirSync("tests/parity/summary", { recursive: true });
for (const { name, source, edits } of FIXTURES) {
  const names = execFileSync("unzip", ["-Z1", source], { encoding: "utf8" }).split("\n").filter(Boolean);
  const entries = names.map((entry) => {
    let data = execFileSync("unzip", ["-p", source, entry.replace(/[[\]*?]/g, "\\$&")], { maxBuffer: 1 << 28 });
    if (entry === SETTINGS) {
      const settings = JSON.parse(data.toString("utf8"));
      for (const [key, value] of Object.entries(edits)) {
        if (!(key in settings)) throw new Error(`${source}: missing ${key}`);
        settings[key] = value;
      }
      data = Buffer.from(JSON.stringify(settings, null, 4) + "\n");
    }
    return [entry, data];
  });
  writeFileSync(`tests/parity/summary/${name}.3mf`, zip(entries));
}
