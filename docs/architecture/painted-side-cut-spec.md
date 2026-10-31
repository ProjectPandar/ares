# Painted side-layer cutting: approved SPEC / public RED

Status: COORD approved the source boundary and authorized a new branch from
`1045b17757156007156440257f18a0d9f3f5afe9`: docs-only SPEC commit first, then public
success-required RED. **No GREEN, production changes, FULL, merge or push.**
Original frozen painted branch/worktree remain unchanged. #90 integration stays
separately gated on the explicit W1 turn.

## Upstream owner, GUI surface and priority

Pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`, paths under `OrcaSlicer/`:

- `src/libslic3r/MultiMaterialSegmentation.cpp:1144–1168 cut_segmented_layers` is
  the single rewrite boundary; invocation/gate2169–2170, caps2174–2179 and
  merge2181, wrapper2197–2207. Width and segmented-region depth select one
  upstream operation, not independently designed Ares features.
- `src/slic3r/GUI/Tab.cpp:2690–2694` exposes both options in Multimaterial Advanced;
  `ConfigManipulation.cpp:987–993` shows segmented-region depth with beam off.
  `src/libslic3r/PrintConfig.cpp:4206–4224` defines advanced mm/coFloat/min0/default0.
- `src/libslic3r/PrintObjectSlice.cpp:874–886` consumes segmentation into actual
  layer/region ownership;1206–1224 applies MMU after conical overhang. Beam is a
  separate later generator1243; compensation1250–1254 retains painted XY suppression.
- `src/libslic3r/libslic3r.h:93` owns double scale division;
  `ClipperUtils.hpp:19,27,346,449` owns Miter/3.0 offset and no-safety-offset difference.

Priority: complete this small missing consumed side producer before considering
painted+beam's separate InterlockingGenerator composition. GUI reachability is
source-cited; discovery executed the same embedded settings through Native CLI,
not an interactive GUI session or other-platform runtime qualification.

## Required source semantics

1. Effective per-object option double→float millimeters→double `scale_` division→
   float scaled radius. Preserve both float casts and evaluation order; do not
   replace them with one f64 scale, add rounding, or clamp depth to width.
2. MMS2169 calls the cut when width>0 OR depth>0 and beam is false. Source1156
   selects depth on even **zero-based object layer indices** when depth!=0;
   otherwise width. This is not a Z-based/global-output-layer parity selector.
3. Source1158 leaves original vectors untouched when selected radius is not
   positive. For positive radius retain all state slots including0 and empties;
   for each nonempty side state subtract the inward offset of the **whole prepared
   layer input**, preserving holes, order and default flags. Do not erode colors
   independently or use a bbox/raw-mesh/recomputed-volume proxy.
4. Cut sides before top/bottom generation and cap-side merge. Preserve existing
   projection/line coloring/repaired VD/lookup, facet and occurrence identities,
   strict/ordinary exports, source casts, six-filament common registry, ordered
   self-trim/ownership, actual apply, compensation and painted XY suppression.
5. **Do not implement tooltip ignore conditions** from PrintConfig4217–4219:
   source1156/2169 accepts depth-only and depth>width, confirmed by both Native
   modes. MMS1151's unused `max(width-depth,0)` temporary must not be activated
   or ported as operative behavior.

## Rust destination / temporary scaffold

Future implementation, requiring separate GREEN authority:

- Private `crates/ares-core/src/project_slice/region_slices/painted/cut.rs` ports
  that helper over existing prepared layers and side-state vectors. Borrow input,
  mutate/move side output, reuse existing equivalent Rust offset/difference kernels;
  no new crate/backend/public API, copied engine, fallback or dormant consumer.
- Existing `painted/mod.rs::apply_project` obtains effective `ObjectOptions` from
  `ResolvedProjectObject.object`, consumes the cut directly between `sides(...)`
  and `top_bottom::segment(...)`, before `merge::merge(...)` and real apply.
- Existing `painted/apply.rs::ObjectRegions::new` loses **only** its positive
  width/depth unsupported branches once that real consumer exists. Its present
  omission/guard is a temporary compatibility shell around the cited upstream
  operation, not an Ares-owned pipeline design. Do not recreate registry lookup.

Included: width and/or segmented-region depth, beam=false, state0/empty retention,
source parity/conversions and actual pre-cap consumption in the existing supported
MMU domain. Deferred: painted+beam/InterlockingGenerator, fuzzy painting, UI/cache/
cancellation framework expansion/SVG/debug exporters, new geometry engines.
Keep unpainted interlocking and ordinary fuzzy options. Modules remain≤400LOC,
portable Rust with no core filesystem/UI/terminal/native runtime dependency.

## Minimal sufficient public RED

From the eight-input sealed discovery choose three independent target3MFs:

| Target | Width/depth | Purpose |
|---|---|---|
| split-w1-d0 |1/0|width alone; comparison anchor for depth changes|
| split-w1-d1p5 |1/1.5|depth>width even-index selection, not tooltip ignore|
| split-w0-d0p5 |0/0.5|depth-only gate, not tooltip ignore|

Control coverage belongs solely to existing
`paint_color::paint_color_split` and its existing
`paint_color_native_preflight`, with the original frozen painted-split
archive/preparer/manifest (width/depth0). The new suite qualifies only its three
targets; it contains no CONTROL, copied preparer or control input branch.
No copied control3MF. Only three target3MFs are added as fixture data: no new
preparer/manifest/G-code/Native outputs. They are byte-identical to discovery;
only the two selected settings differ from the frozen split base. This isolates
width separately from depth>width without adding redundant nominal/equal-depth
or unpainted cases. Input hashes and source citations live beside the tests.

Public seam: existing `native_stream_case::check` → full ordered `NativeCase`
comparison against both API and actual CLI, with raw API==CLI unchanged. All
three new tests require success/output equality; **never assert Unsupported**,
should_panic, normalize order/coordinates, loosen tolerances or invert failure.
Current guard is expected to cause target FAIL; reused control must PASS.

Fresh two-mode Native-only preflight runs the existing paint_color preflight
(its three original inputs) plus the new suite's three-target preflight, with
reference+replay before **any** Ares leg:24 actual Native PIDs globally. Control
is not qualified again inside the new suite. The public four-leg matrix per mode
invokes fresh reference+replay again:16 more PIDs,40total.
Retain original generator-only allowance. New-target activity acceptance is only
positive_model_XYE>0 for both tools, setting-line presence and reference/replay
activity equality: no fixed move/layer counts or Native-derived goldens. Existing
paint_color acceptance remains unchanged. These witness checks do not replace
strict whole-output comparisons.
Any unexpected Native/control/infrastructure result is STOP/report, not a retry
to PASS or production/comparator change.

Discovery's32 Native/8CLI executions were not public RED or whole NativeCase
acceptance. Final odd-layer activity also changed in depth cases; do not assert
even-only final G-code or claim private cut geometry was captured. Linux execution
and wasm32 compilation do not establish Windows/macOS/browser runtime parity.

## Verification and disposition

Before/after commit audits preserve original1045 branch/tree, production, frozen
fixtures/harness/comparator/controller, historical seals and original qualified
binaries. Required fmt, locked/offline workspace checks, strict all-target Clippy,
wasm32 core/vgcode/wasm and list/build gates bind the exact RED tip. Test execution
uses cargo nextest on CPU18/jobs1/nextest1/parity1/CARGO_INCREMENTAL=0. Record
filter exclusions and internal guards separately. No FULL or GREEN in this stage.

Exit now: docs-only SPEC/ARD/roadmap commit followed by independent public RED
commit; both modes each3target FAIL/1control PASS with retained complete API/CLI/
Native artifacts and RED-RR for review. Future GREEN/integration/FULL need new
explicit authorization. See [ARD](ard-painted-side-cut.md).
