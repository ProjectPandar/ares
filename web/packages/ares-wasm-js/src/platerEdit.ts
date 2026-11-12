import type { PlaterEdit } from "./protocol.js";
import type { PlaterSession } from "./wasm/ares_wasm.js";

/** Applies one `platerEdit` request to its session. */
export function applyEdit(session: PlaterSession, edit: PlaterEdit) {
  switch (edit.op) {
    case "translate":
      return session.translate(edit.object, edit.instance, new Float64Array(edit.displacement));
    case "setPosition":
      return session.setPosition(edit.object, edit.instance, edit.axis, edit.value);
    case "rotate":
      return session.rotate(edit.object, edit.instance, new Float64Array(edit.radians));
    case "setAbsoluteRotation":
      return session.setAbsoluteRotation(edit.object, edit.instance, edit.axis, edit.degrees);
    case "setScale":
      return session.setScale(edit.object, edit.instance, JSON.stringify(edit.edit));
    case "setSize":
      return session.setSize(edit.object, edit.instance, JSON.stringify(edit.edit));
    case "scale":
      return session.scale(edit.object, edit.instance, new Float64Array(edit.factor));
    case "mirror":
      return session.mirror(edit.object, edit.instance, edit.axis);
    case "scaleToFit":
      return session.scaleToFit(edit.object, edit.instance);
    case "addPlate":
      return session.addPlate();
    case "deletePlate":
      return session.deletePlate(edit.index);
    case "lockPlate":
      return session.lockPlate(edit.index, edit.locked);
    case "selectPlate":
      return session.selectPlate(edit.index);
    case "increaseInstances":
      return session.increaseInstances(edit.object, edit.num);
    case "decreaseInstances":
      return session.decreaseInstances(edit.object, edit.num);
    case "deleteObject":
      return session.deleteObject(edit.object);
    case "deleteInstance":
      return session.deleteInstance(edit.object, edit.instance);
    case "deleteAllObjects":
      return session.deleteAllObjects();
    case "deletePlateObjects":
      return session.deletePlateObjects();
    case "setNumberOfCopies":
      return session.setNumberOfCopies(edit.object, edit.num);
    case "arrange":
      return session.arrange(edit.plate, JSON.stringify(edit.settings));
  }
}
