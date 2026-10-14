# FillAlignedRectilinear sparse multiline dispatch

Status: SPEC proposal; no Native admission or RED claimed.
Base: `104d8497097dc7239ad63f7a69f3b418b699b3e8` (stack-6).
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Selection and ownership

The merged Rectilinear slice explicitly deferred AlignedRectilinear inheritance
(`fill-rectilinear-multiline-dispatch-spec.md`). Source inspection now identifies
its remaining dispatch gap: Native constructs FillAlignedRectilinear, which
inherits FillRectilinear::fill_surface and overrides only the layer angle;
Ares enables the existing multiline implementation only for Rectilinear.
This is a source-derived candidate, not an observed whole-output difference.
Fresh Native acceptance and a real sparse first difference are prerequisites.

Inbox ownership audit after FULL 6 found no AlignedRectilinear multiline claim.
W2 owns anchor projection/ZAA, W3 first-layer mesh geometry, W4 counterbore,
W1 skirt and W5 enum import; web owns processor OptionsZCorrector. Their files
are excluded. The prior shared polylines extraction is already merged.
Triangles is also merged; its approved branch remains unchanged.

## Upstream boundary

| File / function / lines | Included semantics |
| --- | --- |
| `Fill/FillBase.cpp:50-53`, Fill::new_from_type | AlignedRectilinear constructs FillAlignedRectilinear, not FillMonotonic. |
| `Fill/FillRectilinear.hpp:38-46`, FillAlignedRectilinear | Inherit fill_surface unchanged; `_layer_angle` returns zero. |
| `Fill/FillRectilinear.cpp:3386-3399`, FillRectilinear::fill_surface | Full density (`density > 0.9999f`), count1 and Zag patterns retain by-lines; other sparse count>1 calls one unshifted multiline sweep. AlignedRectilinear inherits this branch. |
| `Fill/FillRectilinear.cpp:2996-3045`, fill_surface_by_multilines | Existing offset, line generation, multiline expansion, half-spacing contraction, ordered clipping and connect-or-chain implementation. Reuse the merged port; do not replace its geometry. |
| `Fill/FillBase.cpp:275-319`, Fill::_infill_direction | Preserve bridge angle precedence, f32 angle additions and object reference; AlignedRectilinear has no layer-angle alternation. |
| `Fill/FillBase.hpp:50-52`, FillParams | Preserve full-density threshold and anchor_length_max<0.05 dont_connect semantics. |
| `Fill/Fill.cpp:926` | Regional fill_multiline only applies to InternalInfill; other roles use count1. |
| `Fill/Fill.cpp:1281-1303,1354` | Normal emission uses the configured filler and existing fill parameters. |
| `Fill/Fill.cpp:1415-1418,1440-1450,1477-1498` | Sparse anchoring constructs the same aligned filler, passes multiline and invokes fill_surface. |
| `PrintConfig.cpp:231,2996-3002` | Existing alignedrectilinear spelling and fill_multiline integer1..10, default1; no schema or inventory change. |

## Rust destination and included behavior

`crates/ares-core/src/fill/rectilinear_multiline.rs::enabled` currently accepts
only ProcessInfillPattern::Rectilinear. Extend that private source-dispatch
predicate to include AlignedRectilinear, only for sparse count>1. Its existing
fill_surface implementation already respects fixed_angle and bridge precedence.
`project_slice/group_fills/params/projection.rs:102-108` already sets
AlignedRectilinear fixed_angle, preserving the upstream zero layer rotation.

Both existing consumers use this predicate:
`project_slice/fill_entities/monotonic.rs:111-123` for normal emission and
`project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/linear.rs:68-82`
for anchoring. No caller change is planned; no edit to anchor_projection.rs.
The current by-lines adapter is a temporary compatibility shell only for the
unported aligned sparse count>1 branch; count1/full-density paths remain intact.
If RED attribution requires other behavior, stop and request scope approval.

Deferred: Line/Zag dispatch, ZAA/Polyline3, bridge classification, new angle or
reference-point behavior, shared offset/union/numerics, fill schema changes,
other patterns and support/ironing changes. No extra bounding-box plumbing,
public API, cache framework, fallback, or new admission guard.

## Native-first RED plan

Use CLI E2E with existing authored prism and holed-prism STL inputs, runtime
3MF fixture adapter and OrcaRunner slice_case cache. Freeze settings per case:
single nozzle/filament, by-layer, no tower, centered object, ordinary fixed
layer height, no ZAA. Verify the Native exported configuration preserves
alignedrectilinear, fill_multiline, requested angle and density. Compare Native
with both public validated Ares API and CLI using unchanged complete ordered
bytes, generator-only normalization, line counts and layer schedule. Diagnose
first actual sparse extrusion command rather than a source-line-number offset.

Proposed 12 cases:

| Case | Geometry / pattern / count / variation |
| --- | --- |
| A1 | Prism / alignedrectilinear / 1 / angle45 density30 |
| A2 | Same / 2 |
| A3 | Same / 3 |
| A10 | Same / 10 |
| A2A0 | Same / 2 / angle0 |
| A2D20 | Same / 2 / density20 |
| AH1 | Holed prism / alignedrectilinear / 1 |
| AH2 | Same / 2 |
| A2C2 | Prism / alignedrectilinear / 2 / real infill combination2 |
| R2 | Prism / rectilinear / 2 / merged-port control |
| M2 | Prism / monotonic solid role / configured count2, effective count1 control |
| T2 | Prism / triangles / 2 / merged-port control |

For every active aligned count>1 witness, also slice its matching count1 Native
and Ares baseline. Check multiple sparse layers retain the fixed orientation.
Native rejection or rewriting a requested value is reported, not silently
substituted. Require Native acceptance in official and seeded plus a real main
whole-output mismatch with sparse attribution. If all candidates agree, report
NO RED FOUND and stop without production changes. RED is a separate tests/input
commit, reviewed before GREEN. Do not submit generated Native outputs.

## Acceptance

After SPEC approval, directed runs use outer taskset CPU11, jobs1/threads2,
load<60 and fresh artifact/cache directories for every run. Record exact HEAD,
clean status, producer identities, acceptance, first difference and raw hashes.
Keep all comparisons and controls. GREEN must make the two modes fully equal;
count1 and non-RED controls must have byte-identical Ares raw outputs to RED.

Run whole option_coverage in both modes, strict clippy, fmt and wasm32 on final
HEAD, with every changed file <=400 LOC. Canonical sweep without
ARES_PARITY_OPTIONS is visibly SKIPPED and Native-not-executed, per its existing
contract. Default-environment workspace/CI and canonical full follow coordinator
scheduling; full requires explicit FULL-GO. No push without exact-hash approval.
