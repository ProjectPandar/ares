# AdaptiveCubic configuration spacing and float octree bounds

Status: SPEC proposal; production not authorized.
Base: 6e0349d9f6da4c0dfdae3cedb26ef9cc0a98e412 (stack-7).
Upstream: OrcaSlicer 8500fcdccaa10b5099ac20d252af3a7c560046f1.
This supersedes the spacing-only Slice A acceptance boundary in SPEC59762910d.
The old cd9d5e0f2 test candidate is archived and will not be merged or reused
with its SupportCubic legs. No new production or tests are included in this doc.

## Ruling and proven selection

The coordinator narrowed the slice after runtime attribution proved A45's full
output closes only when configuration spacing and float bbox precision are
combined. Included pattern is AdaptiveCubic, one printing region per object.
SupportCubic is a separate Slice B, including its unresolved S45 failure.

The owning source boundaries are adaptive_fill_line_spacing
(Fill/FillAdaptive.cpp:276-354), its PrintObject::prepare_adaptive_infill_data
consumer (PrintObject.cpp:977,1006-1007), and build_octree's float mesh bbox
(FillAdaptive.cpp:1495-1497). Existing Ares per-fill spacing and double bbox
construction are temporary compatibility shells around these unported numerical
boundaries; this is not a new Ares-owned slicing pipeline.

Inbox audit: no competing AdaptiveCubic numerical port. W2 owns ZAA/Polyline3,
W3 mesh-edge/first-layer collection order, W4 counterbore, W1 skirt/draft shield,
W5 enum import, web OptionsZCorrector. Their behavioral boundaries are excluded.
W2's mechanical fill carrier work can affect types/projection signatures; rebase
and preserve that layout before production, report any structural adapters before
committing them. Do not touch anchor_projection.rs or surface pattern admission.

## Native-backed attribution, before RED selection

Existing-binary CPU11 experiments on exact public candidate inputs are retained
outside the repository:
- /tmp/worker6-adaptive-spacing-red.mUVyz6: original runtime Native official and
  seeded, public validated API/CLI comparisons. Native accepts all requests.
- /tmp/worker6-adaptive-frame-attribution.3Q7vMQ/exact-report.json: A45 and S45.
- /tmp/worker6-adaptive-frame-attribution.3Q7vMQ/variants-report.json: A50/A125/AAUTO.

Native was never altered or rebuilt. The existing Ares process received only
independently observed Native spacing, bbox max size and bbox center, applied to
both actual allocated root center and origin before triangle insertion. Mesh
vertices, filtering flag, bridge triangles, Flow, ordering and comparator remain
unchanged. Identical generated-by normalization is followed by complete-byte
comparison, including every footer line and exact ordered line count. Diagnostic
parameter interventions establish attribution; they are not production GREEN.

| New RED witness | Requested width | Original public sparse first difference, both modes | Combined intervention |
| --- | --- | --- | --- |
| A45 | 0.45 | layer4 line1436, E1.31586 Native / E1.31587 Ares | Whole bytes equal to original official, original seeded and fresh Native |
| A50 | 0.50 | layer6 line1739, E.11164 / E.11163 | Whole bytes equal to all three references |
| A125 | 125% of nozzle0.4 | Same first difference as A50 | Whole bytes equal to all three references |
| AAUTO | 0 (automatic) | Same first difference as A45 | Whole bytes equal to all three references |

All four use count1/density30/prism with one region. Each witness is admitted
and independently closes under the same two-boundary intervention. No unclosed
width/count/SupportCubic case is placed in the new RED.
S45 still differs at layer4 line1418 E.04926/E.04925 after the same intervention;
retain it as Slice B evidence, never label it PASS or silently discard it.

Earlier diagnostics are retained with limitations: spacing alone failed; a local
center assignment updated origin but not the already-copied allocated root;
an exploratory input with a different combination-height setting had a footer
mismatch. Final reported closures use exact RED inputs and verified actual root
state; no footer normalization or shape-only comparison was introduced.

## Exact upstream type and arithmetic chains

### Configuration spacing: FillAdaptive.cpp:276-354

- :296-309: maximum nozzle diameter is double. Configured sparse width resolves
  get_abs_value(max_nozzle_diameter) as double; preserve percentage arithmetic.
  Exactly zero width uses Flow::auto_extrusion_width(frInfill,
  float(max_nozzle_diameter)), returning float then widened to double.
  Flow.cpp:21-36 uses the f32 product 1.125f * nozzle_diameter.
- RegionFillData width/density are double (:287-292/308). Native's general
  function aggregates actual contributing regions (:315-340), dividing each
  sum by double count (:344-347). One-region acceptance reduces to that region's
  exact config values; do not use rounded extrusion Flow.width or fill density.
- :348: density/100.0f is double because density is double; 100 is exact.
  0.333333333f rounds to f32 (0.3333333432674408), then promotes to double.
  Complete denominator multiplication, width division and final multiline
  multiplication (:352-354) are double, in source order. No reciprocal rewrite.
