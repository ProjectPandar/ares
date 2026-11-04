# Counterbore hole bridging alignment SPEC — classic sacrificial layer

Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
All upstream paths below are relative to `OrcaSlicer/src/libslic3r/` at that pin.
This is a source-cited
libslic3r rewrite slice, not a new Ares bridge pipeline.

## Gap and proposed scope

`counterbore_hole_bridging` is already parsed; the public failure is not an
unknown option. `crates/ares-core/src/project_slice/perimeters/classic/preflight.rs`
`validate_record`, lines 105–113, returns
`SliceError::UnsupportedProjectFeature("counterbore_hole_bridging")` when any
perimeter surface has holes and either:

- mode is `ProcessCounterboreHoleBridging::SacrificialLayer`; or
- dispatch is Arachne and mode is not None.

Classic `PartiallyBridged` already has a real port:
`classic/counterbore.rs::process`, lines 22–66, dispatched from
`classic/prelude.rs:73`; its current mode gate is at lines 33–35.
[Previous partial-bridge spec](classic-counterbore-partiallybridge.md) explicitly
left Filled and Arachne deferred.

**Proposed bounded slice:** implement classic `sacrificiallayer/chbFilled`,
including its perimeter → region fill → surface classification responsibility
chain. Preserve None and classic partiallybridge. Arachne remains explicitly
unsupported on the existing guarded domain. Do not describe this slice as complete support for all enum modes
and wall generators.

## Exact upstream ownership

Source links use the fixed upstream pin, not mutable main:
`https://github.com/SoftFever/OrcaSlicer/blob/8500fcdccaa10b5099ac20d252af3a7c560046f1/src/libslic3r/`.

| File/function and lines | Responsibility in this slice |
| --- | --- |
| `PrintConfig.hpp:401–403`, `CounterboreHoleBridgingOption`; `PrintConfig.cpp:553–557`, enum mapping; `:1547–1563`, option definition | None / Bridges / Filled correspond to `none` / `partiallybridge` / `sacrificiallayer`; do not alias modes. |
| `LayerRegion.cpp:82–142`, `LayerRegion::make_perimeters` | Builds generator with region/object/print options and flows, passes cumulative lower/upper slices, selects Arachne or classic at `:138–141`. This is a dispatch/input owner, **not** the hole-removal algorithm. |
| `PerimeterGenerator.cpp:1144`, `process_classic`; call `:1214` | Runs counterbore processing before classic wall generation. |
| `PerimeterGenerator.cpp:1728–1930`, `process_no_bridge` | Actual geometric owner. Common gates and detector `:1731–1775`; Filled branch `:1777–1855`; shared fill append/remainder/surface mutation `:1899–1930`. |
| `Layer.cpp:185–281`, `Layer::make_perimeters`; split/recovery `:248–279` | Compatible-region fill intersection and recovery of extra fill into first merged region. Gate `:266` is non-None, not Filled-only. |
| `PrintObject.cpp:1520`, `PrintObject::detect_surfaces_type`; `:1579–1583` | For Filled, union slices with generated fill surfaces **for the classification comparison domain**; otherwise extra sacrificial area can disappear. |
| `PrintObject.cpp:1690–1707` | Keeps a separate backup/original-slice domain for internal surfaces when interface shells are off; do not blindly replace every slice domain with the union. |
| `PrintObject.cpp:1912`, inside `detect_surfaces_type`; `LayerRegion.cpp:63–80`, `LayerRegion::slices_to_fill_surfaces_clipped` | Materializes typed fill by intersecting each surface-type group with `fill_expolygons`, retaining the generated boundary. |

Adjacent caller: `PerimeterGenerator.cpp:2093` `process_arachne` also calls
`process_no_bridge` at `:2123`. That source behaviour exists; deferral is an Ares
implementation limit, not an upstream no-op.

## Rust destination boundary

All destinations are within `crates/ares-core/src/project_slice/`:

1. `perimeters/classic/counterbore.rs` and a small Filled-specific sibling if
   needed: port Filled processing at the existing `prelude.rs` invocation,
   before simplification/wall offsets. Reuse existing BridgeDetector port and
   integer geometry. Keep surface metadata and upstream mutation order.
2. `perimeters/classic/preflight.rs::validate_record`: replace only the Filled
   compatibility rejection after real behaviour exists. Preserve the explicit
   deferred Arachne boundary. Do not merely remove the guard.
3. `perimeters/groups/redistribute.rs::apply`: already ports extra-fill recovery
   (`:46–75`); verify Filled uses it, rather than adding another redistribution
   pass or duplicating region state.
4. `prepare_infill/surface_type_detection/stage.rs::stage_record` /
   `classify_slices` / `clipped_fill`: currently classify
   `object.current_surfaces(input)` without Filled's generated-fill union.
   Add the upstream comparison-domain distinction using the existing
   `PreparedLayerRegionPerimeterRecord.fill_surfaces` and `fill_expolygons`.
   materialize the comparison-domain change only for Filled; existing non-Filled
   controls must remain byte-identical. Preserve original-slice internal
   classification semantics and surface types.
   Existing stage structs remain lifecycle carriers, not a parallel pipeline.

No filesystem, terminal, Native invocation, OpenGL, or UI in core. CLI test
adapter alone owns Native execution and evidence. Browser WASM, Windows, macOS,
and Linux continue to use the same platform-neutral implementation. Any new
Rust module stays below 400 lines; do not refactor unrelated stages.

## Included behaviour

- Exact None/lower-slices/unsupported/filtered/hole/bridgeable gates. No layer-0
  hardcoding in place of upstream lower-slice availability.
- Preserve common detector spacing/4, bridge angle, relative angle and model
  alignment semantics, with actual region flow-derived spacing/width.
