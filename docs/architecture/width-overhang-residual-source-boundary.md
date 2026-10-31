# Width/overhang residual: source boundary and RED contract

## Authority and current scope

Authority: OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`, with its Clipper2 dependency
`09f1baceae892df95ea07250dbb6b6d3dedd3d2b` (2.0.1).

This follow-up starts from the accepted B12-restacked geometry tip. It is a
source-cited rewrite slice, not an independently designed slicing or movement
pipeline. No production change is justified merely because the retained test
is grouped under clipping geometry.

The unchanged input is
`tests/parity/clipper-wrapper-geometry/coincident-width-overhang-intersections.3mf`,
SHA256 `7688c5eaac8bb610fae4e92863474c85262d9f83f64737ba17f5574c6b635866`.
It is one model part/instance with a stepped silhouette, Arachne variable-width
perimeters, overhang detection, one wall and no brim/skirt/support/infill/shells.
Do not modify this input or the original seven full-stream comparators.

## Public symptom and source candidates

The existing public validated API/CLI test reproducibly diverges inside the first
object layer that emits overhang walls. It exposes an extra straight-edge split,
a different inner-corner point and a later fan/progress difference. The corner
point difference also appears in the following supported object layer. This does
not prove one common cause or assign every difference to the clipping engine.

| Candidate owner | Rust destination / discriminating observation |
| --- | --- |
| `PerimeterGenerator.cpp:320–389::clip_extrusion`, `:407+::traverse_extrusions`; `ClipperZUtils.cpp:48–80::clip_zpaths`; `Arachne/utils/ExtrusionLine` conversion | Existing `geometry/clipper/{z,z_clip}.rs`, Arachne `traverse/overhang.rs` and `variable_width.rs`; distinguish input junctions, clipped width-tagged paths and constant-width emission paths |
| Cited ClipperUtils/Clipper2 offset and Arachne wall-generation owners reached by the same input | Existing geometry/offset and Arachne modules only if the mismatching corner is already present before emission; no speculative engine correction |
| `GCode/ExtrusionProcessor.hpp:50–355::estimate_points_properties` and `:541–719::ExtrusionQualityEstimator::estimate_extrusion_quality`, with their `GCode.cpp::_extrude` caller | Existing `gcode_emit/motion/overhang.rs`, `overhang/sections.rs` and responsibility-sized children; distinguish prepared path points from newly generated processed points |

Read-only source comparison already finds a concrete beta estimator gap: the
pinned template accepts effective distance-to-speed and overhang-fan predicates,
performs interior discovery, and retains cuts only for source speed/fan changes.
The current Rust section builder instead uses the older distance-only splitting
contract and has no equivalent callback/fan-aware discovery/filtering. This is
an identified source gap, **not yet causal proof for every public difference**.
The earlier stable overhang-investigation record does not establish beta parity.

Ranked predictions before experiments:

1. If width-tagged clipping/conversion owns the first split, that junction is
   already present in the path reaching the emitter; estimator-only changes
   cannot be claimed as the complete repair.
2. If offset/Arachne construction owns the corner, the mismatching corner is
   already present before quality estimation, including the supported layer.
3. If beta quality estimation owns a difference, its input lacks that generated
   point (or the source predicate rejects the old split), and the discrepancy
   appears between estimator input and processed output.

## Non-negotiable production boundary

Follow the owning source after localization. Existing Ares containers/adapters
are temporary compatibility shells around those source concepts, not authority
for new behavior. Preserve source coordinate casts, float/double domains,
strict comparisons, scan order, flow/width identities and role/fan semantics.
No tolerance workaround, endpoint canonicalization, constant-width guess,
fixture branch, Clipper1/native fallback, or parallel substitute algorithm.

If the estimator gap proves causal, include the source effective-speed cap and
fan predicate transport, interior discovery and cut-retention behavior that the
qualified input reaches. Do not repair it by filtering emitted moves or fan
commands. Adjacent curvature/curl families, precompute memoization, other print
strategies and unrelated configuration/feed/progress/statistics remain deferred
unless this public witness demonstrates their owning boundary is required.
Do not treat an upstream pure memo as independent slicing logic.

## Approved RED-first seam and localization contract

The already agreed seam remains the validated public API and CLI against the
runtime official beta producer, stable double replay and the unchanged strict
whole-stream comparator. Keep that original case RED until it actually passes;
footer/config residuals are not erased.

The approved additional seam compares the complete
first Native object-layer block containing the overhang-wall role, including its
initial travel, all moves/E/feeds/M73/fan/width/role metadata. Select the Native
block ordinal, then compare the API block at that same ordinal; never search the
API for a later convenient matching feature. Expected bytes come only from the
runtime oracle, not source-generated expectations or committed output fragments.
Use only the same immutable input and existing native-stream grouping, not ROOT.

A bounded Ares-only read-only debugger capture or one tagged trace is approved,
local only and never committed. Check offset/Arachne construction and ClipperZ
conversion before attributing the persistent corner mismatch to emission. The
capture may inspect variable-width clipping and estimator input/output boundaries for the two affected layers, never instrument Native. Capture
full API before/after, require diagnostic byte identity, restore probe sources
byte-for-byte and remove every tag before production commits. Diagnostic points
cannot replace the official public RED or establish GREEN.

Freeze the qualified focused RED before production code. Acceptance must show
its complete block byte-equal to Native and changed complete API bytes, while
retaining and reporting the original full-stream result and all foreign-owner
residuals. Run only the directed owned cases and required fmt/strict all-target
Clippy; no FULL/broad or beta-ledger baseline rerun.
