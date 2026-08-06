import init, * as bindings from "/target/wasm-browser/ares_wasm.js";
import {
  MODEL, PROCESS, replaceUnique, syntheticEntries,
} from "/task22n-vectors.mjs";

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const bytesOf = (value) => value instanceof Uint8Array ? value : new Uint8Array(value);
const sameBytes = (left, right) =>
  left.length === right.length && left.every((byte, index) => byte === right[index]);
const sameTree = (left, right) => JSON.stringify(left) === JSON.stringify(right);
const quoted = (key, value) => `"${key}": "${value}"`;

class Reader {
  constructor(input) {
    this.bytes = bytesOf(input);
    this.view = new DataView(this.bytes.buffer, this.bytes.byteOffset, this.bytes.byteLength);
    this.cursor = 0;
  }
  require(length) {
    if (!Number.isSafeInteger(length) || length < 0 || this.cursor + length > this.bytes.length) {
      throw new Error("truncated ARES22 checkpoint stream");
    }
  }
  magic(text) {
    for (const expected of encoder.encode(text)) {
      if (this.u8() !== expected) throw new Error("invalid ARES22 checkpoint magic");
    }
  }
  u8() { this.require(1); return this.view.getUint8(this.cursor++); }
  u32() { this.require(4); const value = this.view.getUint32(this.cursor, true); this.cursor += 4; return value; }
  integer(signed = false) {
    this.require(8);
    const value = signed ? this.view.getBigInt64(this.cursor, true) : this.view.getBigUint64(this.cursor, true);
    this.cursor += 8;
    if (signed) return value.toString();
    if (value > BigInt(Number.MAX_SAFE_INTEGER)) throw new Error("ARES22 u64 exceeds safe range");
    return Number(value);
  }
  u64() { return this.integer(); }
  i64() { return this.integer(true); }
  bits64() {
    this.require(8);
    const value = this.view.getBigUint64(this.cursor, true).toString(16).padStart(16, "0");
    this.cursor += 8;
    return value;
  }
  boolean() {
    const value = this.u8();
    if (value > 1) throw new Error("noncanonical ARES22 boolean");
    return value === 1;
  }
  optional(read) { return this.boolean() ? read() : null; }
  list(read) {
    const count = this.u64();
    if (count > this.bytes.length - this.cursor) throw new Error("impossible ARES22 count");
    return Array.from({ length: count }, read);
  }
  take(length) {
    this.require(length);
    const value = this.bytes.subarray(this.cursor, this.cursor + length);
    this.cursor += length;
    return value;
  }
  eof() {
    if (this.cursor !== this.bytes.length) throw new Error("ARES22 checkpoint stream has trailing bytes");
    return this.cursor;
  }
}

const polygon = (reader) => reader.list(() => [reader.i64(), reader.i64()]);
const expolygon = (reader) => ({ contour: polygon(reader), holes: reader.list(() => polygon(reader)) });
const expolygons = (reader) => reader.list(() => expolygon(reader));
const surfaces = (reader) => reader.list(() => {
  const kind = reader.u8();
  if (kind !== 4) throw new Error("invalid ARES22 surface enum");
  return { kind, expolygon: expolygon(reader) };
});
const dense = (values, key, label) => {
  if (values.some((value, index) => value[key] !== index)) throw new Error(`non-dense ${label}`);
};

function parseM(input) {
  const reader = new Reader(input);
  reader.magic("ARES22M\0");
  const objects = reader.list(() => {
    const object = {
      sourceObjectIndex: reader.u64(), transformIndex: reader.u64(), plannedLayerCount: reader.u64(),
      sidecars: reader.list(() => ({
        occurrenceId: reader.u64(),
        layers: reader.list(() => ({ index: reader.u64(), expolygons: expolygons(reader) })),
      })),
      retainedLayers: reader.list(() => ({
        index: reader.u64(),
        regions: reader.list(() => ({ id: reader.u64(), surfaces: surfaces(reader) })),
        lslices: expolygons(reader),
      })),
    };
    for (const sidecar of object.sidecars) dense(sidecar.layers, "index", "sidecar layers");
    dense(object.retainedLayers, "index", "retained layers");
    for (const layer of object.retainedLayers) dense(layer.regions, "id", "regions");
    return object;
  });
  return { magic: "ARES22M\0", byteLength: reader.bytes.length, bytesRead: reader.eof(), objects };
}

