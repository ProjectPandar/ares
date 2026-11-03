# bridge_over_infill: remove the single-extruder guard (multi-filament infill anchors)

Matrix finding (lint's ARES_UNSUPPORTED entry): multi-filament input with
`sparse_infill_filament_id=2` and walls on filament 1 is rejected by ares at
`bridge_over_infill_anchor_extruder_order`
(`prepare_infill/bridge_over_infill/transaction/anchor_projection.rs:96-99`);
native slices it and emits G-code (verified: seeded orca on
`${TMP_ROOT}/ares-print-bed-native/input-multi-used.3mf` → 6.7 MB
`plate_1.gcode`, no error).

## Upstream boundary

- `PrintObject.cpp:2468` (`PrintObject::bridge_over_infill`): the whole pass
  operates on `LayerRegion`s; each `CandidateSurface` carries a
  `const LayerRegion *region` and rewritten surfaces are appended back into
  `region->fill_surfaces` (`PrintObject.cpp:3555+` — `remove_types` +
  `append(new_surfaces)` per region). There is **no cross-region or
  cross-filament equality precondition** — upstream never compares
  `sparse_infill_filament_id` to `internal_solid_filament_id` /
  `top_surface_filament_id`.
- Extruder selection happens at fill emit, per surface type, per region:
  `Fill.cpp:865` `params.extruder = layerm.region().extruder(extrusion_role)`
  (sparse infill) and `Fill.cpp:920-924` (`top_surface_filament_id` /
  `bottom_surface_filament_id` / `internal_solid_filament_id`). Rewritten
  `stInternalSolid` surfaces therefore emit on `internal_solid_filament_id`
  and `stInternal` on `sparse_infill_filament_id` — the tool split is
  automatic.
- `PrintApply.cpp:815-823, 1060-1073`: painted/layer-range regions clone the
  parent config with all six `*_filament_id` set to the painted extruder,
  so a region's filament triple may legitimately differ.

## Ares state

- The guard is the **only** use of `*_filament_id` inside
  `prepare_infill/bridge_over_infill/` — it is an unsupported-closure
  precaution from `dd818eb29` (activate internal bridge transaction), not a
  port of an upstream check.
- Emission already selects per-region filament by role:
  `group_fills/params/projection.rs:241-243` (TopSolidInfill →
  `top_surface_filament_id`, SolidInfill → `internal_solid_filament_id`),
  `perimeters/flow/fill.rs:22` (Infill → `sparse_infill_filament_id`),
  `gcode_emit/tool_ordering/collect.rs:63-66`,
  `tool_buckets/object.rs:231-237`, `wiping/source.rs:56,125-131`,
  `extruders.rs:77-94`, `header.rs:158-168`. No emission change is needed —
  the rewritten surfaces flow through the same role→filament projection.
- Test to update:
  `project_slice/tests/prepare_infill/bridge_over_infill/transaction/options.rs:182-186`
  (`task22o71_unported_anchor_group_params_fail_without_fallback`) —
  the `top_surface_filament_id: 2` mutation currently asserts
  `bridge_over_infill_anchor_extruder_order`; it must be flipped to assert
  the transaction succeeds (native accepts).

## Change

Delete the `*_filament_id` equality block in
`transaction/anchor_projection.rs:validate` — lines 96-99. Nothing else in
the bridge transaction consumes the ids; upstream parity comes from the
existing per-region role→filament emission path, which needs no port.

Explicitly **not** adding: upstream has no equivalent check to port —
`bridge_over_infill` is extruder-agnostic by construction (per-region
surface ownership). The spec's "port" is removing an ares-only guard that
upstream semantics make unnecessary.

## Verification

1. RED: flip the `top_surface_filament_id: 2` case in
   `task22o71_unported_anchor_group_params_fail_without_fallback` to assert
   success — fails while the guard stands.
2. E2E: `ares slice ${TMP_ROOT}/ares-print-bed-native/input-multi-used.3mf`
   produces G-code; byte-compare against seeded-orca
   `/tmp/bridge-orca-out/plate_1.gcode` (modulo header timestamp) — expected
   identical if the transaction is otherwise parity-complete, or a bounded
   diff counted and reported if prior pipeline drift exists on this input.
3. Full gates: `cargo nextest run --workspace`, clippy, fmt, wasm check;
   controls official+seeded.
