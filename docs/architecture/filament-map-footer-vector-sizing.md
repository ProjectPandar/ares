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
| `PrintConfig.cpp:3095–3100, 3118–3120, 3151–3153` | Registered defaults are filament extruder `1`, standard volume `0` (`nvtStandard`), and nozzle `1` at the config-definition layer; the selected project map initialization supplies the serialized nozzle value `0`. | `profiles/filament_slots.rs` keeps the selected/runtime defaults as map `1`, volume `0`, and nozzle `0`. |
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
2. Existing entries are preserved. Newly created entries use the upstream
   defaults: `filament_map` extruder `1` from `PrintConfig.cpp:3095–3100`,
   `filament_volume_map` standard volume `0` (`nvtStandard`) from
   `PrintConfig.cpp:3118–3120`, and selected/runtime
   `filament_nozzle_map` physical nozzle `0`, initialized by
   `PresetBundle.cpp:3966–3970, 4129–4133` from the registered
   `PrintConfig.cpp:3151–3153` definition.
3. This sizing occurs before sequential and non-sequential config views are
   materialized, so `Print` write-back and the existing footer exporter observe
   the same full vectors.
4. Used-filament assignment behavior remains unchanged. In particular, the
   sequential two-tool fixture must retain `filament_map = 1,2` and emit
   `filament_nozzle_map = 0,1` and `filament_volume_map = 0,0`.
5. Manual, single-tool, automatic non-BBL, BBL, volume-type, and multi-nozzle
   map paths retain their existing owners.

## Verification seam

Keep the existing restart-span comparison in the tool-qualified sequential
case. Compare the complete `filament_map`, `filament_map_2`,
`filament_nozzle_map`, and `filament_volume_map` values in the config footer
from Native, the public API, and the CLI. A strict whole-file comparison is
deferred: before the footer, Ares already emits an additional
`G10 S207 P0` at the transition out of the first layer, while Native emits
only `G10 S207 P1`. The comparison does not normalize that command.

Use a minimal non-sequential two-tool control built at runtime from the
same public 3MF input. Both cases configure two filament slots and two
physical nozzles, omit the internal map keys, and use only the second
logical filament. Check all four fields for both cases in the official and
seeded environments. No Native output is committed.

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
