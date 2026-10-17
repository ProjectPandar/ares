# SPEC: conditional scarf smoothness and its overhang dependency

Status: **SPEC REVIEW-REQUEST / docs only**. Proposal approved at
`a30db87fbfd9a7955244ac41449df79754271448`, direct parent approved spRear
GREEN9d9814418e1481114d05cb65848ed723869ac2ce. No RED/production permission until
exact SPEC approval. [ARD](ard-scarf-conditional.md).

## Owner amendment: first-object-over-raft Native UB (2026-10-17)

After approved REDbdb36e092, owner authorizes ONE narrow explicit admission error:
`raft_layers>0 && conditional scarf active && scarf_overhang_threshold>0` →
`SliceError::UnsupportedProjectFeature`, key `scarf_overhang_threshold`.
Conditional scarf active means configured scarf type is not None and conditional
is enabled, with the source non-vase gate respected; do not reject inactive scarf.
Check resolved object/region options at the shared scarf-option/admission boundary
so API and CLI, modifiers/defaults/overrides have the same result. Comment cites
Native SeamPlacer.hpp59-70 (uninitialized field), SeamPlacer.cpp1092/1563 and
GCode.cpp5787-5791 (first-object-over-raft consumption). No scalar is fabricated,
no print/object index is changed. All other defined cases remain the source port.
This is an explicitly authorized Native-UB guard, not a threshold-only fallback.
A new RED-side UB witness is NOT required. GREEN gate is ALL seam+scarf+core-seam
families BOTH modes at the exact hash; residual whole mismatch=STOP. Frozen RED
stays unchanged. This amendment supersedes the earlier blanket no-new-guard rule
ONLY for this conjunction; other unassigned-field paths still STOP/report.

## Decision: the adjacent nonzero-overhang predicate is IN SCOPE

Port Native conditional eligibility as one source-owned decision, not the
existing Ares "conditional always off" bypass and not "threshold0 only" support.
The smoothness call is the observed gap. The adjacent positive-threshold guard
cannot be safely deferred in production: removing the bypass and ignoring that
guard would incorrectly enable scarves on unsupported seams; retaining a new
positive-threshold bypass would invent a fallback. Therefore include its minimum
raw-distance producer/selected-candidate handoff/config projection, with its own
Native ACTIVE/INACTIVE qualification before permanent RED. This does NOT authorize
unrelated candidate-score/embedding precision work or a new slicing pipeline.

## Source ownership → exact Rust boundaries

Upstream pin8500fcdccaa10b5099ac20d252af3a7c560046f1, paths below relative to
OrcaSlicer/src/libslic3r. Rust paths relative to crates/ares-core/src/project_slice.

| Native owner | Included source behavior | Rust destination |
| --- | --- | --- |
| GCode.cpp::GCode::extrude_loop5744,5777-5785 | Preserve non-vase/type/hole/role/layer>0 gates; if conditional, use the real smoothness predicate with angle*PI/180 and active nozzle diameter. | gcode_emit/motion/scarf.rs::build and existing motion/loop_paths.rs::emit callsite. |
| ExtrusionEntity.cpp::ExtrusionLoop::polygon162-172, is_smooth369-424 | Concatenate each path's points EXCEPT its final duplicate endpoint; cyclic previous/current/next walks, f64 norms assigned to f32 lengths/arm accumulators, closing-length max0.1, nozzle-length arm criterion and exact signed-angle ternary `(a>0 ? a<angle : a>-angle)` with native strict comparisons. | Private eligibility helper under gcode_emit/motion/scarf/ (source-owned helper split if needed); existing loop/path geometry, no general geometry API. |
| GCode.cpp5787-5791 | Only after active eligibility AND conditional AND configured overhang threshold>0: selected seam unsupported distance must be STRICTLY less than percent*0.01f*the role's effective line width. | Same scarf eligibility helper/build and existing loop_paths::emit/perimeter::emit_perimeter context. |
| SeamPlacer.cpp::calculate_overhangs_and_layer_embedding1083-1092; SeamPlacer.hpp59-70 | Produce distinct candidate unsupported_dist = double signed lower-layer distance + f32(0.4f*candidate external flow_width), assigned f32. No first-layer assignment upstream. | seam_placement/alignment/context.rs::populate_overhang, LayerPlan in alignment.rs, preparation.rs for field lifetime. Private unsupported-distance child helper if needed. |
| AABBTreeLines.hpp::LinesDistancer300-352; ExPolygon.hpp::to_unscaled_linesf175-193 | Signed f64 distance at candidate f32 XY promoted to double; no-line result +infinity; signed distance uses source outside classification. | Reuse geometry::LineDistanceTree::{nearest_mm,outside_mm} and native line order/scale semantics for this NEW raw field, not current f32 comparator-distance helper. |
| SeamPlacer.cpp::place_seam1550-1563 | Return unsupported_dist of the actual chosen candidate, BEFORE inner projection/staggering. For finalized alignment/random use the stored seam_index, not final fitted XY; for nearest use the emit-time selected best index. | seam_placement/placement.rs::place_collection, runtime/nearest.rs::place_nearest_penalized, seam_placement.rs::nearest_seam_layer and island_print_order.rs::NearestSeamLayer. |
| GCode.cpp5770-5791 local seam_overhang lifetime | Carry that read-only result with its source loop into emission; do not re-query at a projected/shifted G-code point. | perimeters/classic/entity_collections/types.rs::OrderedExtrusionLoop's private seam result; gcode_emit/motion/perimeter.rs and loop_paths.rs handoff. No new public API. |
| GCode.cpp m_config at5777-5791 | Region/default conditional flag, angle/overhang thresholds, role-specific effective outer/inner widths, active nozzle, all source precedence. | motion/options.rs::ScarfOptions and motion/options/build/scarf.rs::from_region; reuse existing MotionOptions nozzle/width context. |

