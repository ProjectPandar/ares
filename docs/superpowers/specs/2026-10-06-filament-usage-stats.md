# Spec: GCodeProcessorResult filament-usage statistics (web Filament view)

Status: draft (structure aligned to upstream)

## Upstream source boundary

`GCodeProcessor` accumulates extruded volume per move in
`GCodeProcessor::UsedFilaments` (`GCode/GCodeProcessor.hpp:583-619`,
accumulation at `GCodeProcessor.cpp:3895-3911`):

`volume_extruded_filament = π·(filament_diameter[f]/2)² · ΔE` on each
`EMoveType::Extrude`, bucketed by `m_extrusion_role`:
- `erSupportMaterial`, `erSupportMaterialInterface`,
  `erSupportTransition` → `support_volumes_per_filament`
- `erWipeTower` → `wipe_tower_volumes_per_filament`
- all other extrusion roles → `model_volumes_per_filament`
- every extrude also increments `total_volumes_per_filament`; color-change
  purge/unretract flushes add to `flush_per_filament` AND
  `total_volumes_per_filament` (`update_flush_per_filament`,
  `GCodeProcessor.cpp:3199-3211, 3954-3959, 4311-4316`).

Caches flush on `process_caches` (`:5939-5944`) at tool-change /
calculate_time boundaries, keyed by the active filament id.
`print_statistics` copy: `GCodeProcessor.cpp:6031-6037`.

`PrintEstimatedStatistics` fields
(`GCode/GCodeProcessor.hpp:46-78`):
`volumes_per_color_change`, `model_volumes_per_extruder`,
`wipe_tower_volumes_per_extruder`, `support_volumes_per_extruder`,
`total_volumes_per_extruder`, `flush_per_filament`,
`used_filaments_per_role`, `modes[]`, `total_filament_changes`, …

`GCodeProcessorResult::extruder_colors` = `filament_colour` config for
empty slots, padded to `filaments_count`, default `#FF8000`
(`GCodeProcessor.cpp:2321-2343`).

`PrintStatistics::total_used_filament` (mm) = Σ `volume / (π·(d/2)²)`
over `total_volumes_per_extruder` (`GCode.cpp:1914-1937`); the
`; filament used [mm|cm3|g]` footer lines are `append(...)` of the same
per-extruder numbers (`GCode.cpp:2330-2370`).

## What the web legend actually reads

`GCodeViewer.cpp` legend paths (EViewType::Tool :3719-3733 and
:3987-3995, EViewType::ColorPrint :4026-4080, plus the summary at
:3560-3640 and all-plates rollup :2620-2720) consume:

| legend field | upstream member | this slice |
|---|---|---|
| Tool-legend usage per extruder | `model_volumes_per_extruder` | included |
| ColorPrint "Model" column | `model_volumes_per_extruder` | included |
| ColorPrint "Support" column | `support_volumes_per_extruder` | included |
| ColorPrint "Flushed" column | `flush_per_filament` | included |
| ColorPrint "Wipe tower" column | `wipe_tower_volumes_per_extruder` | included |
| Summary "total filament" / tool palettes | `total_volumes_per_extruder` → `total_used_filament` | included |
| Legend swatch colors | `extruder_colors` (GCodeProcessorResult) | included |
| `volumes_per_color_change` | partial-time estimate rows | **deferred** |
| `used_filaments_per_role` | feature-type legend | **deferred** |
| `modes[]` time estimates | time sections | already covered by moves/estimate |

mm/g conversions are viewer-side (`get_used_filament_from_volume`,
`GCodeViewer.cpp:2622,3443`): `m = volume/(π·(d/2)²)·0.001` (or imperial
koef), `g = volume·density·0.001` — **NOT** ported; web does the same
conversion from the volumes + `filament_diameter`/`filament_density`.

## Ares plumbing (no new pass)

Emit stage already computes the pieces (`gcode_emit.rs:303-313`):
`finish::account_volumes` re-parses the generated G-code, tracking
E/M82/M83/G90-G91/G92 state and accumulating `π·(d/2)²·ΔE` per active
filament into one totals bucket. `state.extruders.statistics(id)` and
`state.wipe_tower.used_filament(id)` exist.

This slice extends `account_volumes` (still one pass over the emitted
stream) to classify each extruded amount by the current `;TYPE:` feature
marker — the emitted G-code already carries `;TYPE:Support material`,
`;TYPE:Wipe tower`, `;TYPE:* wall/infill` etc.
(`gcode_emit/tags.rs:12,64`; `fan_mover.rs:163` strips the same marker),
which is exactly how upstream's `m_extrusion_role` maps to buckets
(support* → support, wipe tower → wipe tower, everything else → model).
Flush volumes come from the existing color-change / purge handling where
emitted; single-filament cases produce empty maps.

`GCodeProcessorResult` gains upstream-named members:

```rust
pub struct GCodeProcessorResult {
    pub moves: Vec<MoveVertex>,
    pub spiral_vase_mode: bool,
    /// `GCodeProcessorResult::print_statistics` — volume buckets only.
    pub print_statistics: PrintEstimatedStatistics,
    /// `GCodeProcessorResult::extruder_colors`
    /// (`GCodeProcessor.cpp:2321-2343`).
    pub extruder_colors: Vec<String>,
}

pub struct PrintEstimatedStatistics {
    pub model_volumes_per_extruder: Vec<f64>,      // sparse → vec indexed by filament id, 0 for unused
    pub wipe_tower_volumes_per_extruder: Vec<f64>,
    pub support_volumes_per_extruder: Vec<f64>,
    pub total_volumes_per_extruder: Vec<f64>,      // model+support+wipe+flush
    pub flush_per_filament: Vec<f64>,
    /// `PrintStatistics::total_used_filament` in mm (`GCode.cpp:1914-1937`).
    pub total_used_filament: f64,
}
```

(`Vec<f64>` dense-indexed by filament id rather than `std::map` sparse —
every legend loop iterates `used_extruders_ids` and probes `find`, so
zero-filled entries are equivalent; keeps wasm serialization trivial.)

`extruder_colors`: `filament_colour` values; empty/short slots →
`#FF8000`; length = `filaments_count` (`filament_diameter` arity).

`ares-wasm`: pass the struct through on `SliceOutput` (`print_statistics`
+ `extruder_colors` as JSON arrays/objects) — no computation in wasm.

## Acceptance

1. **Footer field compare** (5 printers, stock presets): parse the
   oracle's `; filament used [mm]`, `[cm3]`, `[g]` footer lines and
   assert each `total_volumes_per_extruder[id]`→`mm`,`cm3`,`g`
   conversion matches the emitted oracle values per filament id,
   including multi-filament presets.
2. **Split consistency**: one multi-filament + support + wipe-tower
   case; assert `model + support + wipe + flush == total` per extruder
   (±float tolerance), and `total` matches the footer.
3. `extruder_colors`: preset `filament_colour` values propagate; a
   preset with empty colour slots yields `#FF8000`; vector length =
   `filaments_count`.

RED: `orca_parity`-style test that slices each case with ares, asserts
the fields on `ProcessedGCode.result` equal the oracle-footer-derived
values; fails today because `ProcessedGCode` lacks the fields.
