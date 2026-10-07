// Bounded-impact proof for the committed tagged normal fixture. Native records
// remain the authority; this checks their identities and accumulation recurrence.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
const directory = process.argv[2];
const [raw, stable] = ["raw", "stable"].map(name => JSON.parse(readFileSync(`${directory}/${name}.json`)).vertices);
const bits = value => new Uint32Array(new Float32Array([value]).buffer)[0];
const value = word => new Float32Array(new Uint32Array([word]).buffer)[0];
const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const failures = [];
const check = (condition, detail) => { if (!condition) failures.push(detail); };
check(raw.length === 7348 && stable.length === 7347, "unexpected cardinality");
const phantom = raw[1653];
check(phantom.gcodeId === 679 && equal(phantom.durationBits, [0, 0]), "extra phantom identity/time");
check(equal(phantom.position, raw[1652].position) && equal(phantom.position, raw[1654].position), "phantom position is not preceding processor endpoint");
const group = vertices => {
  const groups = new Map();
  for (const vertex of vertices) {
    if (!groups.has(vertex.gcodeId)) groups.set(vertex.gcodeId, []);
    groups.get(vertex.gcodeId).push(vertex);
  }
  return groups;
};
const r = group(raw.filter(v => v.id !== phantom.id)), s = group(stable);
check(equal([...r.keys()].sort((a,b)=>a-b), [...s.keys()].sort((a,b)=>a-b)), "G-code identity set");
const direct = [], pairs = [];
for (const [gcodeId, vertices] of s) {
  const matches = r.get(gcodeId) ?? [];
  check(matches.length === vertices.length, `group cardinality ${gcodeId}`);
  vertices.forEach((vertex, occurrence) => {
    const before = matches[occurrence];
    if (!before) return;
    pairs.push({ rawId: before.id, stableId: vertex.id, gcodeId, occurrence });
    for (const field of ["type", "role", "layerId", "extruderId", "colorId", "position", "durationBits"]) {
      if (!equal(before[field], vertex[field])) direct.push({ gcodeId, rawId: before.id, stableId: vertex.id, field, raw: before[field], stable: vertex[field] });
    }
  });
}
// Exactly the stale extrusion, its following path-start, and the intended
// ColorChange marker. This is a fixed source-cited fixture boundary, not an
// inferred/expandable allowlist of whatever differences happen to occur.
const expected = ["679:position", "679:durationBits", "680:position", "790:position", "790:durationBits"];
check(equal(direct.map(d => `${d.gcodeId}:${d.field}`), expected), "direct difference set outside approved boundary");
const rawToStable = new Map(pairs.map(p => [p.rawId, p.stableId]));
const remaining = (vertices, map) => vertices.filter(v => v.durationBits.some(x => x !== 0) && v.gcodeId !== 679 && v.gcodeId !== 790).map(v => map ? map.get(v.id) : v.id);
check(equal(remaining(raw, rawToStable), remaining(stable)), "unaffected nonzero-duration order changed");
// Every native prefix, including the complete tail, must obey the source's
// ordered f32 recurrence. This is diagnostic verification, not generated
// expected timing for Ares or a production JS estimator.
for (const [name, vertices] of [["raw", raw], ["stable", stable]]) {
  let previous = [0, 0];
  for (const vertex of vertices) {
    for (let mode = 0; mode < 2; mode++) {
      check(bits(Math.fround(value(previous[mode]) + value(vertex.durationBits[mode]))) === vertex.elapsedBits[mode], `${name} recurrence ${vertex.id}/${mode}`);
    }
    previous = vertex.elapsedBits;
  }
}
const sample = (rawId, stableId) => ({ rawId, stableId,
  rawElapsed: value(raw[rawId].elapsedBits[0]), stableElapsed: value(stable[stableId].elapsedBits[0]),
  stableMinusRaw: value(stable[stableId].elapsedBits[0]) - value(raw[rawId].elapsedBits[0]),
});
const report = {
  inputBoundary: "GCodeProcessor.cpp:3291-3296,5962-6008; OptionsZCorrector::update in GCodeProcessor.hpp:693-706; derived phantom LibVGCodeWrapper.cpp:215-238",
  counts: [raw.length, stable.length], phantom, direct, correspondence: pairs,
  firstPrefixChange: sample(1449, 1450),
  directDurationsSeconds: direct.filter(d => d.field === "durationBits").map(d => ({ gcodeId:d.gcodeId,raw:value(d.raw[0]),stable:value(d.stable[0]),difference:value(d.stable[0])-value(d.raw[0]) })),
  propagation: [1452, 1652, 1653, 1654, stable.length - 1].map(id => {
    const pair = pairs.find(p => p.stableId === id);
    return { gcodeId: pair.gcodeId, occurrence: pair.occurrence, ...sample(pair.rawId, pair.stableId) };
  }),
  explanation: "raw relocates extrusion679 after marker790, adding a zero-time phantom; stable retains679 in place. Duration0.130... is assigned to679 in raw versus790 in stable; stable additionally retains679's own duration. Elapsed differences follow these bounded duration/order changes through ordered f32 additions; real-valued subtraction need not equal the unrounded duration because each addition rounds to f32.",
  failureCount: failures.length, firstFailure: failures[0] ?? null,
};
writeFileSync(`${directory}/bounded-impact.json`, JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify({ counts:report.counts,directDifferences:direct.length,firstPrefixChange:report.firstPrefixChange,tail:report.propagation.at(-1),failureCount:failures.length,firstFailure:report.firstFailure }));
assert.equal(failures.length, 0, "outside source boundary: see bounded-impact.json; stop, no waiver");