- Native's explicit printing_region(0) multiline policy does not license taking
  the first element as a substitute for density/width aggregation. The included
  one-region count1 witnesses have unambiguous region identity. Multi-region
  aggregation and selection remain a separate slice; do not add a new guard.

Observed spacings:
A45=4.4999998658895537, A50/A125=4.9999998509883925,
AAUTO=4.5000000447034827. Current Ares widens Flow.width and uses a f64 one-third
literal, yielding4.4999998852907108 /5.0000000050000004 respectively. Constant
promotion alone does not restore the configured width's lost bits.

### Float frame bounds: FillAdaptive.cpp:1495-1497

Native receives a Vec3f mesh and constructs BoundingBox3Base<Vec3f>.
BoundingBox.cpp:182-185 size() is max-min in f32; :232-235 center() is
(min+max)/2 in f32. Only the completed center and maxCoeff(size) widen to f64.
Ares currently reduces f64 vertices, computes center/size in f64 and retains
these values in root center and origin (fill/adaptive/octree.rs:137-159).

For these inputs, Native widened center is
[1.1546878814697266,1.1547050476074219,1.1547060012817383], versus Ares's
[1.1546894319625274,1.1547060915475527,1.1547060915475544].
Native maximum size67.160446166992188 versus Ares67.160442681381397.
Each actual Native float vertex captured here equals the source-rounded Ares
rotated coordinate; float bbox projection must preserve that cast boundary,
float min/max, float sums/subtractions and final widening. Do not globally cast
triangle vertices or alter triangle insertion in this slice.

## Rust destination and minimal wiring

1. project_slice/fill_entities/adaptive.rs::append and ::anchoring_lines:
   select the source spacing only for AdaptiveCubic. Keep SupportCubic's current
   path intact. Both Adaptive consumers must receive the same exact unrounded
   configuration inputs and arithmetic.
2. Private adaptive/line_spacing.rs if needed. The applicable RegionOptions
   must come from the actual source region identity. Where current SurfaceFill
   retains only f32 width/density, preserve private source spacing metadata at
   group_fills/params/projection.rs and group_fills/types.rs using the existing
   LayerContext.region/nozzles (full f64 values), rather than reconstructing
   values from Flow or using regions[0]. No new public API.
3. fill/adaptive/octree.rs plus private octree/float_bounds.rs if needed:
   use source f32 bbox calculations for the AdaptiveCubic construction only.
   Feed the completed center consistently to root and origin, and the widened
   f32 max size to the existing cube-property ladder. Cube expansion, frame
   rotations, triangle tests, child ordering and filler hooks are unchanged.
   Preserve the old SupportCubic builder entry and its numeric behavior; share
   construction without duplicating the full octree implementation. If this
   requires mechanical extraction, isolate it and prove exact reconstruction.
4. fill/adaptive/mod.rs existing per-slice cache: distinguish Adaptive and
   Support construction identity if they could otherwise alias the same object
   and spacing bits. This is the existing cache and source two-tree ownership,
   not another cache framework; no global state or runtime feature flag.

Every modified/new file <=400 LOC. No UnsupportedProjectFeature guard, test-only
production hook, fallback, ignored test, suppressed lint, or new external crate.
No SupportCubic production changes or guard/assertion reversal is authorized.

## Included and deferred

Included: AdaptiveCubic count1 single-region configured width/percent/auto and
f64 density input, source f32 one-third promotion, float bbox center/max-size,
actual source metadata delivery for emission and anchoring, existing cache
identity separation where necessary.

Deferred: SupportCubic filtering/bridge triangles (PrintObject.cpp:1006-1007 /
FillAdaptive.cpp:1512-1521), multiregion averaging/presence and multiline policy,
count2 geometry/order, global mesh transform precision, arbitrary triangle
insertion precision, ZAA/Polyline3 behavior, shared clipping/offset, other fill
patterns and support ironing. Do not add admission restrictions for those paths.
Any residual requiring another boundary is reported before widening scope.

## New RED, GREEN and gates

After SPEC approval create a new public CLI E2E module, with four Adaptive RED
legs above and Rectilinear/AlignedRectilinear count1 full-stream controls. Runtime
OrcaRunner export/slice, fresh cache/root, Native exported settings verification,
public validated API plus CLI whole ordered-byte equality, line counts and layer
schedule. Only authored inputs go into git; Native captures stay outside it.
Do not cherry-pick cd9d5e0f2 or retain its Support test registrations.

Commit RED separately, run both modes on exact clean HEAD, report it for approval
before production. Then independent GREEN with the same fixtures/assertions;
none of the debugger substitutions become acceptance behavior. Report any
non-RED raw output changes and preserve SupportCubic control raw bytes.
Final exact HEAD: CPU11 jobs1/threads2, directed+whole option_coverage both modes,
strict clippy/fmt/wasm32. Default workspace/CI/full only on coordinator assignment;
full requires FULL-GO. No push without exact-hash authorization.
