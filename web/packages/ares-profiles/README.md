# @ares-slicer/profiles

OrcaSlicer's system printer, process and filament presets
(`resources/profiles`) as plain data, with one lazily loaded chunk per vendor.

This package contains data only. Resolving `inherits` chains, checking
whether presets are compatible (`compatible_printers`,
`compatible_printers_condition`) and composing the full config are left to
[`@ares-slicer/wasm`](../ares-wasm-js). Pass it the preset JSON from this
package. The package has no runtime dependencies.

## Usage

```ts
import { loadVendor, vendors } from "@ares-slicer/profiles";

vendors;                      // [{ id: "Afinia", name, version, description }, ...] (3.5 kB gzip)
const { index, files } = await loadVendor("Snapmaker"); // fetches only the Snapmaker chunk
for (const { sub_path } of index.machine_list) files[sub_path].name;
```

| Export | Description |
| --- | --- |
| `vendors` | One summary per vendor index file, sorted by `id`. `id` is the file stem (`BBL` for `BBL.json`). |
| `loadVendor(id)` | Resolves to `{ id, index, files }`. `id` is the requested file stem, `index` is the vendor's `<vendor>.json`. `files` holds every file listed in its `machine_model_list`, `process_list`, `filament_list` and `machine_list`, keyed by `sub_path`. Rejects for an unknown `id`. |
| `ORCA_SLICER_COMMIT` | The OrcaSlicer commit the data was generated from. |

The data matches the files OrcaSlicer reads in
`PresetBundle::load_vendor_configs_from_json` (`PresetBundle.cpp`; list keys
from `Preset.hpp`). Every top-level `*.json` is a vendor index, which
includes `OrcaFilamentLibrary`; other vendors' filaments can inherit from it.
Each JSON value is the same as in the source. Files no vendor index lists
are left out, and so are the cover images and bed models.

Bundlers turn each vendor into its own chunk. All 66 vendors together come to
about 1.2 MB gzip.

## Building

The data is generated, not committed. Generating it needs an OrcaSlicer
checkout, either at the repository's `OrcaSlicer/` or at the path in
`ORCA_SLICER_DIR`:

```sh
npm run build        # dist/ from <OrcaSlicer>/resources/profiles
npm pack             # prepack regenerates dist/ and adds LICENSE
```

`web/e2e/profiles.spec.ts` checks the packed tarball:
- every vendor index and listed file is equal to its OrcaSlicer source;
- the types resolve under `nodenext`;
- a Vite build emits one chunk per vendor plus a small entry;
- a browser loads only the vendor chunk it asks for.

## Release note

The vendor modules now carry `id` (`{ id, index, files }`), and `@ares-slicer/wasm` requires it. Bump the package version when it is next published. This change does not bump it.

## License

AGPL-3.0-or-later. The preset data comes from OrcaSlicer, which is AGPL-3.0.
