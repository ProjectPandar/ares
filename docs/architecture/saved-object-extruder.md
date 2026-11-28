# Saved single-part extruder selection on 3MF import

## Native boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer::_load_model_from_file`,
deserializes object config and builds the object's volumes (2272–2280,
2314–2320). It then resolves the saved `extruder` selectors:

- `2376–2385`: the limit is the number of embedded `filament_settings_id`
  entries, or the maximum integer if that option is absent. An absent,
  zero, or above-limit object selector becomes `1`.
- `2386–2389`: for exactly one loaded volume, erase that volume's
  `extruder`. Do not copy the value to the object or reset the volume to `1`.
- `2390–2400`: for multiple volumes, erase a zero volume selector and
  normalize an above-limit volume selector to `1`. Keep in-range selectors.

This policy belongs to model import, not a slice or export action.
`src/libslic3r/Model.cpp::read_from_file` (467–472) and `read_from_archive`
(533–543) send saved 3MF model loads to `load_bbs_3mf`, whose importer calls
`_load_model_from_file` (`bbs_3mf.cpp:1536,9569–9578`). The exception is the GUI
archive reader's recognized PrusaSlicer path (`Model.cpp:533–539`). It
calls `load_3mf` and reads its own `Metadata/Slic3r_PE_model.config`
(`src/libslic3r/Format/3mf.cpp:102,842–849`), not the saved part records of this
importer. That separate path is unchanged.

`src/libslic3r/PrintObject.cpp::region_config_from_model_volume`
(4395–4431) applies the object's config before the model part's config.
Erasing only the volume selector lets the object assignment supply defaults
without removing explicit per-feature overrides. The G-code writer exposes
logical assignment in `; filament:` and marks the initial logical filament
with `;VT` on the single-nozzle path (`src/libslic3r/GCode.cpp:4285–4305,
4433–4444`). Neither annotation is a physical tool change.

## Ares boundary

Apply the one-volume branch in `project/load/assemble` after
`assemble/volumes.rs::collect_volumes` binds `Metadata/model_settings.config`
part records to loaded volumes. Count those volumes, not XML rows, build
instances, or the number of presets. Erase only the typed part `extruder`
on the sole volume. Keep the object selector, other volume options, and raw
saved archive records intact. Defer object default/range normalization
(2376–2385) and multi-volume selector normalization (2390–2400).

`project/load.rs::load_project_from_archive` assembles the model for saved
3MF loads. CLI slicing and export, core API, and Web/WASM project load all
consume that normalized model. The rule does not depend on the CLI action,
preset replacement, or a Bambu/Orca application tag. Do not add a format
detector or change Prusa-specific config parsing. Geometry without a saved
part selector and interactive edits made after import are unchanged.

The shared import step runs before effective region resolution and tool
collection, so `project/effective_config/candidates.rs` and
`project_slice/extruders.rs::collect_project_object_extruders` see the same
normalized model. Keep the existing region precedence and tool ordering.
Do not rewrite G-code annotations, re-read the archive, or keep a second
copy of the typed selector for slicing.

## Plain CLI verification

Generate a synthetic one-part, single-nozzle saved project with two entries
of the same pinned filament preset. Keep geometry and project settings fixed;
vary only the saved object and part selector metadata. Run identical
`--slice 1 --outputdir out saved.3mf` commands with pinned Native and Ares,
using pinned installation resources. Require each to produce the standalone
`plate_1.gcode` successfully. Compare the entire ordered byte stream with the
existing generator-banner-only normalization. Do not exclude annotations,
usage, config, or motion.

| Saved object selector | Saved single-part selector | Native logical slot |
| --- | --- | --- |
| `1` | absent | `1` |
| `2` | absent | `2` |
| absent | `2` | `1` |
| `2` | `1` | `2` |

The first two are unchanged controls. The two conflicting part cases must
match their respective object-only controls in Native and expose an ordered
G-code mismatch in Ares before the fix. All four comparisons must pass in
both reference lanes. A same-material project can expose logical selection
without changing physical tool commands or reported filament usage. Do not
infer distinct-material or multi-nozzle motion parity from this fixture.
Probe artifacts: `plain-slice-premise-v1/driver.log` and each selector case's
`gcode.diff`.

Compare the parent and changed model across the existing saved-project load,
plain slice, sliced and unsliced export parity cases in both reference lanes.
Also pair core project-load tests, WASM loaded-scene and direct-project tests,
and Web project import, plate/session load, and save/reopen tests. Keep their
existing assertions and archive comparators.

## Deferred sliced archive behavior

Keep sliced and unsliced export projection and admission unchanged.
The affine guard in `project/export/saved.rs::supports_build_transform`
continues to check retained raw object and part records, not the normalized
part selector. Do not relax it or round-trip nondefault object metadata
until the selected-plate archive can match Native as a whole. The export
probe found two separate sets of differences:

- **Specific to object slot 2:** the saved object's `extruder`,
  `plate_1.json` `first_extruder`, `slice_info.config` layer filament list and
  filament `used_m`/`used_g`, and `filament_sequence.json` `plate_1.sequence`.
  The object-only plain slice already selects slot 2 in both engines. Fix
  the archive projection from live slicing state, not by copying the input
  selector into output statistics. Artifacts:
  `slot-one-premise-v1/slot-two/diff.json` and
  `premise-v1/slot-two-affine-ares/stderr.log`.
- **Present on the default slot too:** 21 retained vectors in
  `project_settings.config`, the `compatible_process_expression_group` and
  `pellet_flow_coefficient` footer values in `plate_1.gcode`, its consequent
  checksum, `slice_info.config` `limit_filament_maps`, and
  `filament_sequence.json` `optimal_assignment` width. These are separate
  from selector resolution. Reuse the existing
  [retained preset boundary](saved-settings-retained-filaments.md) for
  overlapping fields. Artifacts: `slot-one-premise-v1/slot-one/diff.json`
  and `slot-one-premise-v1/slot-default/diff.json`.

Keep the [sliced archive contract](cli-export-3mf-sliced.md) and its strict
comparisons unchanged.
