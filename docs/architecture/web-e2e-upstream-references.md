# Web e2e: references for H2D flow options, the U1 flush matrix and estimation

## Scope

Two web e2e specs fail on main because their expected values do not follow the upstream source
they cite, while the app, `ares-wasm` and `ares-core` already follow it. A third is measured
here and changed elsewhere:

- `dualExtruder.spec.ts`: the expected H2D flow options;
- `printerSwitchFilaments.spec.ts`, the U1 slice: the Native reference for the
  `flush_volumes_matrix` line, measured in section 2 and changed by the explicit-selection
  slice;
- `estimation.spec.ts`: the Native reference slices the 3MF on the CLI route, while the web
  opens it on the GUI route.

This document only changes web e2e code. No `libslic3r` or `libvgcode` behaviour, no app code
and no core code changes. Upstream lines are cited at OrcaSlicer 40ecc66f7 (v2.5.0).

Not covered here: the `filament_ids` line of the U1 slice, the U1 flush matrix reference and
the `compatible_printers_condition` line of `presetSlice.spec.ts`. The web slice of an explicit
preset selection follows the GUI's `PresetBundle::full_fff_config`, not the CLI's
`--load-settings` composition. That route change and its reference changes are specified in
`web-explicit-selection-gui-config.md`. Section 2 keeps the
measurement that slice relies on.

## 1. dualExtruder: H2D flow options

### Upstream

`Sidebar::update_presets` fills each nozzle's flow combo through `update_extruder_variant`
(Plater.cpp:4005-4073). The left combo reads printer extruder index 0 and the right combo reads
index 1 (Plater.cpp:4111-4112). For extruder `index`, each `nozzle_volume_type` label is visited
in display order: Standard, High Flow, Hybrid, TPU High Flow, E3D High Flow, Extra High Flow
(PrintConfig.cpp:6291-6302). A label is offered when either of these holds:

- `extruder_variant_list[index]` contains, as a substring (`boost::algorithm::contains`),
  `extruder_type label + " " + volume label`;
- the label is Hybrid and `extruder_max_nozzle_count[index]` is greater than 1 and not nil.

Two more filters then apply:

- E3D High Flow is skipped unless `printer_variant` is 0.4 or 0.6 (Plater.cpp:4055-4057).
- High Flow is skipped for `printer_variant` 0.2 and for the printer model "Bambu Lab X1E"
  (Plater.cpp:4058-4060, 1945-1951).

The combo hides when the lists are shorter than the index (Plater.cpp:4038-4043).

### Data

| key | source | value |
| --- | --- | --- |
| `extruder_variant_list` | `Bambu Lab H2D 0.4 nozzle.json:46-49` | `[0]` Direct Drive Standard, High Flow, E3D High Flow; `[1]` adds TPU High Flow |
| `extruder_variant_list` (parent) | `fdm_bbl_3dp_002_common.json:56-59` | Standard, High Flow on both |
| `extruder_type` | `fdm_bbl_3dp_002_common.json:52-55` | Direct Drive, Direct Drive |
| `extruder_max_nozzle_count` | `fdm_bbl_3dp_002_common.json:64-67` | 1, 1 |
| `printer_variant` | `Bambu Lab H2D 0.4 nozzle.json:14` | 0.4 |

The H2D 0.6 profile inherits the 0.4 list. The upstream options are:

- left: Standard, High Flow, E3D High Flow;
- right: Standard, High Flow, TPU High Flow, E3D High Flow.

The browser shows these (`web/app/src/state/extruders.ts:22-25`). The spec hard-codes
`["Standard", "High Flow"]` for both nozzles (`dualExtruder.spec.ts:71-73`). That is the
common parent's list, from before the v2.5.0 H2D profiles overrode it. The test fails at the
left nozzle, so the steps after line 73 have not run on main.

### Change

- A new e2e module, `web/e2e/flowOptionsOracle.ts`, reads the raw BBL machine profiles under
  `ORCA_SLICER_DIR` (the same source as `filamentOracle.ts`). It resolves `inherits` for the
  five keys in the table and `printer_model`. It returns the options for a printer and an
  extruder index by the rules above, with the display order and the X1E list restated from the
  cited lines.
- `dualExtruder.spec.ts` asserts both nozzles against the oracle: first for "Bambu Lab H2D 0.4
  nozzle", then again after the diameter switch to "Bambu Lab H2D 0.6 nozzle". The stale
  comment goes.

### Seen, not changed

`volumeTypeOptions` in `extruders.ts` matches the comma-split list exactly and has none of the
three filters. For the H2D 0.4 and 0.6 data the result is the same. For "Bambu Lab H2D 0.2
nozzle" it is not: that profile's list holds Standard and High Flow
(`Bambu Lab H2D 0.2 nozzle.json:30-33`), and upstream drops High Flow at 0.2, so it offers
Standard only, while the browser offers both. That is an app gap, outside this document.

## 2. printerSwitchFilaments: a defined control for the U1 flush matrix

### Upstream

The reference is `cliSwitchedExport` (`web/e2e/platerExport.ts`). It keeps the opened 3MF's
`nozzle_volume_type`, one Standard entry, because the CLI fills that key only when it is absent
(OrcaSlicer.cpp:6981-6985). Native `--slice 0` on that export then does the following:

1. It reads `current_nozzle_volume_type` from the 3MF: one entry (OrcaSlicer.cpp:1830-1838).
2. With no `--nozzle-volume-type`, it sets `new_nozzle_volume_type` to Standard for each of
   the four heads (OrcaSlicer.cpp:3417-3431).
3. The vectors differ, so the matrix is rebuilt (OrcaSlicer.cpp:3865-3866).
4. The dataset selector copies the raw one-entry `volume_types` and reads `volume_types[eidx]`
   for heads 2-4 without a bounds check (OrcaSlicer.cpp:3963-3976).

