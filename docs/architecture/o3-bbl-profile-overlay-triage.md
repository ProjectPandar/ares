# O3-BBL: BBL official-mode difference triage (profile overlay, label identity)

## Decision

Triage only. No production change, no RED, no GREEN is included here; this
document locates owners and proposes slices. The input is three official-mode
cases produced during the O3 scalar-array work.

Upstream revision: OrcaSlicer v2.4.2. Paths below are relative to
`OrcaSlicer/src/`.

**Conclusion: the three first-difference lines do not share one owner.** The
cases surface five distinct differences; two are production defects in the
3MF selected-profile overlay (`project/domain/profile_input*`), one is the
unported STL `ObjectID` label sequence, one is a missing CLI bed-resize step,
and two are oracle-construction mismatches in the parity harness rather than
Ares pipeline defects. Each is sized and cited separately so they can be
addressed independently.

## Reported cases and the full difference set

| Case | Native command | Ares entry | First diff | Also differs |
| --- | --- | --- | --- | --- |
| `stl-model-variant` | `--load-settings machine;process --load-filaments filament --arrange 0 --slice 0 cube10.stl` | `slice_with_profiles` (STL + catalog) | line 6 `model label id` 14 vs 0 | line 168 `filament_self_index` `1,1` vs `1` |
| `embedded-script` | same flags, input `preset-json-ender3-0.2.3mf` (Creality Ender-3 project) | `slice_with_profiles` (3MF + catalog) | line 38 `before_layer_change_gcode` empty vs embedded script | `extruder_clearance_radius` 68 vs 47; `wipe_tower_x/y` 33/238 vs 15/220 and object translation (+18,+18); `filament_self_index`; `inherits_group` 3 slots vs 7 |
| `exported-project-profile-metadata` | `--slice 0 project-array.3mf` (**no** `--load-settings/--load-filaments`) | `slice_with_profiles` (3MF + catalog) | line 87 `different_settings_to_system` `;;` vs `enable_prime_tower;;` | `filament_self_index`; `inherits_group` 3 vs 7 |

## D1. `model label id` (STL only) — unported `ObjectBase` id sequence

Upstream: `libslic3r/Model.hpp:1263-1273` `ModelInstance::get_labeled_id()`
returns `loaded_id` only when `use_loaded_id_for_label && loaded_id > 0`
(`OrcaSlicer.cpp:5734` sets the flag for every instance), otherwise
`id().id`. `loaded_id` is written only by the 3MF loader
(`Format/bbs_3mf.cpp:2346`); an STL instance keeps `0`, so the label is the
process-global `ObjectBase::s_last_id` counter value at construction
(`libslic3r/ObjectID.hpp:65,86-87`, `ObjectID.cpp:5`). Header emission is
`libslic3r/GCode.cpp:2594`; object markers `:5357,:5483`.

Why native prints 14 for a single-object STL (every default-constructed
`ObjectBase` increments; `ObjectBase(-1)` members do not):

| # | Constructor | Source |
| --- | --- | --- |
| 1-2 | outer `Model model;` + its `ModelWipeTower wipe_tower` member | `OrcaSlicer.cpp:1612`, `Model.hpp:1532` |
| 3-4 | inner `Model model;` + `wipe_tower` inside `Model::read_from_file` | `Model.cpp:256` |
| 5-7 | `ModelObject` + `config` + `layer_height_profile` via `load_stl` → `Model::add_object` | `Model.cpp:279,467-500`, `Model.hpp:354,373` |
| 8-13 | `ModelVolume` + `config` + `supported/seam/mmu_segmentation/fuzzy_skin_facets` | `Model.hpp:794,870-879` |
| 14 | `ModelInstance` via `add_default_instances` | `Model.cpp:361,658` |

The move-assignment at `OrcaSlicer.cpp:1640` keeps ids (`assign_copy`), so
the instance label stays 14. For N objects/volumes the sequence continues in
load order; for multiple STL inputs a new outer `Model` is constructed per
file (`:1612`).

