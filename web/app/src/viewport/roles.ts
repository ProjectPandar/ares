import { EXTRUSION_ROLES, MOVE_TYPES, type Toolpaths } from "@ares-slicer/wasm";

const EXTRUDE = MOVE_TYPES.indexOf("Extrude");

/** Used options, as indices into `Toolpaths.optionTypes` in `EOptionType` order (`ViewerImpl.cpp:1013-1015, 1054-1056`). */
export function usedOptions({ options, optionTypes, vertexCount }: Toolpaths) {
  const used = new Array<boolean>(optionTypes.length).fill(false);
  for (let i = 0; i < vertexCount; i++) if (options[i] < used.length) used[options[i]] = true;
  return used.flatMap((on, id) => (on ? [id] : []));
}

/** Role ids with at least one extrusion, in `EXTRUSION_ROLES` order (legend rows). */
export function usedRoles({ tags, vertexCount }: Toolpaths): number[] {
  const used = new Array<boolean>(EXTRUSION_ROLES.length).fill(false);
  for (let i = 0; i < vertexCount; i++) if (tags[4 * i + 1] === EXTRUDE) used[tags[4 * i]] = true;
  return used.flatMap((present, id) => (present ? [id] : []));
}
