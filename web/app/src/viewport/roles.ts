import { EXTRUSION_ROLES, MOVE_TYPES, type Toolpaths } from "@ares-slicer/wasm";

type Rgb = readonly [number, number, number];

/** libvgcode `DEFAULT_EXTRUSION_ROLES_COLORS` (`ViewerImpl.cpp:283`), indexed like `EXTRUSION_ROLES`. */
export const ROLE_COLORS: readonly Rgb[] = [
  [230, 179, 179], [255, 230, 77], [255, 125, 56], [31, 31, 255], [176, 48, 41],
  [150, 84, 204], [240, 64, 64], [255, 140, 105], [77, 128, 186], [255, 255, 255],
  [0, 135, 110], [0, 255, 0], [0, 128, 0], [179, 227, 171], [94, 209, 148],
  [102, 92, 199], [77, 128, 186], [0, 59, 110], [0, 64, 0], [128, 128, 128],
];

/** libvgcode `DEFAULT_OPTIONS_COLORS` (`ViewerImpl.cpp:307-317`) by `EOptionType` name. */
export const OPTION_COLORS: Record<string, Rgb> = {
  Travels: [56, 72, 155],
  Wipes: [255, 255, 0],
  Retractions: [205, 34, 214],
  Unretractions: [73, 173, 207],
  Seams: [230, 230, 230],
  ToolChanges: [193, 190, 99],
  ColorChanges: [218, 148, 139],
  PausePrints: [82, 240, 131],
  CustomGCodes: [226, 210, 67],
};

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
