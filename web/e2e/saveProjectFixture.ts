import { readFileSync } from "node:fs";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { CUBE_FIXTURE } from "./instanceOracle";
import { aboutZ, matrix } from "./platerOracle";

const PRESET_FIXTURE = "tests/parity/bbl-eager-enforce/all-surfaces.3mf";
const modelPath = "3D/3dmodel.model";
const settingsPath = "Metadata/model_settings.config";
export const projectConfig = "Metadata/project_settings.config";

/** A hand-authored scene and preset edit, independent of the project's save API. */
export function saveProjectFixture({
  config,
  edited = false,
  modifier = false,
  singleObject = false,
}: { config?: string; edited?: boolean; modifier?: boolean; singleObject?: boolean } = {}): Buffer {
  const input = unzipSync(readFileSync(new URL(`../../${CUBE_FIXTURE}`, import.meta.url)));
  const sourceModel = "3D/Objects/cube10.stl_1.model";
  const metadata = (key: string, value: string | number) => `<metadata key="${key}" value="${value}"/>`;
  const transform = (x: number, y: number, degrees = 0) =>
    matrix([aboutZ(degrees)], [x, y, 5])
      .filter((_, at) => at % 4 !== 3)
      .join(" ");
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
  if (!singleObject) input[second] = strToU8(mesh.replace(/<object id="1"/, '<object id="3"'));
  delete input[sourceModel];
  input["_rels/.rels"] = strToU8(`<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/${modelPath}" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>`);
  input["3D/_rels/3dmodel.model.rels"] = strToU8(
    strFromU8(input["3D/_rels/3dmodel.model.rels"]).replace(
      /<Relationship [^>]*\/>/,
      (relation) =>
        relation.replace(sourceModel, first) +
        (singleObject ? "" : relation.replace(sourceModel, second).replace("rel-1", "rel-2")),
    ),
  );
  input[modelPath] = strToU8(
    strFromU8(input[modelPath])
      .replace(/<metadata name="Thumbnail_(Middle|Small)">[^<]*<\/metadata>[\t ]*\n?/g, "")
      .replace(/<metadata name="OrcaSlicer">[^<]+<\/metadata>/, '<metadata name="OrcaSlicer">2.5.0</metadata>')
      .replace(
        /<resources>[\s\S]*?<\/resources>/,
        `<resources>${object(2, 1, first)}${singleObject ? "" : object(4, 3, second)}</resources>`,
      )
      .replace(
        /<build[^>]*>[\s\S]*?<\/build>/,
        `<build>${item(2, edited ? transform(145.5, 110, 60) : transform(110, 110))}${singleObject ? "" : item(4, transform(417.2, 110)) + item(4, transform(428.2, 121))}</build>`,
      ),
  );
  input[settingsPath] = strToU8(
    `<config>${settings(2, 1, "Bracket", modifier ? "negative_part" : "normal_part")}${singleObject ? "" : settings(4, 3, "Shim", "normal_part")}${plate(1, "Bracket plate", instance(2, 0, 14))}${singleObject ? "" : plate(2, "Shim plate", instance(4, 0, 15) + instance(4, 1, 16))}</config>`,
  );
  const preset = unzipSync(readFileSync(new URL(`../../${PRESET_FIXTURE}`, import.meta.url)));
  const base = JSON.parse(strFromU8(preset[projectConfig]));
  base.print_settings_id = "0.20mm Standard @BBL X1C";
  base.inherits_group = ["", "", ""];
  base.wipe_tower_x = Array(singleObject ? 1 : 2).fill(base.wipe_tower_x[0]);
  base.wipe_tower_y = Array(singleObject ? 1 : 2).fill(base.wipe_tower_y[0]);
  input[projectConfig] = strToU8(config ?? JSON.stringify(base));
  input["Metadata/filament_sequence.json"] = strToU8(
    JSON.stringify({
      plate_1: { nozzle_sequence: [], optimal_assignment: [], sequence: [] },
      ...(!singleObject && { plate_2: { nozzle_sequence: [], optimal_assignment: [], sequence: [] } }),
    }),
  );
  return Buffer.from(zipSync(input));
}
