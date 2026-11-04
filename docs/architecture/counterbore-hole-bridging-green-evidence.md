# Classic Filled GREEN — bounded public parity evidence

Directed GREEN only; no complete Filled coverage claim.

## Upstream rewrite implemented

Pinned OrcaSlicer source remains `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
This is an extension of the existing upstream-boundary port, not an Ares-owned
bridge feature or G-code patch:

- `PerimeterGenerator::process_no_bridge`, `PerimeterGenerator.cpp:1777–1855`:
  Filled candidate shrink/convexity/bridgeability/support filtering, anchor
  restoration and clamping, unsupported-hole removal and nested-island mutation
  logic. Rust owner: `perimeters/classic/counterbore/filled.rs`, invoked before
  wall generation from existing `classic/counterbore.rs` / prelude.
- Common gates, detector and shared fill/remainder handling remain in the
  existing port (`:1731–1775,1899–1929`). None and classic Bridges retain their
  existing computation. The margin is the named upstream
  `bridged_infill_margin = scale_(BRIDGE_INFILL_MARGIN)` (`:1736`;
  `libslic3r.h:89`), with `min(margin, spacing, external_width)` anchor (`:1774`);
  no config option was invented.
- `Polygon.cpp:107–123,235–255,315–317`: reuse polygon simplification; the
  candidate convexity predicate ports zero-threshold concave-vertex detection.
- `Layer::make_perimeters`, `Layer.cpp:248–279`: use existing group
  redistribution/extra-fill recovery, not another stage or state model.
- `PrintObject::detect_surfaces_type`, `PrintObject.cpp:1579–1583,1698–1704`:
  only Filled expands the top/bottom comparison domain using generated fill.
  Internal surfaces still derive from original slices. No-upper/no-lower
  branches retain upstream original-slice handling. Existing typed fill clipping
  continues to port `LayerRegion::slices_to_fill_surfaces_clipped:63–80`.
- Remove only classic Filled's temporary compatibility rejection. Arachne's
  non-None hole guard remains explicit; no fallback, no silent skip.

## Public GREEN and non-Filled invariance

| Directed gate | Result |
| --- | --- |
| official public counterbore matrix | 11/11 PASS |
| seeded public counterbore matrix | 11/11 PASS |
| Existing non-Filled controls, raw Ares SHA vs RED | 6/6 identical in each mode |
| Arachne capability/precedence guards | 10/10 PASS |
| `cargo fmt --check` | PASS |
| `cargo clippy -p ares-core -p ares-cli --all-targets --all-features -- -D warnings` | PASS |
| `cargo check -p ares-core --target wasm32-unknown-unknown` | PASS |

The five new **classic Filled** cases use public `ares_core::slice_with_profiles`
and complete Native-backed ordered G-code assertions: primary stepped hole,
first-layer hole, straight hole, solid plate and two independent islands.
Primary, first-layer, straight-hole and two-islands changed from exact guard RED
to complete-output PASS. Filled solid remains PASS. This is not a success-shaped
private polygon test or an assertion of mere nonempty output.

Existing classic partial/None controls are unchanged. No M73, movement,
statistics/footer lines were removed. Native/Ares parity uses only established
generator identity normalization.

The lifecycle migration changes only the wall generator to Arachne for
10 existing guard cases. Same literal counterbore error and all existing
0-invocation/disposal/commit assertions remain. These precedence contracts
do not stand in for classic Filled parity.

Final changed Rust modules are below 400 LOC: runner 322, mechanical helpers
85, counterbore 217, Filled 162, surface classification stage 303.

## Coverage limits

**Coverage gaps remain:** no frozen public nested-island-in-hole fixture or
compatible multi-region fixture proves those particular recovery branches.
The implementation ports their source logic, but the two-independent-islands
case does not establish nested-island coverage. Do not claim complete Filled
coverage from this matrix. Follow-up Native-backed reachability work remains
required for those branches.

Arachne counterbore remains DEFERRED as the next separately specified
slice, covering `process_arachne:2123` and both non-None modes. Unsupported
interface-shell/spiral/modifier/raft/multi-material domains are not expanded
here. No self-designed bridge pipeline, changed threshold, new ignore,
assertion relaxation or Native binary commit was introduced.