Ares: `project/domain/profile_input.rs:197` builds
`ProjectInstance::new([1, 0, 0], …)`, so `loaded_label_id = Some(0)`;
`project_slice/gcode_emit/header.rs:22-27` and `gcode_emit/object.rs:43-66`
print it. The 3MF path is already correct (identity from
`model_settings.config`, label 14 in the `embedded-script` case).

Proposed slice (independent): port the `ObjectID` sequence as an explicit
STL-import label allocator in `profile_input.rs::from_stl_with_profiles`
replicating the construction order above (`2 + 2 + Σ_objects(3 + 6·volumes + instances)`),
assigning each instance its `id().id`. Deferred: multiple input files,
`--clone`/`add_object(copy)` re-ids (`OrcaSlicer.cpp:1833`), 3MF `loaded_id`
fallback at `Print.cpp:4729,4952` (already handled by the 3MF loader).

## D2. 3MF overlay applies `handle_legacy` after the overlay (production defect)

Upstream loads each `--load-settings` file through
`DynamicPrintConfig::load_from_json` → `set_deserialize` → `handle_legacy`
(`libslic3r/Config.cpp:577-579`, `PrintConfig.cpp:8033`) **before**
`update_full_config` overlays it (`OrcaSlicer.cpp:2750-2812`). The machine
file carries the legacy key `extruder_clearance_max_radius = 68`; upstream
renames it to `extruder_clearance_radius` (`PrintConfig.cpp:8248-8249`) at
load, so the overlay replaces the embedded `47` with `68`.

Ares `project/domain/profile_input.rs:85-103` merges the raw profile keys into
the embedded raw first and runs `merged.handle_legacy()` afterwards.
`ProjectSettingsRaw::handle_legacy` (`project/raw_settings.rs:81-122`)
iterates a `BTreeMap`, so the renamed `extruder_clearance_max_radius → 68`
is inserted and then overwritten by the embedded `extruder_clearance_radius
= 47` that sorts after it. The STL branch (`:132-138`) already runs
`handle_legacy` on the profile raw alone and is unaffected.

Proposed fix boundary: run `handle_legacy` on `profile_raw` and each
`filament_raws[i]` before `merged_profile_raw`, keep the post-merge call for
the embedded base (the embedded config is itself loaded through
`set_deserialize`). No new rule, no fallback.

## D3. `load_default_gcodes_to_config` is not ported (production defect)

`OrcaSlicer.cpp:548-586,2850` `load_default_gcodes_to_config(load_machine_config,
TYPE_PRINTER)` calls `option<ConfigOptionString>(key, true)` for
`change_filament_gcode, layer_change_gcode, machine_end_gcode,
machine_pause_gcode, machine_start_gcode, template_custom_gcode,
printing_by_object_gcode, before_layer_change_gcode, time_lapse_gcode,
wrapping_detection_gcode`, creating empty options for keys the machine file
lacks. Those keys then overlay the embedded project in `update_full_config`,
which is why native prints `before_layer_change_gcode =` (empty) over the
Ender-3 embedded script. `:2994` (process, no-op branch) and `:3150`
(filament: `filament_start_gcode`/`filament_end_gcode` resized to one empty
slot when empty) are the sibling calls.

Ares `project/domain/profile_input/overlay.rs:51-56` only overlays keys
present in `profile_raw`; the BBL machine chain
(`fdm_machine_common → fdm_bbl_3dp_001_common → Bambu Lab X1 Carbon 0.4 nozzle`)
never defines `before_layer_change_gcode`, so the embedded value survives.

Proposed fix boundary: in `Catalog::compose` (`profiles/catalog.rs:156-201`)
after the machine merge, insert the ten printer G-code keys into the machine
raw as empty `Text` when absent, and resize empty filament
`filament_start_gcode`/`filament_end_gcode` to one empty slot — a direct port
of `load_default_gcodes_to_config`, applied before the overlay reads
`profile_raw`.

## D4. `shrink_to_new_bed` bed re-centering is not ported (deferred, separate slice)

