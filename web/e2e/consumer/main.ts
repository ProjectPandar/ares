// A third-party app using the packed @ares-slicer/wasm tarball exactly as its README shows.
import { AresSlicer } from "@ares-slicer/wasm";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";

const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });
const input = document.querySelector("input")!;
const output = document.querySelector("output")!;

input.addEventListener("change", async () => {
  try {
    const bytes = new Uint8Array(await input.files![0].arrayBuffer());
    const scene = await slicer.loadScene(bytes);
    const catalogJson = await (await fetch("/system-presets.json")).text();
    const { gcode } = await slicer.sliceProject(bytes, undefined, {}, { catalogJson });
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", gcode));
    const sha256 = Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join("");
    output.textContent = JSON.stringify({ objects: scene.objects.length, bytes: gcode.length, sha256 });
  } catch (error) {
    output.textContent = JSON.stringify({ error: String(error) });
  }
});
