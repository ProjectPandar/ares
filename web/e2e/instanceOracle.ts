import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
/** An Ender-3 with a 10 mm cube at (110, 110), identify id 14. */
export const CUBE_FIXTURE = "tests/parity/orca_cli_ender3.3mf";

/**
 * The project the GUI exports with two copies of the cube: at (121, 121) and (132, 132), moved by 5% of
 * the 220 mm bed each (Plater.cpp:19289-19331), on plate 1 with identify ids 15 and 16 (the session's
 * stand-in for upstream's run-time id, one past the largest).
 */
export function cubeWithTwoCopies(): Uint8Array<ArrayBuffer> {
  const entries = unzipSync(readFileSync(`${repositoryRoot}${CUBE_FIXTURE}`));
  const model = strFromU8(entries["3D/3dmodel.model"]);
  entries["3D/3dmodel.model"] = strToU8(model.replace("</build>",
    "<item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 121 121 5\" printable=\"1\"/><item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 132 132 5\" printable=\"1\"/></build>"));
  const settings = strFromU8(entries["Metadata/model_settings.config"]);
  const instance = (id: number, identify: number) =>
    `    <model_instance>\n      <metadata key="object_id" value="2"/>\n      <metadata key="instance_id" value="${id}"/>\n      <metadata key="identify_id" value="${identify}"/>\n    </model_instance>\n`;
  entries["Metadata/model_settings.config"] = strToU8(settings.replace("  </plate>", `${instance(1, 15)}${instance(2, 16)}  </plate>`));
  return zipSync(entries) as Uint8Array<ArrayBuffer>;
}