`OrcaSlicer.cpp:1777` reads the embedded `printable_area` as the old bed;
`:3766-3795` sets `shrink_to_new_bed = 1` when the loaded machine bed is
larger (Ender-3 220×220 → X1C 256×256), `= 2` when smaller or the exclude
area changed. `:3958-4053 translate_models` then translates every plate
instance by the center delta (`(256-220)/2 = 18` on both axes) and adds the
same `wipe_offset` to `wipe_tower_x/y` (`15,220 → 33,238`). This explains
the geometry/G29 shift and the wipe-tower lines in the `embedded-script`
case. Ares has no equivalent (`grep shrink_to_new_bed` is empty).

This is a model-placement slice, not a config overlay slice; it should be
specified separately after D2/D3 land (both are prerequisites for the
printable-area comparison to see the loaded machine's bed).

## D5. `inherits_group` 7 slots (3MF + catalog cases) — Text/List shape mismatch

Upstream keeps `inherits_group` as a `ConfigOptionStrings` of
`filament_count + 2` slots, written slot-wise (`OrcaSlicer.cpp:2816-2817`,
`:2840-2842` machine, `:2983-2985` process, `:3163` filaments) and
serialized by `escape_strings_cstyle` (`Config.cpp:72-120`).

Ares `profiles/catalog.rs:193-195` stores the composed group with
`insert_strings`, i.e. one pre-serialized `RawConfigValue::Text`
(`"0.20mm Standard @BBL X1C";;`). `overlay.rs:128-153` then reads positional
elements via `RawConfigValue::element` (`raw_settings.rs:179-185`), which
returns the whole text at index 0 and `None` elsewhere, so `inherits_slot`
places the full serialized string in both the process and machine slots:
`"A";;` + `;` + `;"A";;` = 7 tokens. The STL branch copies the Text through
unchanged, which is why only the 3MF cases show it.

Proposed fix boundary: `Catalog::compose` stores `inherits_group` (and the
expression groups written by `cli_metadata::filament_conditions`, which must
be checked for the same shape) as `RawConfigValue::List`, matching what
`merged_profile_raw` reads. This is a one-line shape fix plus a RED that
slices a 3MF with a catalog and asserts the three-slot group.

## D6. `filament_self_index` `1,1` vs `1` — oracle flattening, not Ares (needs ruling)

Native's `1,1` comes from the CLI sizing `filament_self_index` by the loaded
filament's `filament_extruder_variant` length (`OrcaSlicer.cpp:3117-3337`;
`update_values_to_printer_extruders` later collapses the filament vectors to
the active variant but leaves `filament_self_index` untouched, hence
`filament_extruder_variant = "Direct Drive Standard"` next to `1,1`). The
loaded `filament.json` has two variants only because the harness flattener
`crates/ares-cli/tests/orca_parity/presets.rs:145-163` resolves `inherits`
by plain key overwrite, so the leaf `Bambu PLA Basic @BBL X1C`
(`filament_extruder_variant = [Standard, High Flow]`) replaces the root
`fdm_filament_common` (`[Standard]`).

Upstream's own inheritance (`libslic3r/Preset.cpp:1680-1684` →
`PrintConfig.cpp:10209-10298 update_diff_values_to_child_config`) skips the
variant key (`:10263`) and maps child values onto the **parent's** variant
layout via `set_only_diff`, so the merged X1C preset has one variant and
`PresetBundle::full_fff_config` (`PresetBundle.cpp:133-135`) would emit
`filament_self_index = 1`. Ares `options/filament_options.rs:119-147,245-262`
(`apply_profile_child` / `profile_variant_mapping`) is a port of that path
and produces `[1]` — confirmed by probing `profile_placed_project` on the STL
case (`filament_self_index = [1]`, `filament_extruder_variant =
["Direct Drive Standard"]` before any materialization).

Two upstream paths disagree; Ares matches the preset-loader path and the
oracle matches a test-only flattening. Options:

