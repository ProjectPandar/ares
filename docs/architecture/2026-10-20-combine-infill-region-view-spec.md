# PrintObject::combine_infill: restore a region-local layer view

## Owning upstream rewrite slice

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:4166-4173`,
  `PrintObject::combine_infill`: work on each PrintRegion separately; skip
  only that region if combination is disabled or sparse density is zero.
- `4175-4190`: choose that region's surface kind/pattern and maximum combined
  height, capped by its relevant nozzle diameters.
- `4193-4217`: allocate a new `combine(m_layers.size(),0)` vector **inside the
  region loop**; reset current_height/num_layers there. Walk physical layers,
  skip `layer->id()==0` at4200, flush the lower group's count at4208 and the
  final pending count at4217.
- `4221-4237`: obtain each candidate pair/group from
  `m_layers[i]->regions()[region_id]` at4230, not consecutive records from
  another region's flattened array. `layerms.front()` is the candidate
  group's bottom LayerRegion; its area-threshold call at4237 observes that
  same input boundary before any fill Flow projection or emitted path.
- `4238-4291`: existing multi-layer intersection, clearance and rewriting
  mark combined thickness/layer count on the upper surface and internal void
  surfaces below. Those geometry/math behaviors are not redesigned here.

Rust destination:
`crates/ares-core/src/project_slice/prepare_infill/combine_infill/process.rs`
`apply`, `layer_configs`, `groups`, and their existing record access/mapping
into `combine_group`. Restore a source region's ordered physical-layer view,
reset grouping state for each region, skip its actual planned layer_id0,
and route candidate groups back to the correct region-layer records.

The existing flat `PreparedSurfaceTypeObject.records` remains only a temporary
storage/compatibility shell. It must not define an Ares-owned cross-region
layer sequence. This is a rewrite of the upstream region-loop lifecycle and
indexing boundary, not a new grouping algorithm, infill feature or pipeline.

## First inconsistent intermediate, observed read-only

Existing75-layer authored overhang project, two distinct active regions,
layer height0.2, maximum combination height0.4, sparse15%, no geometry search.

Fresh read-only GDB on seeded Native and the existing ac3cc CLI:

| Region | Native first two candidate bottom layer IDs | Ares first two candidate local pairs |
|---|---|---|
| 0 |1,3|[1,2],[3,4]|
| 1 |1,3|[0,1],[2,3] — first mismatch|

Native breakpoint is `LayerRegion::infill_area_threshold` with a caller-frame
filter for `PrintObject::combine_infill`; PrintObject.cpp:4237 calls it on
`layerms.front()`. Its receiver's actual region ID/layer ID are observed,
not guessed from G-code. The candidates may have an empty sparse intersection
in early solid layers; this is a **candidate schedule**, not a claim that
those first solid layers already get rewritten.

Rust breakpoint is `process::combine_group`; actual flat arguments for region0
are bottom/top1/2 then3/4, count2. For region1 they are75/76 then77/78, count2.
Mapping through the unchanged region-major layout (75 layers per region)
gives local0/1 then2/3. The first bad intermediate is the **second region's
candidate group's bottom layer0**, versus Native bottom layer1.

A second observation at Rust `groups` shows150 flat configurations and
scheduled upper indices74,76,78... . `groups:145` skips global index0 only,
not each region's own first print layer. The native reset at4193-4196 and
per-region `layer->id()==0` test at4200 are missing from that flat schedule.
Source alignment is retained from surface detection/perimeter records;
`perimeters/types.rs:160` maps region/layer as
`region_index*layers.len()+layer_index`. No evidence supports changing
geometry, Flow spacing, volume or cooling to repair this input boundary.

Native's actual nonempty combined upper layers are even in both regions
(first zero-based6 observed). Public output first positive Sparse physical
layer is Native7 versus Ares6. These later observations agree with the earlier
candidate-schedule mismatch; neither progress nor feedrate is the root cause.
All successful diagnostic streams match canonical respective streams.

## Independent Native matrix and controls

Use existing authored overhang inputs/placement/profile, global sparse15%,
rectilinear sparse fill,0.2 mm layers. Distinct-equivalent regions use global
seam_gap10% versus local absolute0.04; its non-combining baseline is already
frozen in `tests/parity/seam-gap-fill-group-key/absolute-equal.3mf` and must be
reused, not copied. The density25% target uses the original parameter modifier.
Single-region control reuses the original without-modifier input, not a
searched or rebuilt mesh. Preserve all non-settings bytes relative to the
matching original single-/multi-region input project.

| Case | Combination / regional distinction | Official + seeded |
|---|---|---|
| combined-sparse | enabled, equivalent-physical seam configs distinct | DIVERGENT target |
| combined-density25 | enabled, local sparse25% vs global15% | DIVERGENT sibling target |
| canonical | disabled, distinct equivalent-seam configs | PASS |
| combined-identical | enabled, identical effective modifier config | PASS |
| combined-single | enabled, no modifier | PASS |
| one-layer-distinct | enabled, distinct regions, max height0.2 | PASS |
| density25-without-combination | disabled, local sparse25% | PASS |

Each input has two fresh canonical references per mode:28 Native streams and
14 CLI outputs for this seven-case matrix. Repeat/cross-mode normalized
identities match. Only generator/date identity is normalized; complete
M73/time/feedrate/E/wipe/retraction remain significant.

Target deltas, identical in both modes:

| Target | first strict byte / line | Native | Ares | lengths |
|---|---|---|---|---|
| combined-sparse |1281 /46|`M73 P3 R14`|`M73 P3 R15`|155436/162659|
| combined-density25 |1306 /48|`M73 P4 R14`|`M73 P4 R15`|164181/165005|

Single-region, identical-config, one-layer gate and combination-off controls
isolate regional grouping lifecycle, not generic combination, density or
modifier loading. The sibling is a target, not a falsely labeled passing
control: it shares the same region-local indexing boundary.

## Included and explicitly deferred

Included: the native per-region layer view/reset/first-print-layer skip,
correct region-local candidate record mapping, preservation of existing
per-region activation/max-height/pattern inputs and surface transport.
No new public option/API, geometry adapter, thread policy or feature guard.
Core stays byte-oriented and UI/filesystem/OpenGL/terminal-free on browser
WASM, Windows, macOS and Linux.

Deferred: the remaining Flow.spacing/mm3 comparator fields (no attributed
Native witness); clearance/scaled-width arithmetic, float rounding, clipping/
intersection kernels, area thresholds, surface rewrite mathematics, bridge
angles/anchoring, bbox/phase/graph, E/wipe/feedrate/cooling/time/progress.
Do not silently port an adjacent difference if the region-view change leaves
a strict residual. New raft/mixed-nozzle geometry witnesses are deferred;
retaining the cited actual layer_id0 predicate is part of this source slice.

## Staged plan and stop rule

1. SPEC only: this exact upstream region-loop/input boundary and the
   seven-case matrix. No production or public test edit in this commit.
2. RED only: freeze the six existing new
   discovery ZIPs/SHA manifest and reuse the already-frozen canonical input.
   This is the third similar case suite: **first extract the shared case-runner
   skeleton** from flow-ratio/seam-gap suites,
   retaining every existing assertion/input and Native/API/CLI raw hash.
   Reuse shared complete-output/producer checks; add only this suite's public
   admission/activity. Require2 target FAIL/5 control PASS per mode and all
   prior suites unchanged, without production edits.
3. Bounded production-only GREEN: restore
   per-region physical-layer scheduling/mapping and actual layer_id0 skip,
   keep the frozen RED/harness/inputs unchanged and geometry/math consumers
   untouched. Any strict residual means **STOP/report**, not expanding into
   clearance/kernel/Flow/G-code/E/time repair or adding a fallback/guard.
4. Gates: fmt, locked workspace/all-target clippy `-D warnings`, core+wasm
   browser wasm32 check. Final acceptance uses a fresh canonical official+seeded FULL with zero
   FAIL/skipped.

No tolerance, output filtering, Native golden, inverted assertion, new
allow/expect/ignore, unsupported guard, legacy fallback.