- Filled candidate shrink; simplified single convex contour and single expanded
  contour checks; bridgeable intersection and expanded-contour support test.
  Restore candidate with `spacing + bridge_anchor_offset`, where
  `bridge_anchor_offset = min(bridged_infill_margin, perimeter_spacing,
  ext_perimeter_width)` (`:1774`), then clamp to original surface
  (`:1778–1801`). At this pinned source `:1736` initializes the margin from
  `scale_(BRIDGE_INFILL_MARGIN)` (`libslic3r.h:89`, constant 1). Retain
  the named upstream margin and do not invent a config option.
- Remove only holes with a nonempty wall band that has no lower support;
  supported holes stay holes. Preserve no-increment-after-erase behaviour
  (`:1803–1825`).
- Recover nested material islands as clockwise holes, matching offset order,
  split/append/delete handling and metadata (`:1828–1854`).
- Append generated Internal fill; subtract it from perimeter remainder and move
  floating remnants into fill. Retain shared ordering (`:1899–1930`).
- Compatible-region extra-fill preservation and Filled comparison-domain union
  before classification, followed by typed fill clipping. Public output must
  exercise the complete chain; a standalone polygon witness is insufficient.

## Deferred behaviour

- Arachne counterbore handling (`process_arachne:2123`) for both non-None modes.
  Keep its current rejection explicit; no classic fallback or silent ignore.
  Arachne is the next counterbore slice: separately specify its
  dispatch/pre-wall mutation and both non-None modes, then public Native RED.
- Expanding presently unsupported interface-shell, spiral, modifier, raft or
  multi-material domains merely to satisfy this slice. Preserve supported
  existing behaviour; if a proposed fixture needs one of these, stop and request
  a scope change rather than weakening guards.
- New bridge detection, mesh repair, slicing/config models, seam/travel/cooling,
  bridge-over-infill or extra-bridge-layer changes. Counterbore addition is not
  permission to implement adjacent pipeline features.

## Public Native-backed RED plan

Before production edits, enumerate failures:
wrong enum/dispatch; guard-only acceptance; contour convexity/filter mismatch;
unsupported hole retained or supported hole removed; lost nested island;
extra fill dropped in group split or classification; changed surface ordering;
None/partial/no-op regressions; and a fixture never reaching the Filled branch.

Use the existing public `ares_core::slice_with_profiles` and the Native adapter
in `crates/ares-cli/tests/orca_parity/runner/counterbore_partiallybridge.rs` as
precedent, not a test-only core seam. Add a separate Filled test module.
Use the original WEMAKE3D PhoenixProV1 0.4mm machine/process and Generic
PLA catalog/selection/parent-chain provenance already frozen in
`tests/parity/counterbore-partiallybridge/`. Flatten the same profiles only for
Native CLI settings; use `Placement::None` and deterministic metadata/context.
Override only `counterbore_hole_bridging=sacrificiallayer` for primary; retain
and record effective `wall_generator=classic`. This is an explicit process
setting, **not** a claim that the stock profile enables Filled.

Freeze input, catalog, settings, override hashes and expected geometry before
its first Native run. Proposed cases:

| Case | Purpose / expected base result to verify |
| --- | --- |
| Existing stepped-hole STL + Filled | Primary public RED: Native succeeds, base returns exact UnsupportedProjectFeature. Prove Filled changes output versus None and generated sacrificial area survives to G-code. |
| Same input + None, and existing classic partiallybridge suite | Already-supported complete-output regression controls; no mode aliasing. |
| First-layer-hole + Filled | No lower slices; Native no-op geometry, current guard is expected too broad. |
| Straight-hole + Filled | Lower supports hole wall; prove supported hole remains open. |
| Solid-plate + Filled | No holes; existing public success/no-change control to verify. |
| Two-islands and a nested-island counterbore fixture + Filled | Distinguish independent-island ordering from true island-in-hole reconstruction. Existing two-islands alone does not prove nested branch reachability. |
| Compatible multi-region counterbore fixture + Filled | Exercise extra fill outside region slices and first-region recovery through a supported public project input; prove nonempty extra fill, not just dispatch. |

For new geometries/project inputs, freeze a repeatable generator and manifold /
volume audit plus profile provenance before Native. If the existing stepped
fixture is filtered out, preserve that failed reachability attempt and submit a
source-derived fixture amendment; do not tune geometry toward candidate GREEN.
Multi-region uses the existing public load-settings/project API if needed; do
not invent an API to expose private stage output. Confirm reachability with a
read-only Native trace/geometry witness if complete-output comparison cannot
identify it. Diagnostics supplement, never replace, public E2E assertions.

Each case saves input/config/profile hashes, raw Native G-code/stdout/stderr,
Ares G-code or exact error JSON and comparison first byte/line in a fresh
artifact root. Compare **complete ordered G-code**, allowing only the established generated-by identity
normalization; retain movement, M73, statistics and footer. No goldens generated
from Ares. No movement-only checks, skip, new ignore, thresholds or assertions
weakened. Assert PASS eventually; base guard errors are RED, not passing tests.

Directed Native/Ares execution uses isolated target/cache/artifact roots and
process ownership. Use `cargo nextest run` with only this directed slice;
preserve existing case deadline and descendant cleanup.

## Exit sequence

1. SPEC + [ARD](ard-counterbore-hole-bridging.md) fix scope and RED matrix.
   Public RED precedes any production change.
2. Port exact Filled responsibility chain, then original RED and controls fresh
   in official and seeded modes. Preserve all failures.
3. If Native Variant A occurs: identical Ares SHA and Native fresh ×3 PASS
   evidence before adjudication; never silently accept or retry loop.
4. After implementation, strict clippy, rustfmt and WASM checks plus directed
   evidence, then the full suite.