Existing producers/association and chosen indices are authoritative; no new
nearest lookup or candidate producer. Required seam metadata may not be silently
lost by the existing clockwise-rectangle runtime shortcut. `runtime::placement_modes`
and `apply_objects` are in scope ONLY for source-required context availability:
retain native selection identity/physical planes and existing geometry/placement
results, not an independent rotation/shortcut rule. If satisfying the handoff
requires a distinct geometry/scoring prerequisite, STOP/report instead of
absorbing it into GREEN. Nearest selection itself, random arithmetic, AlignedBack
visibility bias and spRear comparator/fit remain frozen.

### Precision, lifetime and UB rules

- Unsupported distance is NOT the nonnegative/clamped overhang penalty and NOT
  embedded distance. Do not invert a penalty, substitute zero, or add epsilon.
  Keep f32 candidate storage, promote for signed distance, retain f32 0.4*width
  before double addition and final f32 assignment. Existing penalty/embedding
  helpers and seam choices are not changed by adding this field.
- Use one optional candidate-distance vector per OBJECT physical layer (absent
  at object data-index0, present for later object layers); carry an optional
  selected scalar with the prepared loop. Preserve source print-layer/type/
  conditional/smoothness/positive-threshold short-circuit order. In the frozen
  no-raft domain, data-index0 also has print layer_id0, so Native never reads
  its uninitialized field. Do NOT assume this identity globally: GCode.hpp671
  reads Layer::id, while PrintObjectSlice.cpp57 starts IDs after raft layers.
  Any currently admitted source path that can consume an unassigned raw field
  must STOP/report before GREEN, not acquire a fabricated zero/new guard or
  silent scarf disable. This does not authorize raft/support or UB-case work.
- For an empty preceding distance tree, +infinity is the explicit upstream
  AABBTreeLines.hpp333-337 result, not a new defensive fallback. Preserve the
  strict source predicate (infinite unsupported distance cannot pass it).
- Role width in5788-5789 is configured outer/inner `get_abs_value(nozzle)`, not
  actual calculated extrusion-flow width or candidate external flow_width.
  Reuse nozzle context; project configured widths separately if existing emit
  widths have already resolved auto-width0 into a nonzero flow width. Source
  configured0 stays0 here. Preserve mixed float/double evaluation and strict
  `<`; zero percent short-circuits exactly as source specifies, not as a
  special supported-domain feature flag.
- No initialized-oracle substitution, blanket UB-guard removal, threshold-only
  runtime support, diagnostic core I/O, new public schema or OS dependency.

## Included and deferred

Included: exact smoothness helper, conditional configuration, raw unsupported
producer and selected-index/loop lifetime, both eligibility predicates, source
activation order; reuse existing scarf construction and quantized XYZ/E writer.
Existing construction is a temporary compatibility shell only at the eligibility
entry: replace its unconditional conditional-returnNone with the upstream decision.

Deferred: scarf ramp/path/clip/speed/flow redesign, seam scoring/geometry/visibility/
embedding precision and alternate point-in-polygon policy for THEIR existing
fields, new random/nearest placement behavior, painting, support planning, travel,
arcs, M73/time/cache changes, viewer/libvgcode. Reuse geometry's existing f64
query/outside APIs for the raw field; do not globally rewrite geometry or change
old penalty/embedding results. Changed Rust files≤400LOC; split source-owned
private eligibility/distance helpers rather than growing the current scarf file.

## Actual approved discovery (not formal RED)

Exact production base9d981, fresh official Native FIRST + same-hash real bare
CLI, original closed6-vertex/8-triangle10x10x0.8mm prism,4 layers/one object/part/
filament/Classic2walls, aligned, support/brim/tower off. External scarf, inner0,
entire0,start10%,length10,steps10,angle0,overhang0%.

- Control conditional0 SHA0e2448b51db7003392c83e123aed4d1340d49eecbee2b725c4ca5bf12fd842b1:
  Native0/CLI0/complete whole generator-only strict PASS.
- Positive conditional1 SHAb89abfb6df717f88a513723f469c966577c7361d789f623dcd3e9feddca4849d:
  Native0/CLI0/complete whole strict FAIL451/line18/column9,38646/33780bytes,
  M73P0R2 versusP0R1. First motion91 Native raw181
  `G1 X109.964 Y114.425 Z.22 F9000 ; move to first perimeter point`
  versus Ares raw183 `G1 X110 Y114.497 F9000`.
