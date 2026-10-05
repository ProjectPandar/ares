# E2E Coverage Gaps

Production regions never executed by the seeded E2E corpus
(`orca_parity` seeded 418 cases + `ksr_fdmtest_v4` 88 cases, per-case
profraw attribution). Region ranges are `startLine:col-endLine:col` from
`llvm-cov export` on the instrumented ares-core test binary
(`ares_core-3eb8b8deb188e29f`). For each gap, the upstream OrcaSlicer
`libslic3r` function and the option needed to reach it are listed so the
coordinator can add E2E oracle cases after pruning finishes.

## project_slice/prepare_infill/combine_infill

- `process.rs:68-132` — `layer_configs()` + helpers (`selected_nozzle`,
  `absolute_height`, `infill_area_threshold`): builds per-region layer
  config vectors from region options (flow/height). Upstream:
  `LayerRegion::infill_area_threshold` (`LayerRegion.cpp:975-979`) and the
  layer-config collection inside `PrintObject::combine_infill`.
  Trigger: any E2E case with `infill_combination = true` on a region.
- `process.rs:136-139` — `groups()` early path / per-config compare loop:
  strict `<` vs `<=` epsilon grouping (`CombineInfill.cpp` grouping).
  Trigger: `infill_combination = true` with regions whose combined layer
  heights straddle `infill_combination_max_layer_height`.
- `process.rs:141-164` — `combine_group()` body: merges layers within a
  group, propagates errors. Trigger: same as above; needs >= 2 regions
  that actually combine.
- Net: the entire `combine_infill` pipeline is E2E-dark because no seeded
  case enables `infill_combination`. Recommended E2E case: a 3MF/project
  with two adjacent regions sharing `sparse_infill_density > 0` and
  `infill_combination = 1`, expected to produce the combined-infill
  G-code path (`Layer::infill_entity`) identical to OrcaSlicer.

## project_slice/prepare_infill/fill_surfaces

- `fill_surfaces.rs:110` + `fill_surfaces.rs:118` (`retag` calls inside
  `prepare_fill_surfaces`): the `Internal → InternalSolid` retag fires
  only when `sparse_infill_density ≈ 100.0` within 1e-4 and not spiral,
  or when `bottom_shell_layers == 0` retags Bottom/BottomBridge →
  Internal. Upstream: `Layer::prepare_fill_surface`
  (`Layer.cpp` fill-surface retag block). Trigger: a case with
  `sparse_infill_density = 100` (full-density infill promotion) and/or
  `bottom_shell_layers = 0`. No seeded case reaches either retag branch,
  so `retag()` (fill_surfaces.rs:115-119) never executes under E2E.
