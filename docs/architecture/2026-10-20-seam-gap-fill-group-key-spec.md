# SurfaceFillParams: seam-gap loop clipping is owner data, not group identity

Status: SPEC-REVIEW-REQUEST; docs only. Discovery code base is content-approved
flow-ratio GREEN `14017addf9250d71d8a6fc468df1dff67f6b441f`, not formal main
acceptance. Main remains `a5eb0f0356057fe9c62ec3319834f82b40a7731a`.
Queue: W1 → W4 → consistent-grid → flow-ratio → any subsequently approved
slice. This requests SPEC review only, no FULL or merge window.

## Owning upstream rewrite boundary

OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/Fill.cpp:275-305`,
  `SurfaceFillParams::operator<`: ordered-set identity has geometric
  flow width/height/nozzle, bridge/role/speed etc., but no seam_gap or
  loop_clipping discriminator. The separate `operator==` is not the ordered
  set's equivalence relation.
- `Fill.cpp:1012-1051`, `group_fills`: intern by that comparator, retain the
  first parameter and contributing region as owner, append other matching
  surfaces/region IDs and union their no-overlap domains. Existing priority/
  domain union then combines their touching polygons.
- `Fill.cpp:1274-1279`, `Layer::make_fills`: select
  `m_regions[surface_fill.region_id]` and only then set
  `f->loop_clipping = coord_t(scale_(owner.config().seam_gap.get_abs_value(
  surface_fill.params.flow.nozzle_diameter())))`.
- `Fill.cpp:1470-1475`: sparse anchoring uses the same grouped owner region
  and retained Flow nozzle for its filler clipping assignment.

Rust destination: only
`crates/ares-core/src/project_slice/group_fills/coalesce.rs::compare`'s
`left.loop_clipping.cmp(&right.loop_clipping)` term. Remove that extra key;
keep the computed value, existing first-interned parameter/region ownership,
flow/nozzle resolution and consumer behavior unchanged. The current
`SurfaceFillParams.loop_clipping` member remains a temporary compatibility
shell transporting the source owner's Fill field, not an Ares-owned group
identity. This is a source-cited ordered-set slice, not new pipeline design.

## Exact nozzle, region and arithmetic chain

The first-layer Bottom/Monotonic witness is **not a nozzle mismatch**:

1. `Fill.cpp:861-866` chooses `FlowRole::frSolidInfill` for this solid,
   non-top surface. `Fill.cpp:954-959` obtains its actual Flow through
   `layerm.flow(extrusion_role, actual_height)` for a non-bridge surface.
2. `LayerRegion.cpp:26-28` delegates to that region's `PrintRegion::flow`,
   retaining the first-layer flag. `PrintRegion.cpp:7-23,25-53` selects
   `internal_solid_filament_id` for frSolidInfill, obtains printer
   `nozzle_diameter[extruder(role)-1]`, and narrows it to float.
   In this witness both regions select filament1 and nozzle0.4 mm.
3. `Flow.cpp:129-143` passes that float into Flow unchanged;
   `Flow.hpp:71,128-139` stores/returns `m_nozzle_diameter` as float.
   The exact retained nozzle is `0.40000000596046448`, promoted to double
   when passed to `ConfigOptionFloatOrPercent::get_abs_value(double)`.
4. `Config.hpp:1284-1285` returns `ratio_over * value / 100` for percent,
   otherwise the absolute value. The base is **Flow nozzle**, not first-layer
   extrusion width0.41999998688697815, spacing0.37707960605621338, average
   nozzle, or a new lookup through the output filament selector. The Bottom
   output selector is also1 here, but source flow-role selection is a
   separate operation and must not be substituted by that coincidence.
5. `libslic3r.h:70,93` supplies scale factor1e-6 and division-only `scale_`;
   the explicit `coord_t` cast in Fill.cpp:1279 truncates, not rounds.

| Config read with that same nozzle | absolute mm | scaled double | coord_t |
|---|---|---|---|
| region0:10% |0.040000000596046449|40000.000596046448|40000|
| region1:20%, if projected separately |0.080000001192092898|80000.001192092896|80000|
| absolute0.04 mm |0.040000000000000001|40000|40000|

**Native does not compute two active filler values40000/80000 for this
merged group.** It forms one group, owner0/regions[0,1], then reads owner0's
10% and actually assigns40000 to the single filler. The region1 row above
explains its configured alternative; it is not the selected owner value.

Rust `group_fills/params/projection.rs:152-160` instead computes each region's
transport value before interning, with the same Flow nozzle;
`geometry/coord.rs:49-59` also truncates the scaled quotient. Both arithmetic
results are correct in isolation. The first inconsistent lifecycle point is
`coalesce.rs:122`: comparing those transport values creates two groups,
where upstream interning first merges them and chooses owner0. No percent,
float precision, scaling, nozzle selector or clipping arithmetic fix belongs
in this slice. Retaining the first interned value already supplies40000.

## Native-first input matrix

Reuse the original passing compatible-region overhang input and existing
parameter modifier, with global/local sparse density15%,0.2 mm layers,
classic walls, monotonic Bottom and monotonic-line Top. Ten input-only ZIP
members; all eight non-settings members unchanged. No mesh/placement search,
G-code/preview/golden, new guard or unsupported-feature restriction.

| Case | Global seam_gap | Modifier seam_gap | Official + seeded |
|---|---|---|---|
| equal-density |10%|none|PASS|
| seam-equal |10%|10%|PASS|
| seam20 |10%|20%|DIVERGENT target|
| global-seam20 |20%|none|PASS|
| absolute-equal |10%|0.04 mm|PASS|

Two fresh canonical Native slices per case per mode:20 Native and10 CLI
streams. Repeat/cross-mode input and normalized output identities match.
Global20% controls ordinary seam-gap/perimeter semantics. Absolute-equal is a
particularly important control: Native really admits two distinct configs,
10% versus absolute0.04, yet both Native and Ares form one common fill group.
Thus the target is not simply a generic multi-region/modifier failure.

Target SHA:
`93d3b3ffae0821609f1052388e4f0313e8a7e2fae3ac43657539984537932c08`.
Both modes: first strict byte691/line25, Native `M73 P0 R15` versus CLI
`M73 P0 R16`, normalized sizes167303/190296. First changed motion is the
physical1 Bottom-fill entry, possibly retaining the preceding Outer-wall
TYPE. Keep all progress/feedrate/E/wipe/retraction/time bytes; neither
progress formatting nor perimeter generation is the repair boundary.

This target's input and both stream hashes are unchanged from the b49
observation. It was explicitly excluded from the approved flow-ratio port;
this independent slice does not reopen or broaden that GREEN.

## Source-localized runtime proof

Existing seeded Native/DWARF, CPU12, read-only GDB:

- Before filler creation, target has one Bottom group, owner0/IDs[0,1], one
  merged expolygon. Effective regional seam configs are10%/20%.
- At `FillMonotonic::fill_surface`, after Fill.cpp:1279, actual
  `this->loop_clipping` is40000. The caller's retained Flow supplies the
  exact nozzle/width/spacing above; both solid-role/output selectors are1.
- Ares emits two Bottom/Monotonic groups. Their complete parameter records
  differ only in idx and loop_clipping40000/80000.
- Absolute-equal admits distinct percent/absolute configs and one group in
  both implementations. Its Native filler actually retains40000 as well.
- Successful diagnostic streams match canonical streams. Initial missed
  base-class breakpoint and incorrect namespace for global SCALING_FACTOR
  are preserved as diagnostic failures, not accepted observations. No
  native build, instrumentation or source/test edits were used.

## Included, deferred and staged execution

Included: remove only the loop_clipping discriminator; preserve current
first-owner transport and all other comparator fields; freeze the five
source-authored input cases and use existing shared complete-output harness.
Core remains byte-oriented and UI/filesystem/OpenGL/terminal-free on WASM,
Windows, macOS and Linux. No new public API, configuration or feature flag.

Deferred: moving/recalculating projection, flow-role/output-selector changes,
mixed-nozzle/bridge-specific new witnesses, geometric Flow spacing/mm3 key
questions, concentric kernel/loop trimming, perimeters, narrow-surface/gap
fill/Voronoi, bbox, anchoring length/max, phase, graph, E/wipe/time/progress.
The common comparator affects existing fill families, whose current owner
transport/consumers remain untouched; no concentric or anchoring kernel port
may be silently added to close a residual.

1. SPEC only: approve this exact source/owner/nozzle boundary and five-case
   matrix. No public RED or production change in this commit.
2. After independent SPEC approval, RED only: freeze input ZIPs/SHA manifest,
   reuse shared fresh Native/public API/CLI repeat/complete checks, preserve
   every existing assertion and prior input. Require1 target FAIL/4 controls
   PASS per mode and prior suites unchanged; submit a separate RED RR.
3. After independent RED approval, GREEN only: delete the single comparison
   term, keep RED/harness/inputs frozen. Any strict residual after this bounded
   port means STOP/report, not a kernel/E/time/nozzle/projection/other-key fix.
4. Gates: fmt, locked workspace/all-target clippy `-D warnings`, browser
   wasm32 core+wasm check. Follow the queue; when authorized, rebase onto
   then-current main with range-diff and run fresh canonical official/seeded
   FULL on the exact clean final hash, CPU20–23/threads4, zero FAIL/skipped.
   If main advances, rebase and rerun final-hash acceptance.

No tolerance, M73/E filtering, fallback, new allow/expect/ignore, Native
golden, inverted assertion, unsupported guard, worker merge/push/stash.

Evidence:
`${EVIDENCE_ROOT}/w3-seam-gap-key-spec-review/README.md`.
