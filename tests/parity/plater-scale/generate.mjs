// Test-data preparation only: two-instances.3mf is orca_cli_ender3.3mf with its 10 mm cube as two
// instances of one object, at (90, 110, 5) and (130, 110, 5). Run with `node generate.mjs` from a
// checkout whose web dependencies are installed.
import fs from "node:fs";
import { unzipSync, zipSync, strFromU8, strToU8 } from "../../../web/node_modules/fflate/esm/index.mjs";

const here = (name) => new URL(name, import.meta.url);
const files = unzipSync(fs.readFileSync(here("../orca_cli_ender3.3mf")));

const item = (x) => `<item objectid="2" transform="1 0 0 0 1 0 0 0 1 ${x} 110 5" printable="1" auto_drop="1"/>`;
const model = strFromU8(files["3D/3dmodel.model"]).replace(/<item [^>]*\/>/, `${item(90)}\n  ${item(130)}`);
files["3D/3dmodel.model"] = strToU8(model);

const sibling = [
  "    <model_instance>",
  '      <metadata key="object_id" value="2"/>',
  '      <metadata key="instance_id" value="1"/>',
  '      <metadata key="identify_id" value="15"/>',
  "    </model_instance>",
].join("\n");
const settings = strFromU8(files["Metadata/model_settings.config"]).replace(
  "    </model_instance>\n  </plate>",
  `    </model_instance>\n${sibling}\n  </plate>`,
);
files["Metadata/model_settings.config"] = strToU8(settings);

fs.writeFileSync(here("two-instances.3mf"), zipSync(files, { mtime: "1980-01-01" }));
