// Browser-WASM oracle comparator. Same bounds as gcode_moves_oracle.rs.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";

const floatColumns = [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const floatBounds = [1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 2];
const roleMap = [0, 1, 2, 3, 4, 5, 6, 15, 7, 8, 16, 9, 10, 17, 11, 12, 18, 13, 14, 19];
const bits = new DataView(new ArrayBuffer(4));
function ordinal(value) {
  assert(Number.isFinite(value));
  bits.setFloat32(0, value, true);
  const n = bits.getInt32(0, true);
  return n < 0 ? -2147483648 - n : n;
}
function load(root, name, Type) {
  const b = readFileSync(`${root}/${name}.bin`);
  return new Type(b.buffer.slice(b.byteOffset, b.byteOffset + b.byteLength));
}
export function compareBuffers(root) {
  const rows = readFileSync(`${root}/oracle.csv`, "utf8").trim().split("\n");
  assert.equal(rows.shift(), "gcode_id,type,role,extruder_id,cp_color_id,x,y,z,delta_extruder,feedrate,width,height,mm3_per_mm,travel_dist,fan_speed,temperature,time0,layer_id,internal_only");
  const moves = rows.map(line => line.split(",").map(Number));
  // --dump-move-dynamics (procgcode-dynamics.patch): the same moves' raw acceleration, jerk and actual_feedrate.
  const dynamicsRows = readFileSync(`${root}/dynamics.csv`, "utf8").trim().split("\n");
  assert.equal(dynamicsRows.shift(), "gcode_id,type,internal_only,pressure_advance,acceleration,jerk,actual_feedrate");
  const dynamics = dynamicsRows.map(line => line.split(",").map(Number));
  assert.equal(dynamics.length, moves.length);
  dynamics.forEach((d, i) => assert.deepEqual([d[0], d[1], d[2]], [moves[i][0], moves[i][1], moves[i][18]], `dynamics row ${i}`));
  const values = load(root, "move-values", Float32Array);
  const ids = load(root, "move-ids", Uint32Array);
  const tags = load(root, "move-tags", Uint8Array);
  const vv = load(root, "vertex-values", Float32Array);
  const vi = load(root, "vertex-ids", Uint32Array);
  const vt = load(root, "vertex-tags", Uint8Array);
  assert.equal(values.length, moves.length * 12);
  assert.equal(ids.length, moves.length * 2);
  assert.equal(tags.length, moves.length * 5);
  const diffs = ["kind,index,field,expected,actual,ulp"];
  function number(kind, i, field, expected, actual, bound) {
    const ulp = Math.abs(ordinal(expected) - ordinal(actual));
    if (ulp > bound) diffs.push(`${kind},${i},${field},${expected},${actual},${ulp}`);
  }
  function integer(kind, i, field, expected, actual) {
    if (expected !== actual) diffs.push(`${kind},${i},${field},${expected},${actual},`);
  }
  moves.forEach((r, i) => {
    assert.equal(r.length, 19);
    floatColumns.forEach((col, j) => number("move", i, col, r[col], values[12 * i + j], floatBounds[j]));
    [0, 17].forEach((col, j) => integer("move", i, col, r[col], ids[2 * i + j]));
    [1, 2, 3, 4, 18].forEach((col, j) => integer("move", i, col, r[col], tags[5 * i + j]));
  });
  // LibVGCodeWrapper.cpp:221-325 (2.5.0). Independent oracle-side projection.
  let vertex = 0;
  function checkVertex(curr, dyn, position, actualFeedrate, phantom) {
    assert(16 * vertex < vv.length, `browser has fewer vertices than the projection at gcode_id ${curr[0]}`);
    const expected = [...position, curr[11], curr[10], curr[9], actualFeedrate, curr[12], curr[14], curr[15], phantom ? 0 : curr[16], 0, 0, 0, dyn[4], dyn[5]];
    const bounds = [1, 1, 1, 1, 1, 0, 0, 1, 0, 0, phantom ? 0 : 2, 0, 0, 0, 0, 0];
    expected.forEach((n, j) => number("vertex", vertex, j, n, vv[16 * vertex + j], bounds[j]));
    [curr[0], curr[17]].forEach((n, j) => integer("vertex", vertex, `id${j}`, n, vi[2 * vertex + j]));
    [roleMap[curr[2]], curr[1], curr[3], curr[4]].forEach((n, j) => integer("vertex", vertex, `tag${j}`, n, vt[4 * vertex + j]));
    vertex++;
  }
  for (let i = 1; i < moves.length; i++) {
    const curr = moves[i], prev = moves[i - 1], dyn = dynamics[i], prevDyn = dynamics[i - 1];
    const changed = (a, b) => Math.fround(a) !== Math.fround(b);
    // starts_path (:227-237): type, role, mm3_per_mm, acceleration or jerk changes.
    if ([0, 8, 9, 10].includes(curr[1]) && (i === 1 || prev[1] !== curr[1] || prev[2] !== curr[2] || changed(prev[12], curr[12])
      || changed(prevDyn[4], dyn[4]) || changed(prevDyn[5], dyn[5]))) {
      checkVertex(curr, dyn, prev.slice(5, 8), prevDyn[6], true);
    }
    checkVertex(curr, dyn, curr.slice(5, 8), dyn[6], false);
  }
  assert.equal(vv.length, vertex * 16);
  assert.equal(vi.length, vertex * 2);
  assert.equal(vt.length, vertex * 4);
  writeFileSync(`${root}/diff.csv`, `${diffs.join("\n")}\n`);
  assert.equal(diffs.length, 1, `moves/vertices mismatch: ${root}/diff.csv`);
  return { moves: moves.length, vertices: vertex, mismatches: diffs.length - 1 };
}
