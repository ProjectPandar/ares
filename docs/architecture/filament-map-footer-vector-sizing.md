# Configured filament map vector sizing

## Upstream boundary

This slice rewrites the configured-filament vector sizing that precedes the
`Print` map write-back and G-code config export in OrcaSlicer v2.5.0,
revision `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Upstream owner | Included behavior | Rust destination |
|---|---|---|
| `PresetBundle.cpp:3966–3970, 4129–4133` | Initialize `filament_map`, `filament_nozzle_map`, and `filament_volume_map` with one entry per configured filament when the project filament set is assembled. | `profiles/filament_slots.rs::FilamentSlotState::normalize` |
| `PresetBundle.cpp:4328–4372` | `set_num_filaments` resizes all three maps whenever the configured filament count changes, preserving existing values and filling new entries with the upstream defaults. | The same normalization boundary, after the filament count is known. |
| `PresetBundle.cpp:5534–5567` | Full FFF config keeps the filament map and volume map sized to `num_filaments` before variant expansion. | Existing full-config materialization consumes the normalized vectors; no footer-only substitution. |
| `Print.cpp:3700–3743, 4661–4680` | Sequential map preparation and `update_filament_maps_to_config` retain configured entries for unused filaments, while writing the complete nozzle and volume vectors back before export. | Existing `project_slice::gcode_emit::tool_ordering` and config-view checkpoint path. |
| `PrintConfig.cpp:11855–11903, 11979–12042` | Variant rebuilding uses the final filament map and accepts a volume map only when it is filament-count sized. | Existing `options/project_variants` refresh and variant materialization. |

## Observed behavior

`tests/parity/sequential-first-layer-nozzle-temperature/second-tool.3mf`
contains two filament colors, two configured nozzles, and a two-entry manual
`filament_map`, but omits the internal nozzle and volume map keys. Only the
second tool is emitted in sequential by-object mode. Native exports:

```text
filament_map = 1,2
filament_map_2 = 0,1
filament_nozzle_map = 0,1
filament_volume_map = 0,0
```

Ares currently exports the first two lines correctly but emits one-entry
`filament_nozzle_map = 1` and `filament_volume_map = 0`. The existing Rust
normalizer resizes all three internal maps only when filament colors grow. A
project whose colors already have the configured count therefore keeps the
registered one-entry defaults. The existing sequential footer then serializes
those stale vectors. This is a configuration-state sizing defect, not a
footer formatting defect and not a change to filament assignment semantics.

Main already contains the adjacent non-BBL automatic-map rewrite in squash
commit `971b136e4` (`fix(cli): publish used filament slots for non-BBL maps`).
That behavior derives a static identity recommendation after actual usage and
publishes used-only extruder/volume assignments with a complete nozzle map.
This residual is different: it is the sequential Manual/by-object path, where
`FilamentSlotState::normalize` only resizes the three maps when filament colors
are expanded (`filament_slots.rs:86–88`). It must not alter the 971b136e4 owner
or broaden the FilamentGroup optimizer.

## Required behavior

1. Once the configured filament count is known, all three internal maps have
   exactly that length.
2. Existing entries are preserved. Newly created entries use upstream defaults:
   extruder `1`, nozzle `0`, and standard volume `0`.
3. This sizing occurs before sequential and non-sequential config views are
   materialized, so `Print` write-back and the existing footer exporter observe
   the same full vectors.
4. Used-filament assignment behavior remains unchanged. In particular, the
   sequential two-tool fixture must retain `filament_map = 1,2` and emit
   `filament_nozzle_map = 0,1` and `filament_volume_map = 0,0`.
5. Manual, single-tool, automatic non-BBL, BBL, volume-type, and multi-nozzle
   map paths retain their existing owners.

## Verification seam

Turn the existing tool-qualified sequential case into a strict whole-file
Native/API/CLI comparison after the vectors are sized. The test must continue
to assert the temperature/tool restart behavior and must compare the complete
G-code stream without removing or rewriting the two map lines.

Add a minimal non-sequential two-tool synthetic project built at runtime from
an existing public 3MF input. Configure two filament slots, two physical
nozzles, omit the internal map keys, and use only the second logical filament.
Slice identical bytes through Native, the public API, and the CLI in official
and seeded environments. Compare the complete `filament_map`,
`filament_map_2`, `filament_nozzle_map`, and `filament_volume_map` values and
the complete G-code stream. The synthetic input must remain unmodified on
disk and no Native output is committed.

Run the existing filament-map, flush, and multi-tool inventories in both
Native modes. Previously passing cases must remain passing. Run formatting,
strict workspace/all-target Clippy, and the core/WASM check because the
normalization boundary is shared by browser and native slicing.

## Deferred behavior

This slice does not change filament grouping, map recommendation, nozzle
selection, flush matrix recomputation, printer admission, variant selection,
footer key ordering, or any public API. It does not port undefined upstream
behavior from malformed or internally inconsistent configurations. Existing
multi-nozzle expansion and dynamic per-layer map publication remain deferred
to their current source boundaries.