const flow = (reader) => ({
  fields: [reader.u32(), reader.u32(), reader.u32(), reader.u32()],
  bridge: reader.boolean(), mm3: reader.bits64(),
});
const record = (reader) => ({
  identity: [reader.u64(), reader.u64(), reader.u64(), reader.u64(), reader.u64()],
  compatible: reader.list(() => reader.u64()),
  current: [reader.u64(), reader.u64()],
  lower: reader.optional(() => reader.u64()),
  upper: reader.optional(() => reader.u64()),
  upperSame: reader.optional(() => [reader.u64(), reader.u64()]),
  currentSurfaces: surfaces(reader),
  lowerSlices: null, upperSlices: null, upperSameSurfaces: null,
});

function finishRecord(reader, value) {
  if (value.lower !== null) value.lowerSlices = expolygons(reader);
  if (value.upper !== null) value.upperSlices = expolygons(reader);
  if (value.upperSame !== null) value.upperSameSurfaces = surfaces(reader);
  Object.assign(value, {
    height: reader.bits64(), sliceZ: reader.bits64(),
    flows: Array.from({ length: 4 }, () => flow(reader)),
    spiral: reader.boolean(), rotation: reader.bits64(), dispatch: reader.u8(),
  });
  if (value.dispatch > 1) throw new Error("invalid ARES22 dispatch enum");
  return value;
}

function validateN(predecessor, objects) {
  if (predecessor.objects.length !== objects.length) throw new Error("N/M object mismatch");
  predecessor.objects.forEach((before, objectIndex) => {
    const object = objects[objectIndex];
    if (!sameTree([before.sourceObjectIndex, before.transformIndex, before.plannedLayerCount],
      [object.source, object.transform, object.planned])
      || object.slots.length !== before.retainedLayers.length || object.slots.length !== object.planned) {
      throw new Error("N/M object header mismatch");
    }
    before.retainedLayers.forEach((layer, index) => {
      if (layer.regions.length !== 1) throw new Error("N/M region cardinality mismatch");
      const region = layer.regions[0];
      const value = object.slots[index];
      if (value === null) {
        if (region.surfaces.length !== 0) throw new Error("N missing nonempty slot");
        return;
      }
      const lower = index === 0 ? null : index - 1;
      const upper = index + 1 === object.slots.length ? null : index + 1;
      const expected = {
        identity: [object.source, object.transform, index, layer.index, region.id],
        compatible: [region.id], current: [0, index], lower, upper,
        upperSame: upper === null ? null : [0, upper], currentSurfaces: region.surfaces,
        lowerSlices: lower === null ? null : before.retainedLayers[lower].lslices,
        upperSlices: upper === null ? null : before.retainedLayers[upper].lslices,
        upperSameSurfaces: upper === null ? null : before.retainedLayers[upper].regions[0].surfaces,
      };
      for (const key of Object.keys(expected)) {
        if (!sameTree(value[key], expected[key])) throw new Error(`N/M record mismatch: ${key}`);
      }
    });
  });
}

function parseN(input) {
  const reader = new Reader(input);
  reader.magic("ARES22N\0");
  const predecessorLength = reader.u64();
  const predecessorBytes = reader.take(predecessorLength);
  const predecessor = parseM(predecessorBytes);
  const objects = reader.list(() => ({
    source: reader.u64(), transform: reader.u64(), planned: reader.u64(),
    slots: reader.list(() => reader.boolean() ? finishRecord(reader, record(reader)) : null),
  }));
  const frame = {
    magic: "ARES22N\0", byteLength: reader.bytes.length, bytesRead: reader.eof(),
    predecessorLength, predecessor, objects,
  };
  validateN(predecessor, objects);
  return { frame, predecessorBytes };
}