- ONLY seam_slope_conditional0→1 differs (decoded setting and ZIP-entry proof).
  Native control/positive full motion sequences identical,48 XYZE/Z-extrusion
  moves versus0 in Ares positive. First Native slope extrusion
  `G1 X109.72 Y113.938 Z.23 E.0027 ; perimeter`. Both config footers echo settings.
- Angle0 is in declared0..180 range (PrintConfig.cpp5546-5556). Nondegenerate
  loop arms/nozzle0.4/positive10steps10mm/start0.1; optional overhang read disabled
  by0%. This qualifies smoothness, NOT the newly included nonzero dependency.

History/queue and prior-port/UB audit remain in the approved proposal. This SPEC
adds no claim that new taper inputs or API/seeded executions have happened yet.

## SPEC → qualified frozen public RED → separately authorized GREEN

1. Exact SPEC REVIEW-REQUEST (these docs only), then await approval. No permanent
   test/fixture or production change in this phase; no Native launch required.
2. After approval, fixed nonzero-dependency qualification budget: FOUR Native
   commands, official+seeded low/high pair, before permanent RED. New sibling of
   the SAME6v/8triangle prism: retain bottom vertices, scale top XY by1.15 about
   base triangle centroid (z unchanged), four layers and all original settings;
   conditional1, angle0; ONLY overhang1% versus100% differs. Freeze both complete
   inputs/ZIP-entry SHAs BEFORE Native. Expect low Native-INACTIVE (0 slope XYZE)
   versus high Native-ACTIVE (>0). All other Native motion/role/context settings
   equal; changing only threshold proves eligibility attribution without
   uninitialized read or debugger-patched oracle. Require Native0, complete
   footers, no missing output. If activity split/acceptance does not occur,
   STOP/report: no parameter/mesh search, repeated Native, threshold invention
   or silent zero-only implementation. Actual observed counts/first blocks freeze
   into the later RED; no fabricated numeric expectation.
3. Freeze FIVE public cases and exact manifest at a production-unchanged RED hash
   for review:
   original conditional0 control; original conditional1/angle0 positive;
   original conditional1/angle155 sharp negative control; qualified taper low1%
   inactive control and high100% active dependency positive. Retain original
   positive/control byte inputs unchanged. Native ACTIVE assertions on original
   control+positive (48 slope moves and recorded first block), taper high (>0
   with observed first block); INACTIVE on sharp155 and taper low (0). Role/layer
   eligibility and first-layer inactivity must be checked, not just echo flags.
4. Formal RED budget TEN fresh Native requests, five per mode, official+seeded.
   Each Native FIRST then public slice_project_plate_validated(default overrides,
   seed0) AND real bare CLI on IDENTICAL bytes. Persist both public calls and
   admission/complete streams, BOTH entire strict comparisons and Native activity
   receipts BEFORE success/raw/target assertions. Collect all five case results
   before final target assertions so first expected positive failure does not
   suppress the second dependency witness. Controls require strict PASS; active
   positives genuinely FAIL target equality until GREEN, not expected-error PASS.
   If any control fails, Native activity premise changes, an API/CLI admission
   fails or a stream is missing: STOP, NOT EVALUATED where appropriate, no RED
   laundering or input replacement. Strict comparisons retain M73/motion/arcs/
   statistics/config/footer, unchanged golden.rs normalization only.
5. Exact RED REVIEW-REQUEST and separate GREEN authorization. Port ONLY the cited
   source decision/helper/raw-data handoff; frozen RED unchanged. Both active
   positives and all controls must become complete whole PASS/raw API==CLI in
   both modes with Native ACTIVE/INACTIVE states preserved. Residual whole
   failure after bounded port means STOP/report, not a clock/geometry fix.
6. Exact GREEN REVIEW-REQUEST: targeted existing scarf/slope/PA/seam families,
   including nonconditional and sharp-corner controls, previous physical-plane/
   spRear; audit actual Native gates/outcomes (not nominal skips), scoped fmt,
   strict core+CLI-tests clippy, core+adapter WASM, changed Rust≤400LOC. Browser
   WASM/Windows/macOS/Linux logic remains byte-API/platform-neutral. Coordinator
   owns full/workspace scheduling/integration; RED merges ONLY with GREEN.

All execution CPU15/jobs1/nextestthreads1; fresh per-mode/per-phase cache/artifacts/
TMPDIR; load<60 before builds/EVERY Native, START/END receipts and complete command/
status snapshots; complete each mode's qualification/comparison audit before the
next mode; no Native retries-to-success. Qualification4+formal10 is an
explicit budget, not permission for broad discovery/full tests. Preserve all
approved spRear ancestry; no stash/rebase/merge/push authorization implied.

Discovery report /home/indexyz/.ares-evidence/worker4/scarf-conditional-slice-proposal-review.md;
root seam-deferrals-proposal-9d981-Sap6, archive SHA
173c422497330ecf7e9edab55e466b081da9b3147d8cf5897e4ce862c50e531f.
