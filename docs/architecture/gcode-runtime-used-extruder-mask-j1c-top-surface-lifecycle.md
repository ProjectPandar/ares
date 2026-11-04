# J1c: avoid-crossing top-fill-surface lifecycle


## Upstream boundary and non-negotiable decisions

Port the **current layer-region fill-surface state consumed by
`AvoidCrossingPerimeters::get_boundary`**, not a new Ares routing algorithm.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Source citations (all relative to `OrcaSlicer/src/libslic3r`):

- `GCode.cpp:7350–7362`: `GCode::travel_to` starts from logical `last_pos()`.
  `:7416–7431` gates routing on reduce-crossing, the one-shot disable and clear
  writer position, then recomputes retraction. `:7434–7453` retracts/wipes and
  replans if the logical start moved. Preserve those semantics.
- `GCode/AvoidCrossingPerimeters.cpp:1233–1288`: `travel_to` selects internal
  or external coordinates, uses the safe-zone containment predicate, lazily
  builds/rebuilds the boundary and restores the original endpoints.
  `:1291–1313` applies absolute/percent max-detour and wipe eligibility. The
  frozen input has max-detour **0**; that limiting branch is not this root.
- `PrintObject.cpp:583–595`: `prepare_infill` invokes
  `LayerRegion::prepare_fill_surfaces` after surface classification, before
  later shell and infill work.
- `LayerRegion.cpp:936–959`: `prepare_fill_surfaces` turns Top into
  Internal/InternalVoid when top-shell count is zero and spiral mode is off.
  This is existing behavior, not a proposed configuration special case.
- `GCode/AvoidCrossingPerimeters.cpp:1099–1134`: `get_boundary` gathers only
  surfaces that **currently** satisfy `is_top()` in each region's
  `fill_surfaces`, unions them, and subtracts their inset from the inner-offset
  layer boundary. With no current Top surfaces it returns that boundary
  without the subtraction.
- `GCode/AvoidCrossingPerimeters.cpp:437–479, 620–685`: contour travel is
  simplified against this boundary. A stale hole changes which shortcuts
  intersect it even when outer-contour geometry and shortest direction agree.

Rust destination: `crates/ares-core/src/project_slice.rs::slice_source` and
`extract_top_surfaces`. Feed the existing per-object/per-layer carrier from the
mature, post-infill-preparation region `fill_surfaces`, rather than its early
`PreparedPostSurfaceTypeDetection` snapshot. The early copied snapshot is a
temporary compatibility shell and must be replaced, not supplemented by a
zero-top-count filter at the router.

## Frozen RED and read-only causal evidence

Original J1 role2 input SHA256:
`7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4`.
It has reduce-crossing enabled, top/bottom shell count 0, sparse infill 0%,
nearest seam, wipe enabled and no support/tower. Do not tune this fixture.

Existing test:
`j1_by_layer_role2_usage_matches_complete_output` in
`crates/ares-cli/tests/orca_parity/runner/j1_used_extruder_mask_follows_emitted_tools.rs`.
The complete-byte comparator keeps M73, stats and footer; only generated-by
identity is normalized. Three other J1 cases remain **witness-only**
controls, not complete-byte parity claims.

The mask and J1b changes remain separate commits. With the concurrent
avoid-crossing support-spacing changes overlaid, the single fresh primary
comparison remained RED at line 222 (`M73 P46 R1` before versus after the
same G1). The Ares raw output SHA remained
`da4e1fc6b3cc1d32285dab67ec2e9d4d2643bf83c552e0c5829c108fc3b200ad`;
those changes did not change this input's output.

Read-only Nix GDB17.2 probes over that unmodified overlay and the pinned Native
binary identify the **first real movement difference**, not merely M73:

- Both actual planner invocations start at `(2300288, -2274688)` and end at
  `(-2974998, 3025002)` in scaled object coordinates; external flags are false.
- Native boundary has **one** contour. Ares has **three**. Their first/outer
  contour is integer-for-integer identical (16 vertices).
- Ares alone has rectangular holes at X `[-8480231,-3519764]` and
  `[3519769,8480236]`, Y `[-2480231,2480236]`.
- During Ares simplification, the segment from `(-2372392,-227568)` to
  `(-9089312,3089312)` hits the added left-hole edge
  `(-3519764,2480236)→(-3519764,-2480231)` at `(-3519764,339015)`.
  The same candidate in the preceding non-top layer has no collision.
