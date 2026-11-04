# E2E Coverage Gaps

Production regions never executed by the seeded E2E corpus
(`orca_parity` seeded 418 cases + `ksr_fdmtest_v4` 88 cases, per-case
profraw attribution). Region ranges are `startLine:col-endLine:col` from
`llvm-cov export` on the instrumented ares-core test binary
(`ares_core-3eb8b8deb188e29f`). For each gap, the upstream OrcaSlicer
`libslic3r` function and the option needed to reach it are listed so
E2E oracle cases can be added after pruning finishes.

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

## project_slice/prepare_infill/horizontal_shell_promotion

- `promote.rs:25-55` — `stage_decision()` non-empty path
  (`ExtraSolidInfills::parse_raw` + `schedule.matches_layer`) and
  `commit()`'s `Internal → InternalSolid` retag loop. Upstream:
  OrcaSlicer `PrintObject::extra_solid_infill` schedule matching
  (`PrintObject.cpp`). Trigger: `extra_solid_infills` option set to a
  non-empty schedule (e.g. `1` or `0,2,4`) on a project; all 6 tests
  exercise regions unreachable without it.

## project_slice/prepare_infill/vertical_shell_assignment

- `vertical_shell_assignment.rs:47-55, 72-100` — assign/stage retag +
  difference path: Internal→InternalSolid promotion where the vertical
  shell fully covers an internal surface, and the empty-filter / alloc
  exact no-op short-circuit. Upstream: `PrintObject::prepare_infill`
  vertical-shell assignment (`PrintObject.cpp`). Trigger: a model whose
  top/bottom shell boundary crosses internal sparse infill so the
  assignment loop has nonzero work; seeded corpus never reaches the
  uncovered regions, so no E2E case exercises assignment.

## project_slice/prepare_infill/horizontal_shell_propagation

- `geometry.rs:148-165` — `first_factor`/`second_factor` select on
  `ensure_vertical_shell_thickness` mode (None/CriticalOnly/Moderate/
  EnsureAll). Upstream: `PrintObject::discover_horizontal_shells` factor
  tables (`PrintObject.cpp`). Trigger: projects with
  `ensure_vertical_shell_thickness = critical_only` / `moderate` /
  `ensure_all`; the None-only path is what seeded E2E reaches.
- `geometry.rs:113-120` + `gather.rs`/`rebuild.rs`/`hooks.rs` — miter-5
  propagation opening, ordered geometry-site error mapping. Real
  production paths gated by the same option + non-empty neighbor sets.

## vertical_shell_{s,filtering,projection,regularization,trimming,assignment} + surface_type_detection

- Predominant uncovered regions are `#[cfg(test)]` instrumentation
  (`observe`/`record_*` failure hooks, EVENTS/VOLUME_SNAPSHOTS
  thread-locals) that cannot run under E2E — not real gaps. Real
  uncovered production paths: `ensure_vertical_shell_thickness` modes
  other than None and the Internal→InternalSolid retag arms where the
  shell fully covers internal surfaces.
