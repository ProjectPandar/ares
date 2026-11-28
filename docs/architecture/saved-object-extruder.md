# Saved single-part extruder selection in plain slicing

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

`src/libslic3r/PrintObject.cpp::region_config_from_model_volume`
(4395–4431) applies the object's config before the model part's config.
Erasing only the volume selector lets the object assignment supply defaults
without removing explicit per-feature overrides. The G-code writer exposes
logical assignment in `; filament:` and marks the initial logical filament
with `;VT` on the single-nozzle path (`src/libslic3r/GCode.cpp:4285–4305,
4433–4444`). Neither annotation is a physical tool change.

## Ares boundary

Apply the one-volume branch for a saved model part on the plain positional
3MF slice route. Count loaded model volumes, not XML rows, build instances,
meshes after slicing, or the number of presets. Keep the object selector
unchanged. Erase only the typed part `extruder` used for slicing and keep
all other part options and retained archive metadata intact. This slice does
not add object-value or multi-volume selector normalization.

`project/load/assemble/volumes.rs::collect_volumes` binds saved part records
to loaded volumes. `project_slice/cli_plates.rs` loads those objects for the
plain saved CLI request through `ProjectInput::SavedCliBytes`. Scope the
normalization to the loaded plain-slice model before selector validation or
plate application, so `project/effective_config/candidates.rs` and
`project_slice/extruders.rs::collect_project_object_extruders` both see the
same resolved input. Keep the existing region precedence and tool ordering.
Do not rewrite G-code annotations, re-read the archive, or add a separate
source of assignment state. Keep generic import, interactive edits, explicit
preset replacement, STL, and export routes unchanged.

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

## Deferred sliced archive behavior

Do not relax the affine-export guard or round-trip nondefault object metadata
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
