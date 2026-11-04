# CrossHatch sparse dont_connect admission

No Native run or production edit yet.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Owning upstream boundary

`FillCrossHatch::_fill_surface_single`, `Fill/FillCrossHatch.cpp:178-230`,
generates, multiline-expands and clips its pattern, then calls
`Fill::chain_or_connect_infill` at line 225 without a positive-anchor precondition.
`FillParams::dont_connect`, `Fill/FillBase.hpp:48-52`, selects that mode when
`anchor_length_max < 0.05f`. `Fill/FillBase.cpp:1820-1828` chains multiple
polylines and appends them in that mode; it invokes boundary connection only
otherwise. No equivalent CrossHatch entry assertion exists upstream.

The public option is `PrintConfig.cpp:3165-3189` (`infill_anchor_max`,
coFloatOrPercent), including the explicit zero choice. `Fill.cpp:978-989`
projects sparse anchor lengths from the owning region, converts percentages
when present, and clamps anchor_length to anchor_length_max.
`Fill.cpp:1286-1287` passes both projected values to the emission generator;
`Fill.cpp:1481-1482` also passes them for sparse anchoring.
Solid and bridge roles retain the upstream unlimited values at lines 973-977.

## Current Rust mismatch and destination

`crates/ares-core/src/fill/cross_hatch.rs:34` has
`debug_assert!(params.anchor_length_max >= 0.05)`, before geometry generation.
For an admitted sparse request below that threshold, the debug/test public
slicing API can panic before reaching the upstream dont_connect behavior.
This is a source-level hypothesis; no public Native RED has yet been claimed.

`fill/connect.rs:68-76` already implements the exact threshold and chains
multiple polylines via the existing chain_polylines port.
`cross_hatch.rs:93-104` already forwards both anchors to that connector.
`project_slice/fill_entities/crosshatch.rs:14-31` forwards the current region's
values. The bridge-over-infill CrossHatch arm uses the same primitive.
The existing `transaction/anchor_projection.rs` admits finite small anchor maxima.

Proposed production boundary: only remove the obsolete CrossHatch entry
assertion, with a source reference to FillBase.hpp:52 and FillBase.cpp:1820-1828.
The shared connector, pattern geometry, ordering, multiline offset, clipping,
rotation, flow and emission code remain unchanged. The existing entry check is
an Ares compatibility scaffold, not an upstream restriction.
No existing test assertion is proposed for reversal or removal.

## Included and deferred behavior

Include sparse CrossHatch on one object, one region, one nozzle/filament,
by_layer, with finite nonnegative absolute-mm anchor maxima, multiline 1/2,
and the actual public API plus CLI path. Include the strict 0.05 boundary
and unchanged connected-mode controls.

Defer all other patterns, ironing, negative/nonfinite anchors, percent-length
coverage, heterogeneous regions, new connection algorithms and pattern numeric
changes. In particular, do not edit anchor_projection/anchoring-direction
files, SupportCubic, seam slice_z, support/brim or LockedZag,
skirt/brim/wipe, or processor/browser code. Past CrossHatch
ironing work is merged; this proposal does not change ironing dispatch.

The previously reported LockedZag density finding is not this slice: it
has already been shown the initial density is overwritten by the per-bucket
values (FillRectilinear.cpp:3902/3924); its public 15% probes had no RED.

## Native-first public RED

Run only directed tests; no full/workspace run.
Each mode uses a fresh artifact/cache root and the canonical producer identities.

Use the tracked authored `tests/parity/multiline-union/prism.stl`, SHA256
`1d50f639d940e2a95c324b8f7ef1a33ba9bcf7e83706ba3cf0669f67f6feae7e`,
with the existing runtime fixture adapter and OrcaRunner export/slice cache.
Use the Ender3 classic 0.2mm single-filament setup, CrossHatch sparse pattern,
30% density, 0.45mm sparse width, two walls, three top/bottom layers;
no tower/support/ironing/brim/skirt/arcs. Keep nonempty stock templates.
Only the requested anchor maximum and multiline count vary.

Eight controls: anchor maximum 0, 0.049, 0.05 and 1000mm, each multiline 1/2.
The four below-threshold legs are prospective RED; the four at/above-threshold
legs are prospective whole-byte controls. Native admission and effective option
consumption must first be demonstrated in official and seeded. Zero/0.049
must reach the source-defined threshold, not be rejected or silently rewritten.
Native runtime outputs remain outside the repository.

For every leg assert public API and CLI admission, API=CLI raw bytes, complete
ordered Native comparison with only the existing generated-by normalization,
line count, physical layer schedule and sparse-role activity. Preserve artifacts
including input/effective values and comparison diagnostics. A panic/admission
failure against admitted Native is a valid public RED; do not use a compile
failure or a hand-constructed numeric example instead.

If Native rejects a witness, report it before changing the request. If all
legs already match on main, report NO RED FOUND and stop without production.
Commit/report the public RED separately before any GREEN. Keep all admitted
RED legs and all control assertions. Any residual after the bounded removal
means STOP and a new source-localization report, not a larger silent change.

## Acceptance

At the exact final clean hash: eight legs official+seeded, the entire
option_coverage group in both modes including non-Native tests, strict workspace
clippy, fmt and wasm32 check. At/above-threshold and other existing CrossHatch
controls must retain raw bytes against the same input on main; no comparator
change or coordinate sorting. Canonical absent ARES_PARITY_OPTIONS sweep is
reported explicitly as visible SKIPPED / Native-not-executed.

Any shared primitive change or existing unrelated raw-output change stops the
slice. No new guards, ignores, suppressions, probes,
public APIs or Native artifacts; files stay at most 400 LOC.
