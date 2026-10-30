// The @ares-slicer/wasm PlaterSession client driven from the test: each call is one session
// method, and edits return the session layout (instance matrices and plates).
import { AresSlicer, type PlaterSession } from "@ares-slicer/wasm";
import AresWorker from "@ares-slicer/wasm/worker?worker";
import wasmUrl from "@ares-slicer/wasm/ares_wasm_bg.wasm?url";

const slicer = new AresSlicer({ worker: new AresWorker(), wasmUrl });
let session: PlaterSession;

const plater = {
  async load(bytes: number[], filename: string) {
    session = await slicer.createPlaterSession(new Uint8Array(bytes), filename);
    const { objects, plates } = await session.scene();
    return { objects: objects.map(({ instances }) => ({ instances })), plates };
  },
  translate: (object: number, instance: number, displacement: [number, number, number]) => session.translate(object, instance, displacement),
  setPosition: (object: number, instance: number, axis: 0 | 1 | 2, value: number) => session.setPosition(object, instance, axis, value),
  rotate: (object: number, instance: number, radians: [number, number, number]) => session.rotate(object, instance, radians),
  absoluteRotation: (object: number, instance: number) => session.absoluteRotation(object, instance),
  setAbsoluteRotation: (object: number, instance: number, axis: 0 | 1 | 2, degrees: number) =>
    session.setAbsoluteRotation(object, instance, axis, degrees),
  addPlate: () => session.addPlate(),
  deletePlate: (index: number) => session.deletePlate(index),
  async slice(plate: number) {
    return new TextDecoder().decode((await session.slice(plate)).gcode);
  },
};

Object.assign(window, { plater });
await slicer.ready();
document.querySelector("output")!.textContent = "ready";