async function sha256(input) {
  const digest = await crypto.subtle.digest("SHA-256", bytesOf(input));
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

async function fixtureBytes() {
  const response = await fetch("/tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
  return new Uint8Array(await response.arrayBuffer());
}

const viewRecord = (value) => value === null ? null : ({
  identity: value.identity, compatible: value.compatible, current: value.current,
  lower: value.lower, upper: value.upper, upperSame: value.upperSame,
  geometry: [value.currentSurfaces.length, value.lowerSlices?.length ?? 0,
    value.upperSlices?.length ?? 0, value.upperSameSurfaces?.length ?? 0],
  height: value.height, sliceZ: value.sliceZ,
  flows: value.flows.map((item) => [item.fields, item.bridge, item.mm3]),
  spiral: value.spiral, rotation: value.rotation, dispatch: value.dispatch,
});

async function inspectOracle(m, n, slots) {
  const parsed = parseN(n);
  const selected = slots ?? parsed.frame.objects[0].slots.map((_, index) => index);
  return {
    input: { magic: "ARES22M\0", byteLength: m.length,
      bytesRead: parsed.frame.predecessor.bytesRead, sha256: await sha256(m) },
    output: { magic: "ARES22N\0", byteLength: n.length,
      bytesRead: parsed.frame.bytesRead, sha256: await sha256(n),
      predecessorLength: parsed.frame.predecessorLength },
    embedsInput: sameBytes(m, parsed.predecessorBytes),
    objects: parsed.frame.objects.map((object) => ({
      source: object.source, transform: object.transform, planned: object.planned,
      slotCount: object.slots.length, populated: object.slots.filter(Boolean).length,
    })),
    records: selected.map((index) => viewRecord(parsed.frame.objects[0].slots[index])),
  };
}

async function execute(project, slots) {
  const m1 = bytesOf(await bindings.task22nBrowserInputOracle(project));
  const m2 = bytesOf(await bindings.task22nBrowserInputOracle(project));
  const n1 = bytesOf(await bindings.task22nBrowserOracle(project));
  const n2 = bytesOf(await bindings.task22nBrowserOracle(project));
  return {
    raw: { m: m1, n: n1 },
    result: { ...(await inspectOracle(m1, n1, slots)),
      inputRepeatable: sameBytes(m1, m2), outputRepeatable: sameBytes(n1, n2) },
  };
}

const applyEdits = (entries, edits) => {
  for (const [path, before, after, mode] of edits) {
    const text = decoder.decode(entries[path]);
    const replaced = mode === "all" ? text.replaceAll(before, after) :
      replaceUnique(text, before, after);
    if (mode === "all" && replaced === text) throw new Error(`expected at least one ${before}`);
    entries[path] = encoder.encode(replaced);
  }
};

function archiveEntries(fixture, pair, after) {
  const entries = Object.fromEntries(
    Object.entries(globalThis.fflate.unzipSync(fixture)).filter(([name]) => !name.endsWith("/")),
  );
  const process = decoder.decode(entries[PROCESS]);
  for (const [name, text] of syntheticEntries(process, pair.layers === 3)) {
    entries[name] = encoder.encode(text);
  }
  applyEdits(entries, pair.setup);
  if (after) applyEdits(entries, [pair.delta]);
  return entries;
}

const zipEntries = (entries) => {
  const mtime = new Date(1980, 0, 1);
  return globalThis.fflate.zipSync(Object.fromEntries(Object.keys(entries).sort()
    .map((name) => [name, [entries[name], { mtime }]])), { mtime });
};

async function optionPair(fixture, pair) {
  const beforeEntries = archiveEntries(fixture, pair, false);
  const afterEntries = archiveEntries(fixture, pair, true);
  const changedEntries = Object.keys(beforeEntries).filter(
    (name) => !sameBytes(beforeEntries[name], afterEntries[name]),
  );
  const [path, from, to] = pair.delta;
  const replacementExact = decoder.decode(afterEntries[path]) ===
    replaceUnique(decoder.decode(beforeEntries[path]), from, to);
  const before = await execute(zipEntries(beforeEntries));
  const after = await execute(zipEntries(afterEntries));
  return {
    changedEntries, replacementExact, before: before.result, after: after.result,
    mEqual: sameBytes(before.raw.m, after.raw.m), nEqual: sameBytes(before.raw.n, after.raw.n),
  };
}

const status = (call) => new Promise((resolve) => {
  let settled = false;
  const finish = (value) => {
    if (settled) return;
    settled = true;
    window.removeEventListener("error", onError);
    window.removeEventListener("unhandledrejection", onRejection);
    resolve(value);
  };
  const failed = (error) => finish({ resolved: false, error: String(error) });
  const onError = (event) => { event.preventDefault(); failed(event.error ?? event.message); };
  const onRejection = (event) => { event.preventDefault(); failed(event.reason); };
  window.addEventListener("error", onError, { once: true });
  window.addEventListener("unhandledrejection", onRejection, { once: true });
  Promise.resolve().then(call).then(() => finish({ resolved: true }), failed);
});

const capture = async (call) => {
  try { return { resolved: true, value: await call() }; }
  catch (error) { return { resolved: false, error: String(error) }; }
};

async function increaseElse(fixture, definition) {
  const beforeEntries = archiveEntries(fixture, definition, false);
  const afterEntries = archiveEntries(fixture, definition, true);
  const [path, from, to] = definition.delta;
  const beforeProject = zipEntries(beforeEntries);
  const afterProject = zipEntries(afterEntries);
  const beforeM = bytesOf(await bindings.task22nBrowserInputOracle(beforeProject));
  const afterM = bytesOf(await bindings.task22nBrowserInputOracle(afterProject));
  return {
    changedEntries: Object.keys(beforeEntries).filter(
      (name) => !sameBytes(beforeEntries[name], afterEntries[name])),
    replacementExact: decoder.decode(afterEntries[path]) ===
      replaceUnique(decoder.decode(beforeEntries[path]), from, to),
    mEqual: sameBytes(beforeM, afterM),
    oracle: await capture(async () => inspectOracle(
      afterM, bytesOf(await bindings.task22nBrowserOracle(afterProject)),
    )),
    public: await status(() => bindings.sliceProject(afterProject)),
  };
}

async function tinyBridgeFlows(fixture, definitions) {
  const results = [];
  for (const definition of definitions) {
    const project = zipEntries(archiveEntries(fixture, definition, false));
    results.push({
      name: definition.name,
      input: await status(() => bindings.task22nBrowserInputOracle(project)),
      oracle: await status(() => bindings.task22nBrowserOracle(project)),
      public: await status(() => bindings.sliceProject(project)),
    });
  }
  return results;
}

const releaseSettings = (entries) => {
  const options = JSON.parse(decoder.decode(entries[PROCESS]));
  return {
    nozzleDiameter: options.nozzle_diameter,
    initialLayerLineWidth: options.initial_layer_line_width,
    innerWallLineWidth: options.inner_wall_line_width,
    layerHeight: options.layer_height,
    initialLayerPrintHeight: options.initial_layer_print_height,
    bridgeLineWidth: options.bridge_line_width,
    thickBridges: options.thick_bridges,
    bridgeFlow: options.bridge_flow,
  };
};

async function releaseRounding(fixture, definition) {
  const entries = archiveEntries(fixture, definition, false);
  const project = zipEntries(entries);
  const original = project.slice();
  const beforeM = bytesOf(await bindings.task22nBrowserInputOracle(project));
  const oracle = await status(() => bindings.task22nBrowserOracle(project));
  const publicResult = await status(() => bindings.sliceProject(project));
  const afterM = bytesOf(await bindings.task22nBrowserInputOracle(project));
  return {
    settings: releaseSettings(entries), archiveUnchanged: sameBytes(project, original),
    inputRepeatable: sameBytes(beforeM, afterM), oracle, public: publicResult,
  };
}

async function horizontalShellPropagationMatrix(fixture) {
  const vectors = [
    ["ensure-all-after-promotion", [[PROCESS, quoted("extra_solid_infills", ""),
      quoted("extra_solid_infills", "1#")]]],
    ["moderate-active", [[PROCESS,
      quoted("ensure_vertical_shell_thickness", "ensure_all"),
      quoted("ensure_vertical_shell_thickness", "ensure_moderate")]]],
  ];
  const results = [];
  for (const [name, edits] of vectors) {
    const entries = Object.fromEntries(
      Object.entries(globalThis.fflate.unzipSync(fixture)).filter(([path]) => !path.endsWith("/")),
    );
    applyEdits(entries, edits);
    const project = zipEntries(entries);
    results.push({
      name,
      first: await status(() => bindings.sliceProject(project)),
      second: await status(() => bindings.sliceProject(project)),
    });
  }
  return results;
}

async function extraSolidBoundaryMatrix(fixture) {
  const stl = encoder.encode("solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square");
  const vectors = [
    ["max", "2147483647", true],
    ["near-range", "2147483646#2,2147483647", true],
    ["oversized", "2147483648", false],
  ];
  const results = [];
  for (const [name, pattern, valid] of vectors) {
    const entries = Object.fromEntries(
      Object.entries(globalThis.fflate.unzipSync(fixture)).filter(([path]) => !path.endsWith("/")),
    );
    applyEdits(entries, [[PROCESS, quoted("extra_solid_infills", ""),
      quoted("extra_solid_infills", pattern)]]);
    results.push({
      name, valid,
      json: await status(() => bindings.slice_stl(
        stl, JSON.stringify({ extra_solid_infills: pattern }),
      )),
      raw: await status(() => bindings.sliceProject(zipEntries(entries))),
    });
  }
  return results;
}

export async function startProjectSlicePage() {
  await init();
  window.parseTask22nVector = (input) => parseN(input).frame;
  window.sha256Text = (text) => sha256(encoder.encode(text));
  window.sliceFixtureProject = async () => {
    try { await bindings.sliceProject(await fixtureBytes()); return { resolved: true }; }
    catch (error) { return { resolved: false, error: String(error) }; }
  };
  window.task22nBindingExports = Object.keys(bindings).sort();
  window.task22nFixtureOracles = async () => (await execute(
    await fixtureBytes(), [0, 1, 229, 459],
  )).result;
  window.task22nOptionMatrix = async (pairs) => {
    const fixture = await fixtureBytes();
    const results = [];
    for (const pair of pairs) results.push(await optionPair(fixture, pair));
    return results;
  };
  window.task22nIncreaseElse = async (definition) =>
    increaseElse(await fixtureBytes(), definition);
  window.task22nTinyBridgeFlows = async (definitions) =>
    tinyBridgeFlows(await fixtureBytes(), definitions);
  window.task22nReleaseRounding = async (definition) =>
    releaseRounding(await fixtureBytes(), definition);
  window.task22o25ExtraSolidBoundaries = async () =>
    extraSolidBoundaryMatrix(await fixtureBytes());
  window.task22o26HorizontalShellPropagation = async () =>
    horizontalShellPropagationMatrix(await fixtureBytes());
  window.task22nPaths = { MODEL, PROCESS };
  window.aresReady = true;
}
