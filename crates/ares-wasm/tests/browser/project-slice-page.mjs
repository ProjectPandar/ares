import init, * as bindings from "/target/wasm-browser/ares_wasm.js";
import {
  SMALL_OPTION_REPLACEMENTS,
  semanticBytes,
  smallArchiveReplacements,
} from "/task22m-vectors.mjs";

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const magics = ["ARES22L\0", "ARES22M\0"].map((text) => ({
  bytes: encoder.encode(text), text,
}));
const bytesOf = (value) => value instanceof Uint8Array ? value : new Uint8Array(value);
const sameBytes = (left, right) =>
  left.length === right.length && left.every((byte, index) => byte === right[index]);
const hasMagic = (bytes, magic) =>
  magic.every((expected, index) => bytes[index] === expected);

class Reader {
  constructor(input, magic) {
    this.bytes = bytesOf(input);
    this.view = new DataView(this.bytes.buffer, this.bytes.byteOffset, this.bytes.byteLength);
    this.cursor = 0;
    for (const expected of magic) {
      if (this.u8() !== expected) throw new Error("invalid ARES22 checkpoint magic");
    }
  }
  require(length) {
    if (this.cursor + length > this.bytes.length) {
      throw new Error("truncated ARES22 checkpoint stream");
    }
  }
  u8() { this.require(1); return this.view.getUint8(this.cursor++); }
  integer(signed = false) {
    this.require(8);
    const value = signed
      ? this.view.getBigInt64(this.cursor, true)
      : this.view.getBigUint64(this.cursor, true);
    this.cursor += 8;
    if (signed) return value.toString();
    if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
      throw new Error("ARES22 u64 exceeds JavaScript's safe range");
    }
    return Number(value);
  }
  u64() { return this.integer(); }
  i64() { return this.integer(true); }
  list(read) { return Array.from({ length: this.u64() }, read); }
  eof() {
    if (this.cursor !== this.bytes.length) {
      throw new Error("ARES22 checkpoint stream has trailing bytes");
    }
    return this.cursor;
  }
}

const polygon = (reader) => reader.list(() => [reader.i64(), reader.i64()]);
const expolygon = (reader) => ({
  contour: polygon(reader),
  holes: reader.list(() => polygon(reader)),
});
const expolygons = (reader) => reader.list(() => expolygon(reader));
const record = (reader, records, read) => {
  const start = reader.cursor;
  const value = read();
  records.push([start, reader.cursor]);
  return value;
};
const assertDense = (values, key, label) => {
  if (values.some((value, index) => value[key] !== index)) {
    throw new Error(`ARES22 ${label} is not dense`);
  }
};

function parsePostRegions(input, magic) {
  const reader = new Reader(input, magic.bytes);
  const records = { sidecar: [], retained: [] };
  const withLslices = magic.text === "ARES22M\0";
  const objects = reader.list(() => {
    const object = {
      sourceObjectIndex: reader.u64(),
      transformIndex: reader.u64(),
      plannedLayerCount: reader.u64(),
      sidecars: reader.list(() => ({
        occurrenceId: reader.u64(),
        layers: reader.list(() => record(reader, records.sidecar, () => ({
          index: reader.u64(), expolygons: expolygons(reader),
        }))),
      })),
      retainedLayers: reader.list(() => record(reader, records.retained, () => {
        const layer = {
          index: reader.u64(),
          regions: reader.list(() => ({
          id: reader.u64(),
          surfaces: reader.list(() => {
            const type = reader.u8();
            if (type !== 4) throw new Error("ARES22 surface is not Internal");
            return { type, expolygon: expolygon(reader) };
          }),
          })),
        };
        if (withLslices) layer.lslices = expolygons(reader);
        return layer;
      })),
    };
    for (const sidecar of object.sidecars) {
      assertDense(sidecar.layers, "index", "sidecar layer index");
    }
    assertDense(object.retainedLayers, "index", "retained layer index");
    for (const layer of object.retainedLayers) {
      assertDense(layer.regions, "id", "region ID");
    }
    return object;
  });
  const bytesRead = reader.eof();
  return {
    ast: { magic: magic.text, byteLength: reader.bytes.length, bytesRead, objects },
    records,
  };
}

function parseCheckpoint(input) {
  const bytes = bytesOf(input);
  const magic = magics.find((candidate) => hasMagic(bytes, candidate.bytes));
  if (!magic) throw new Error("invalid ARES22 checkpoint magic");
  return parsePostRegions(bytes, magic);
}

async function sha256(input) {
  const digest = await crypto.subtle.digest("SHA-256", bytesOf(input));
  return Array.from(new Uint8Array(digest), (byte) =>
    byte.toString(16).padStart(2, "0")).join("");
}

const allExPolygons = (layers) => layers.flatMap((layer) => layer.expolygons);
const geometryTotals = (values) => ({
  expolygons: values.length,
  holes: values.reduce((sum, value) => sum + value.holes.length, 0),
  points: values.reduce((sum, value) =>
    sum + value.contour.length + value.holes.reduce((count, hole) => count + hole.length, 0), 0),
});

