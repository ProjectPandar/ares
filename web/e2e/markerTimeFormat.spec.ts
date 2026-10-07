import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// Numeric display seam: f32 reconstruction, <=1s branch, day/hour/minute
// decomposition and printf ties-to-even. Expected text comes only from C++.
test("marker Time text matches native source-kernel boundary records", async () => {
  const { formatMarkerTime } = await import("../app/src/preview/toolPositionFormat");
  const native = JSON.parse(readFileSync(new URL("../../tests/parity/marker-time/native.json", import.meta.url), "utf8")) as {
    formats: { elapsedBits: number; durationBits: number; text: string }[];
    synthetic: { elapsedBits: number[]; durationBits: number[]; text: string[] }[];
  };
  const view = new DataView(new ArrayBuffer(4));
  const float = (bits: number) => { view.setUint32(0, bits); return view.getFloat32(0); };
  const records = [...native.formats, ...native.synthetic.flatMap((row) => row.text.map((text, mode) => ({
    elapsedBits: row.elapsedBits[mode], durationBits: row.durationBits[mode], text,
  })))];
  for (const record of records) {
    expect(formatMarkerTime({ elapsedSeconds: float(record.elapsedBits), vertexSeconds: float(record.durationBits) }),
      JSON.stringify(record)).toBe(record.text);
  }
});
