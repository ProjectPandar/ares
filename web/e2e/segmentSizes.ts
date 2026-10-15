import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { expect, type Page } from "@playwright/test";

interface NativeVertex {
  type: number;
  positionBits: number[];
  hwaBits: number[];
}

const EXTRUDE = 10;
const TRAVEL = 8;
const WIPE = 9;

/**
 * Toolpath instance end sizes against the Native source-kernel oracle
 * (`tools/chain-probe/segment-hw.nix`: libvgcode `extract_pos_and_or_hwa`,
 * `ViewerImpl.cpp:954-968`, over the real conversion of the same G-code). Each
 * segment end takes the height/width of its own vertex (`Shaders.hpp:91-118`).
 */
export async function checkSegmentSizes(page: Page, artifacts: string) {
  const out = `${artifacts}segment-hw/`;
  mkdirSync(out, { recursive: true });
  const oracle = process.env.ARES_SEGMENT_HW_BIN;
  if (!oracle) throw new Error("ARES_SEGMENT_HW_BIN is required for the segment heights/widths oracle");
  execFileSync(oracle, ["--process-gcode", `${artifacts}oracle-input.gcode`,
    "--dump-moves", `${out}moves.csv`, "--dump-segment-hw", `${out}native.json`], { stdio: "ignore" });
  expect(readFileSync(`${out}moves.csv`)).toEqual(readFileSync(`${artifacts}oracle-moves.csv`));
  const vertices = (JSON.parse(readFileSync(`${out}native.json`, "utf8")) as { vertices: NativeVertex[] }).vertices;

  // Instances as the web mesh builds them: every moved extrude / travel / wipe
  // vertex with its predecessor; f32 bits [start h, start w, end h, end w].
  const ends: number[] = [];
  for (let i = 1; i < vertices.length; i++) {
    const { type, positionBits: b } = vertices[i];
    if (type !== EXTRUDE && type !== TRAVEL && type !== WIPE) continue;
    const a = vertices[i - 1].positionBits;
    if (a[0] === b[0] && a[1] === b[1] && a[2] === b[2]) continue;
    ends.push(...vertices[i - 1].hwaBits.slice(0, 2), ...vertices[i].hwaBits.slice(0, 2));
  }
  let hash = 0x811c9dc5;
  for (const byte of new Uint8Array(new Uint32Array(ends).buffer)) hash = Math.imul(hash ^ byte, 0x01000193);
  const expected = `${ends.length / 4}:${(hash >>> 0).toString(16).padStart(8, "0")}`;
  writeFileSync(`${out}expected.json`, `${JSON.stringify({ probe: expected, ends }, null, 0)}\n`);
  await expect(page.getByTestId("viewport"), "instance end sizes (count:fnv32 of f32 bits)").toHaveAttribute("data-segment-sizes", expected);
  return expected;
}