1. Fix the harness: make `presets.rs::flatten` port
   `update_diff_values_to_child_config` (variant-mapped child overlay) so the
   flat files fed to the native CLI equal upstream's merged preset. Ares
   production unchanged. **Recommended** — it removes the divergence for
   every BBL two-variant filament, not just these cases.
2. Change Ares composition to key-overwrite inheritance. Rejected here: it
   diverges from `Preset.cpp` and would alter every variant-mapped value.

## D7. `different_settings_to_system` in the export case — command asymmetry

Native sliced `project-array.3mf` with no `--load-filaments`, so
`load_filament_count == 0` and `disable_wipe_tower_after_mapping` never
turns on (`OrcaSlicer.cpp:2275-2279`); `different_settings[0]` stays empty
(`:3673-3701` is skipped). Ares ran `slice_with_profiles`, whose overlay
always models the loaded-filament path and records `enable_prime_tower` in
slot 0 (`overlay.rs:159-207`). `enable_prime_tower = 0` itself agrees on
both sides (single-filament normalization).

No Ares change: the cases compared different commands. The export-case
test must either slice the exported 3MF through `slice_project_plate`
(plain 3MF entry) or run native with the same `--load-settings/--load-filaments`
flags as the Ares catalog request.

## Proposed order and exit criteria

1. D5 (`inherits_group` List shape) — smallest, unblocks byte comparison of
   the 3MF+catalog cases. RED: 3MF + catalog slice asserts
   `; inherits_group = "0.20mm Standard @BBL X1C";;`.
2. D2 (legacy before overlay) — RED: Ender-3 3MF + BBL catalog asserts
   `extruder_clearance_radius = 68`.
3. D3 (`load_default_gcodes_to_config`) — RED: same input asserts empty
   `before_layer_change_gcode`.
4. D1 (STL label sequence) — RED: STL + catalog asserts `model label id: 14`
   and the object markers.
5. D6 harness ruling, then D7 test command alignment; D4 as its own spec.

Each RED is an E2E native-vs-Ares comparison normalized only on the
`; generated by` line; after D1-D3+D5 the `stl-model-variant` case should be
byte-identical once D6 is ruled, and the `embedded-script` case reduces to D4.

## Chain exit criterion: whole-file byte comparison

Until D6/D7 land, the D1/D3 cases compare only the owned lines (the
`; model label id` line, the ten printer G-code tokens) because the same
inputs still carry the open D5 (`inherits_group` shape), D2 (legacy
ordering), and D6 (harness filament flattening) differences, so a whole-file
comparison cannot pass yet. These single-line comparisons are interim
scaffolding, not the chain's end state. The final slice of this chain, after
D6 and D7, must upgrade the `stl-model-variant` STL case and both 3MF catalog
cases (`embedded-script`, `exported-project-profile-metadata`) to whole-file
byte comparison against native, normalized only on the `; generated by`
line. The chain is not done until those cases compare whole files.

Current state: the `exported-project-profile-metadata` case (aligned commands,
`project_export_metadata.rs`) and the `stl-model-variant` STL case
(`stl_variant_whole_file.rs`) compare whole files after D1/D2/D5/D6; the
`embedded-script` case keeps its interim line comparisons until D4
(`shrink_to_new_bed`) lands, since its residual is exactly the bed
re-centering gap.

## Known native nondeterminism (official 2.4.2)

The official binary's reference output flips between two travel variants on
identical inputs. Observed twice during the D5/D2/D6/D7 gate rounds, each
time failing a different tree-family case while the input zip entries and
the Ares bytes stayed identical across rounds:

- `tree_support::…slim_tree_auto_overhang` (round 1) and
  `support_annotations::…mirrored_support_blocker` (round 2) share the same
  first-difference line — `G1 X113.388 Y113.135 F9000` vs
  `G1 X113.244 Y113.297 F9000` — one native ND source, not per-case bugs.
- Per-occurrence disposition evidence: input zip entries byte-identical,
  Ares output byte-identical, native reference byte-different.
