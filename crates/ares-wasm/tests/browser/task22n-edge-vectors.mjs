import { PROCESS } from "./task22n-vectors.mjs";

const LEAF = "3D/Objects/task22n_box.model";
const quoted = (key, value) => `"${key}": "${value}"`;
const option = (key, from, to) => [PROCESS, quoted(key, from), quoted(key, to)];

export const RELEASE_ROUNDING = {
  layers: 1,
  setup: [
    [LEAF, 'z="0.4"', 'z="2e-7"', "all"],
    option("layer_height", "0.2", "2e-7"),
    option("initial_layer_print_height", "0.2", "2e-7"),
    [
      PROCESS,
      '"nozzle_diameter": [\r\n\t\t"0.4",\r\n\t\t"0.4"\r\n\t]',
      '"nozzle_diameter": [\r\n\t\t"100",\r\n\t\t"100"\r\n\t]',
    ],
    option("initial_layer_line_width", "0.5", "500%"),
    option("inner_wall_line_width", "0.45", "500%"),
    option("bridge_line_width", "100%", "0"),
    option("bridge_flow", "1", "2.2250738585072014e-308"),
  ],
};