`o8-x2d-scalar-flush-context.md` ("Four-head scalar witness") records this as undefined
behaviour. Native's 61/203 is not a parity target.

### Ares

When the type vector is shorter than the head count and the cells need a dataset,
`options/flush_matrix.rs:62-92` keeps the supplied matrix and resizes `flush_multiplier`. The
browser therefore prints the session's 280 seed scaled by 0.3, 1, 1, 1: 84/280.

### Measured

These are out-of-repo Native slices (OrcaSlicer 2.5.0) of the plater export from a main run of
the spec:

| reference input | `flush_volumes_matrix` | lines that differ from the browser |
| --- | --- | --- |
| export as built, one Standard | 61/203 | `filament_ids`, `flush_volumes_matrix` |
| export with one Standard per `nozzle_diameter` entry | 84/280 | `filament_ids`, `nozzle_volume_type` |

The first row reproduces the spec's own Native output; the only other differing line is the
generator banner. In the second row nothing changes, so the matrix is not rebuilt: the current
and new types are equal, and so are the head counts. Upstream behaviour is defined there and
equals the browser's line. Four Standard entries is also what the GUI session's own
`full_fff_config` holds for this switch.

### Change

None in this document. The explicit-selection slice now follows the GUI's
`PresetBundle::full_fff_config`, so the browser's `nozzle_volume_type` becomes the session's
one entry per head. The four-entry export B above is then the route-faithful reference for
every line, and the one-entry export A is no longer needed. The reference change moves to the
slice that switches that route: `web-explicit-selection-gui-config.md`.

## 3. estimation: a GUI-route reference

### Upstream

The web opens a 3MF with `load_and_select: "always"` (`web/app/src/state/projectPresets.ts`),
which is the GUI load. The GUI starts from `FullPrintConfig::defaults()` (Plater.cpp:9463).
`load_external_preset` then calls `update_non_diff_values_to_base_config`, which needs a system
base preset (Preset.cpp:1090-1115, 3053-3068). With no BBL vendor installed there is none, so
the presets load as "<name>(time-cost.3mf)" and keys the 3MF lacks keep their defaults:
`enable_pre_heating` is false (PrintConfig.cpp:8840-8841).

The reference is the PrintStatistics probe slicing the 3MF on the CLI. `load_project_preset`
fills each missing key from the system preset (OrcaSlicer.cpp:3163-3196), so the reference
uses the X2D values for those keys, for example `enable_pre_heating` 1.

### Measured

Both fixtures, `time-cost.3mf` and `unused-filament.3mf`, store 653 keys, with no
`enable_pre_heating` and no `different_settings_to_system`.

The preset-select probe opened `time-cost.3mf` on the GUI route and gave:

| key | GUI route (probe) | browser | CLI reference |
| --- | --- | --- | --- |
| `enable_pre_heating` | 0 | 0 | 1 |
| `bottom_color_penetration_layers` | 0 | 0 | 3 |
| `extruder_clearance_dist_to_rod` | 40 | 40 | 33 |
| `fan_direction` | undefine | undefine | both |

Model printing time is 2h15m41s in the browser and 2h9m35s in the reference: 366 s apart.

### Why the existing helpers do not fit

- `platerExport` is not usable here. `selectedPresetLines.ts:94-96` records that the CLI rejects
  the export of project-local presets with -17 (the compatibility check is at
  OrcaSlicer.cpp:2882-3003 in v2.5.0; not re-run for this document). The helper below keeps the
  preset names, so it does not depend on that check.
- `withSelectedPresets` rewrites config lines after the slice. The statistics depend on the
  values during the slice, so rewriting lines afterwards is not enough.

### Change

A new helper in `web/e2e/selectedPresetLines.ts`, `withGuiMissingKeys(opened, selection, out)`,
builds the reference input:

- Input: the opened fixture and `openedSelection`, the preset-select probe's GUI result.
- Output: a 3MF whose project config gains, for every key the fixture lacks, the value from
  the probe's `full_fff_config`.
- Keys the fixture stores and preset names stay as they are. Both routes keep stored keys when
  no system base is found.

On the CLI, that 3MF has no missing key for `load_project_preset` to fill. Since
`different_settings_to_system` is absent, `refresh_unlisted` is false and every project key
stays (OrcaSlicer.cpp:2775-2791). The PrintStatistics probe then slices the GUI's values.

`estimation.spec.ts` changes:

- It slices that 3MF for both fixtures.
- It needs both probes. If the statistics probe is set and the preset-select probe is not, it
  throws instead of skipping.
- It records the added keys and values in `result.json` and asserts that the reference
  G-code's config block holds those values.

### Expected result and deferred residual

A separate Ares gap remains after the change, already measured elsewhere. With every key
equal, about 13.84 s of model time remains, plus an early tool change lift: Z 3.6 before
`CHANGE_LAYER` 0.8. This document does not change that.

The spec is expected to still fail on the time rows after this change, with a gap of about
14 s instead of 366 s. That residual is a core slicing gap with its own upstream boundary. It
gets its own entry and is not part of this change.

## Commits and verification

The commit order is docs, then test. There is no feat commit, because no app or core code
changes.

The three specs run on the desktop project with the Native references, and the whole web suite
is compared with main.

| spec | expected after the change |
| --- | --- |
| `dualExtruder.spec.ts` | the flow assertions pass; the first later difference, if any, is reported as found |
| `printerSwitchFilaments.spec.ts` U1 slice | unchanged here; see the explicit-selection slice |
| `estimation.spec.ts` | fails only by the deferred residual, or passes |
| the rest of the suite | no spec that passes on main fails |
