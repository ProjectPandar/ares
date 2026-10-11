# SPEC: FillStars sparse multiline trapezoidal branch

Status: SPEC-REVIEW-REQUEST. Baseline `c2de7a2e4`. This commit is documentation
only: no Native execution, test, production edit or new coverage claim.

## Priority and ownership

Select the `tri-hexagon` / `FillStars` active `fill_multiline > 1` dependency
under the [roadmap's current full-acceptance priority](../roadmap.md#current-acceptance-priorities-full-acceptance-open).
The current Option plan already declares `fill_multiline` min1/max10/interior5
and the `sparse_infill_pattern` enum covered. That does not establish this
coupled consumer: the current Stars adapter explicitly defers the trapezoidal
branch and unconditionally uses the ordinary scanline-family generator.
This is not another inventory-count or domain-status promotion.

No `FillStars`/trapezoidal claim was found in the coordinator inbox. Exclude
brim, scarf/APA, G4/tool ordering, profile inheritance, CI and web. Do not change
Grid or Triangles dispatch, their generators, or their prior parity contracts.
Any conflicting assignment discovered before implementation stops this slice.

## Upstream ownership

Source tree: `OrcaSlicer`, revision `8500fcdcc`. Paths in this table are relative
to `src/libslic3r`; line references were checked against the local source.

| Source boundary | Contract to rewrite / preserve |
| --- | --- |
| `PrintConfig.cpp:238`, `:2996-3003` | `tri-hexagon` maps to `ipStars`; `fill_multiline` is coInt, default1, min1/max10. |
| `Fill/Fill.cpp:925-926`, `:1281-1288` | Only sparse `erInternalInfill` receives the configured bundle count; other roles receive1. Density converts from percent to float fraction; anchors pass to FillParams. |
| `Fill/FillRectilinear.hpp:105-115` | `FillStars::fill_surface`; its `_layer_angle` returns0. Do not add ordinary alternating scanline rotation. |
| `Fill/FillRectilinear.cpp:3487-3505` | Count>1 dispatches to `fill_surface_trapezoidal(..., Pattern_type=2)`; count1 retains `fill_surface_by_multilines`. |
| `Fill/FillRectilinear.cpp:3048-3085` | Shared trapezoidal setup: truncate scaled spacing before integer multiplication for d1, triangular period, float-origin infill direction and double base-angle addition, rotate the domain, extended object bbox, effective infill-layer id. |
| `Fill/FillRectilinear.cpp:3260-3336` | Mode2 star-row coordinates, integer halves/chamfers, aligned symmetric limits, normal/mirrored rows and alternating pair shifts. Apply `(layer_id / thickness_layers) % 3` orientation when thickness_layers>0; otherwise use layer_id. |
| `Fill/FillRectilinear.cpp:3344-3384` | Bundle expansion, half-spacing surface contraction, source fallback to the original domain if contraction is empty, ordered clipping, `< scale_(0.8*spacing)` short-line removal, connect-or-chain branch, rotation back. |
| `Fill/FillBase.cpp:275-329`, `:1569-1578`; `Print.hpp:347`; `Fill/Fill.cpp:1224` | Object-centered reference, source angle arithmetic and transpose-merged/sqrt2-scaled PrintObject bbox. Surface-local bounds must not replace the object-wide generation domain. |
| `Fill/FillBase.cpp:2712-2784`, `Fill/FillBase.hpp:52` | Existing `multiline_fill` bundle expansion; `dont_connect` is anchor_length_max<0.05f. |
| `Fill/FillBase.cpp:133-175` | Existing extrusion collection, flow/width/height and role materialization remain the consumer boundary; no new Ares extrusion pipeline. |

This slice implements mode2 and its necessary setup/finalization, not the
mode0 Grid or mode1 Triangles branches. Preserve the source integer casts,
integer division, float density/angle starting points, double calculations,
rotation center and clipping/connection order. The call to `multiline_fill`
passes spacing as float; preserve that narrowing at this source boundary.
Do not substitute rounded
coordinates, a per-surface grid or a synthesized approximate star pattern.

## Rust destination and existing scaffold

- `ares-core/src/project_slice/fill_entities/stars.rs::append`: conditional
  source dispatch. Preserve the count1 route and collection flow/role behavior.
- `ares-core/src/fill/stars_trapezoidal.rs` with private children only if needed
  to keep each file below400LOC: `FillRectilinear::fill_surface_trapezoidal`
  mode2 rows and finalization. Reuse the existing `fill::multiline_offset`,
  checked rotation, ordered open-path clipping and `fill::connect` primitives;
  do not create another geometry/cache framework or public API.
- Minimal Stars-specific wiring in `project_slice/fill_entities/generate.rs`
  carries layer id, surface thickness_layers and the required object-wide
  extended bbox. Existing `ObjectFillGeometry`/X-bound adapters are temporary
  compatibility shells, not proof that all bbox provenance is ported. Derive
  both dimensions from the source-centered box; do not globally change the
  legacy bbox callers or reinterpret compensated slice bounds as mesh size.
  If correct provenance requires a wider traversal change, report that boundary
  before editing it. The finite inputs below disable XY compensation.
- `ares-cli/tests/orca_parity/option_coverage/tri_hexagon_multiline.rs` and
  dedicated helpers: runtime Native admission, complete-output RED and controls.
  A registration line is the only intended shared test-module edit. No domain
  plan, inventory count, generic activation or comparator change is proposed.

The current `fill::multiline::fill_surface` remains the source rewrite of
`fill_surface_by_multilines` for its existing callers. For Stars count>1 it is
an incorrect provisional dispatch, not a legacy fallback to retain after a
real RED justifies the new branch. Adjacent primitive mismatches discovered
by the RED require a separate bounded report rather than a broad refactor.

## Finite Native admission / RED design

Use one nozzle / one filament, by-layer classic walls, no support/raft/brim,
prime tower, arc fitting, scarf, fuzzy skin, SAFC or adaptive PA. Fixed .2mm
layer height, positive30% sparse density, rectilinear solid/bottom/top fill,
and stock templates/settings. Admit two provenance-recorded slicing inputs:
a centered64x48x4mm prism and the same prism with a centered8x8mm through-hole.
Inputs are authored geometry/project data; store no Native-generated output.
Native export may resolve settings outside git; Ares must consume those exact
exported project bytes. Keep input/archive-member hashes in runtime receipts.

| Case | Geometry | fill_multiline | Density | Infill direction | Connection |
| --- | --- | --- | --- | --- | --- |
| S1 | prism |1 |30% |45deg |stock anchors |
| S2 | prism |2 |30% |45deg |stock anchors |
| S3 | prism |3 |30% |45deg |stock anchors |
| S10 | prism |10 |30% |45deg |stock anchors |
| S2A | prism |2 |15% |0deg |stock anchors |
| S2N | prism |2 |30% |45deg |infill_anchor_max=0 |
| H1 | holed prism |1 |30% |45deg |stock anchors |
| H2 | holed prism |2 |30% |45deg |stock anchors |

After approval, freeze Native/Ares requests and all first outcomes. Require
successful Native slicing and unchanged effective requested values. Prove
actual positive-XYE sparse extrusion, at least three reached source infill-layer
phases and both odd/even bundle witnesses. Hole case must reach clipping on
both exterior and hole boundaries. The anchor-zero request must actually reach
the source no-connect branch. A Native rejection, timeout, rewritten request,
absent consumer, unstable Native output or unrelated first difference is not
this port's RED; report it without changing context to manufacture a PASS.

Use existing OrcaRunner `slice_case`, producer identity/lock/receipt and cache.
Compare the complete ordered bytes and line count with the existing generator-
only normalization. Preserve M73, travel, flow, footer, repeats and ordering.
Retain first differing line/byte, layer, role and complete outputs outside git;
confirm the first body difference is sparse-infill geometry/order, not admission
or metadata. S1/H1 are count1 controls. Do not infer active multiline coverage
from a settings echo, scalar range plan or filtered polygon comparison.

If the finite count>1 cases already match current main completely, report
NO RED FOUND with evidence and do not implement from source suspicion alone.
Otherwise commit the real Native-backed RED before any core GREEN. No tests
or Native runs are authorized by this spec submission itself.

## Included, deferred and acceptance

Included: sparse Stars mode2 row generation and source finalization for the
finite1/2/3/10, angle/density, hole and connect/no-connect witnesses; unchanged
count1 behavior and role-specific count routing. Source connection primitives
remain the single implementation. Core stays byte/geometry-only and portable
to WASM, Windows, macOS and Linux; filesystem/Native work stays in CLI E2E.

Deferred: Grid/Triangles trapezoidal modes, other patterns, complete multiline
pattern exhaustion, infill combination>1, bridge/solid Stars selection,
object/region overrides, modifier/range settings, nonzero XY compensation,
non-axis-aligned model transforms, multinozzle/SEMM, towers and GUI consumers.
Source integer/floating arithmetic is included; no UB or nondeterminism
emulation, fallback, ignore or assertion tolerance is permitted.

Exit after SPEC approval: Native-backed RED -> bounded core GREEN -> fresh
cold/hot directed witnesses in seeded/official and entire option_coverage group
(including non-Native tests), strict clippy/fmt/wasm32 and fresh default workspace.
Any original expectations updated must be itemized; unrelated assertions/plans
remain unchanged. A production GREEN then queues a fresh full on latest main:
shared base keyed by exact main hash, seeded/official candidate legs run in
parallel with distinct fresh roots/caches, each threads2. Require cand.fail to
be a subset of the corresponding shared base.fail (currently both empty).
No inherited print-statistics failure exception. Audit changed raw streams and
retain all failures before REVIEW-REQUEST; no automatic reruns or push.