function summarize(ast) {
  const sidecars = ast.objects.flatMap((object) => object.sidecars);
  const sidecarLayers = sidecars.flatMap((sidecar) => sidecar.layers);
  const retainedLayers = ast.objects.flatMap((object) => object.retainedLayers);
  const surfaces = retainedLayers.flatMap((layer) =>
    layer.regions.flatMap((region) => region.surfaces));
  const lslices = retainedLayers.flatMap((layer) => layer.lslices ?? []);
  return {
    objects: ast.objects.map((object) => ({
      sourceObjectIndex: object.sourceObjectIndex,
      transformIndex: object.transformIndex,
      plannedLayerCount: object.plannedLayerCount,
      occurrenceIds: object.sidecars.map((sidecar) => sidecar.occurrenceId),
      sidecarLayerCounts: object.sidecars.map((sidecar) => sidecar.layers.length),
      retainedLayerCount: object.retainedLayers.length,
      regionCounts: object.retainedLayers.map((layer) => layer.regions.length),
    })),
    sidecar: geometryTotals(allExPolygons(sidecarLayers)),
    retained: geometryTotals(surfaces.map((surface) => surface.expolygon)),
    lslices: geometryTotals(lslices),
    allInternal: surfaces.every((surface) => surface.type === 4),
  };
}

async function recordIdentities(bytes, ranges, slots) {
  return Promise.all(slots.map(async (slot) => {
    const range = ranges[slot];
    if (range === undefined) return null;
    const recordBytes = bytes.subarray(range[0], range[1]);
    return { byteLength: recordBytes.length, sha256: await sha256(recordBytes) };
  }));
}

async function inspect(bytes, full, slots = []) {
  const parsed = parseCheckpoint(bytes);
  return {
    byteLength: bytes.length,
    bytesRead: parsed.ast.bytesRead,
    magic: parsed.ast.magic,
    sha256: await sha256(bytes),
    summary: summarize(parsed.ast),
    ast: full ? parsed.ast : undefined,
    sidecarRecords: await recordIdentities(bytes, parsed.records.sidecar, slots),
    retainedRecords: await recordIdentities(bytes, parsed.records.retained, slots),
  };
}

async function fixtureBytes() {
  const response = await fetch("/tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
  return new Uint8Array(await response.arrayBuffer());
}

async function execute(project, full = false, slots = []) {
  const inputFirst = bytesOf(await bindings.task22mBrowserInputOracle(project));
  const inputSecond = bytesOf(await bindings.task22mBrowserInputOracle(project));
  const outputFirst = bytesOf(await bindings.task22mBrowserOracle(project));
  const outputSecond = bytesOf(await bindings.task22mBrowserOracle(project));
  return {
    input: await inspect(inputFirst, full),
    output: await inspect(outputFirst, full, slots),
    inputRepeatable: sameBytes(inputFirst, inputSecond),
    outputRepeatable: sameBytes(outputFirst, outputSecond),
  };
}

async function buildArchive(fixture, enabled) {
  const entries = Object.fromEntries(
    Object.entries(globalThis.fflate.unzipSync(fixture)).filter(([name]) => !name.endsWith("/")),
  );
  const process = decoder.decode(entries["Metadata/project_settings.config"]);
  for (const [name, text] of smallArchiveReplacements(enabled, process)) {
    entries[name] = encoder.encode(text);
  }
  const names = Object.keys(entries).sort();
  const semantic = semanticBytes(entries);
  const mtime = new Date(1980, 0, 1);
  const sorted = Object.fromEntries(names.map((name) => [name, [entries[name], { mtime }]]));
  return {
    bytes: globalThis.fflate.zipSync(sorted, { mtime }),
    entries,
    semantic: { byteLength: semantic.length, sha256: await sha256(semantic) },
  };
}

async function smallResult(fixture, enabled) {
  const archive = await buildArchive(fixture, enabled);
  const repeat = await buildArchive(fixture, enabled);
  return {
    entries: archive.entries,
    result: {
      archive: { byteLength: archive.bytes.length, sha256: await sha256(archive.bytes) },
      semantic: archive.semantic,
      archiveRepeatable: sameBytes(archive.bytes, repeat.bytes),
      ...(await execute(archive.bytes, true)),
    },
  };
}

function onlySwitchChanged(disabled, enabled) {
  const names = Object.keys(disabled);
  if (names.length !== Object.keys(enabled).length) return false;
  const process = "Metadata/project_settings.config";
  if (names.some((name) => name !== process && !sameBytes(disabled[name], enabled[name]))) {
    return false;
  }
  const { disabled: before, enabled: after } = SMALL_OPTION_REPLACEMENTS;
  return decoder.decode(disabled[process]).replace(before, after) === decoder.decode(enabled[process]);
}

export async function startProjectSlicePage() {
  await init();
  window.sliceFixtureProject = async () => {
    try {
      await bindings.sliceProject(await fixtureBytes());
      return { resolved: true };
    } catch (error) {
      return { resolved: false, error: String(error) };
    }
  };
  window.parseTask22Vector = (input) => parseCheckpoint(input).ast;
  window.sha256Text = (text) => sha256(encoder.encode(text));
  window.task22mBindingExports = Object.keys(bindings).sort();
  window.task22mFixtureOracles = async () =>
    execute(await fixtureBytes(), false, [0, 46, 49, 459]);
  window.task22mSmallOracles = async () => {
    const fixture = await fixtureBytes();
    const disabled = await smallResult(fixture, false);
    const enabled = await smallResult(fixture, true);
    return {
      switchOnlyChanged: onlySwitchChanged(disabled.entries, enabled.entries),
      disabled: disabled.result,
      enabled: enabled.result,
    };
  };
  window.aresReady = true;
}
