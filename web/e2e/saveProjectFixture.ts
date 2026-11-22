import { readFileSync } from "node:fs";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { CUBE_FIXTURE } from "./instanceOracle";
import { aboutZ, matrix } from "./platerOracle";

const modelPath = "3D/3dmodel.model";
const settingsPath = "Metadata/model_settings.config";
export const projectConfig = "Metadata/project_settings.config";

/** A hand-authored scene and preset edit, independent of the project's save API. */
export function saveProjectFixture({ edited = false, modifier = false } = {}): Buffer {
  const input = unzipSync(readFileSync(new URL(`../../${CUBE_FIXTURE}`, import.meta.url)));
  const sourceModel = "3D/Objects/cube10.stl_1.model";
  const metadata = (key: string, value: string | number) => `<metadata key="${key}" value="${value}"/>`;
  const transform = (x: number, y: number, degrees = 0) => matrix([aboutZ(degrees)], [x, y, 5])
    .filter((_, at) => at % 4 !== 3).join(" ");
  const object = (id: number, part: number, path: string) =>
    `<object id="${id}" type="model"><components><component p:path="/${path}" objectid="${part}"/></components></object>`;
  const item = (id: number, placement: string) => `<item objectid="${id}" transform="${placement}" printable="1"/>`;
  const instance = (id: number, index: number, identify: number) =>
    `<model_instance>${metadata("object_id", id)}${metadata("instance_id", index)}${metadata("identify_id", identify)}</model_instance>`;
  const plate = (id: number, name: string, instances: string) =>
    `<plate>${metadata("plater_id", id)}${metadata("plater_name", name)}${metadata("locked", "false")}${instances}</plate>`;
  const settings = (id: number, part: number, name: string, kind: string) =>
    `<object id="${id}" auto_drop="1">${metadata("name", name)}${metadata("extruder", 1)}<part id="${part}" subtype="${kind}">${metadata("name", name)}</part></object>`;

  const first = "3D/Objects/Bracket_1.model";
  const second = "3D/Objects/Shim_2.model";
  const mesh = strFromU8(input[sourceModel]);
  input[first] = strToU8(mesh.replace(/x="(-?\d+(?:\.\d+)?)"/g, (_, x: string) => `x="${Number(x) * 1.6}"`));
  input[second] = strToU8(mesh.replace(/<object id="1"/, '<object id="3"'));
  delete input[sourceModel];
  input["3D/_rels/3dmodel.model.rels"] = strToU8(strFromU8(input["3D/_rels/3dmodel.model.rels"])
    .replace(/<Relationship [^>]*\/>/, (relation) => relation.replace(sourceModel, first) + relation.replace(sourceModel, second).replace("rel-1", "rel-2")));
  input[modelPath] = strToU8(strFromU8(input[modelPath])
    .replace(/<resources>[\s\S]*?<\/resources>/, `<resources>${object(2, 1, first)}${object(4, 3, second)}</resources>`)
    .replace(/<build[^>]*>[\s\S]*?<\/build>/, `<build>${item(2, edited ? transform(145.5, 110, 90) : transform(110, 110))}${item(4, transform(374, 110))}${item(4, transform(385, 121))}</build>`));
  input[settingsPath] = strToU8(`<config>${settings(2, 1, "Bracket", modifier ? "negative_part" : "normal_part")}${settings(4, 3, "Shim", "normal_part")}${plate(1, "Bracket plate", instance(2, 0, 14))}${plate(2, "Shim plate", instance(4, 0, 15) + instance(4, 1, 16))}</config>`);
  const config = JSON.parse(strFromU8(input[projectConfig]));
  config.wipe_tower_x = [config.wipe_tower_x[0], config.wipe_tower_x[0]];
  config.wipe_tower_y = [config.wipe_tower_y[0], config.wipe_tower_y[0]];
  if (edited) config.layer_height = "0.27";
  input[projectConfig] = strToU8(JSON.stringify(config));
  input["Metadata/filament_sequence.json"] = strToU8(JSON.stringify({
    plate_1: { nozzle_sequence: [], optimal_assignment: [], sequence: [] },
    plate_2: { nozzle_sequence: [], optimal_assignment: [], sequence: [] },
  }));
  return Buffer.from(zipSync(input));
}