- Native retains waypoint `(-9089312,3089312)` → emitted
  `G1 X152.911 Y103.089`; Ares instead retains `(-2910714,-1055281)` →
  `G1 X159.089 Y98.945`. This is the line-333 movement difference in the
  fresh overlay comparison (the earlier M73 difference remains at line 222).
- Ares shortest-direction observations are backward in both layers, with
  large forward/backward margins, not a float tie. The extra-hole collision,
  not shortest-direction selection or max-detour, explains this shortcut.
- GDB Ares output is byte-identical to the uninstrumented overlay output.
  No production logging, debugger state mutation or Native rebuild was used.

Debugger calls are diagnostic-only, not substituted for fresh public-API E2E
receipts.

The source chain matches the observed extra holes:
`project_slice.rs:161–170` extracts Top immediately after classification;
`prepare_infill/fill_surfaces.rs::prepare_record:89–96` subsequently retags
zero-top-count Top surfaces; `extract_top_surfaces`' copied polygons are never
updated. `layers/entry_builder.rs:48–51` and `layers/entry.rs:73` pass that old
snapshot to `motion/path/avoid_crossing/boundary.rs:150–168` for subtraction.

## Included change and plan

1. Retarget the existing extraction to the mature region surface owner after
   `combine_infill::prepare`, before `fill_entities::prepare` consumes it.
   The owner is
   `prepared.predecessor.predecessor.predecessor`:
   `PreparedPostHorizontalShellPropagation`, containing current `objects`
   and the same `PreparedPostClassicTraversal` record identities.
   Preserve object/region/layer alignment and the existing Top predicate.
   The current prepared-state chain retains the mutated surfaces; no new
   pipeline, parallel state or raw-configuration parsing is needed.
2. Keep the existing extraction/carrier shape and routing consumer. Do not
   change `fill_surfaces::prepare_record`, shortest direction, intersection
   arithmetic, safe-zone tests, boundary inset, max-detour, wipe, seam,
   cooling or processor time/export insertion rules.
3. Before GREEN, establish the existing complete-byte primary RED as the J1c
   symptom and add a frozen test-only lifecycle supplement: retain
   the original zero-top case and a separately frozen nonzero-top control
   to witness that genuinely surviving Top regions are still avoided.
   Any additional control is frozen before its first Native call and uses
   public slicing callers; no private-emitter test API.
4. Implement a separate J1c commit after the RED. Keep touched Rust files
   ≤400 LOC; extract only the affected
   top-surface helper if the surgical edit would exceed the limit.

## Deferred behavior and ownership

Do not fix the pre-existing float-distance or collinear-intersection gaps,
external-planner lifecycle gaps, support default-spacing rules, by-object
ordering, candidate collector, or any independent route/Numerical issue.
M73 progress and cooling are downstream symptoms: no insertion-rule change
is allowed. Whole-file GREEN is still mandatory; if fixing the proven
boundary lifecycle leaves another difference, freeze it and seek a new
source-cited boundary rather than expanding this repair silently.

**Function overlap:** the concurrent `avoid_crossing.rs::route` and
`avoid_crossing/boundary.rs::Boundary::build` support-spacing changes were
inspected read-only. This spec does **not** propose touching either function
or file; it corrects the upstream fill-surface owner supplying them. If GREEN
work discovers that either shared function needs an edit, amend this spec
with the exact shared function before editing.

## Exit gates

- Unchanged primary input becomes **whole-file GREEN**, including all M73,
  stats/footer, and the frozen three startup witnesses remain exact.
- Fresh directed lifecycle controls demonstrate zero-top removal
  and preservation of surviving Top surfaces; seed0, runs1, no reused
  Native result cache, prebuild outside a shared 300s case deadline, retain
  errors and owned-descendant cleanup receipts.
- The full seeded selection includes BBL/multi-extruder cases, so base and
  candidate seeded runs must come from the same round on the same base; do
  not carry stale seeded results across a behavior change. Preserve
  independent mask, J1b and J1c commits.
- Strict clippy, rustfmt, WASM and workspace/official gates; candidate named
  failures ⊆ matched base. No failing-test bypass or timeout promoted to
  parity.
- The unrelated legitimate non-GREEN matrices and witness-only limitations
  remain reported honestly.
