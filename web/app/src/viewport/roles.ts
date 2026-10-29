import type { Toolpaths } from "@ares-slicer/wasm";

/** Used options, as indices into `Toolpaths.optionTypes` in `EOptionType` order (`ViewerImpl.cpp:1013-1015, 1054-1056`). */
export function usedOptions({ options, optionTypes, vertexCount }: Toolpaths) {
  const used = new Array<boolean>(optionTypes.length).fill(false);
  for (let i = 0; i < vertexCount; i++) if (options[i] < used.length) used[options[i]] = true;
  return used.flatMap((on, id) => (on ? [id] : []));
}
