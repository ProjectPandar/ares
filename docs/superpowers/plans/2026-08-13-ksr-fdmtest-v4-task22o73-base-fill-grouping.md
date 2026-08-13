# Task 22O.73 implementation plan

## Status

Implementation and verification steps 1-18 are complete. The exact-tree final
evidence below closes the plan; O73 remains lifecycle-inactive.
Plan date: 2026-08-13.

1. Re-audit pinned OrcaSlicer commit
   `8500fcdccaa10b5099ac20d252af3a7c560046f1` at
   `Fill/Fill.cpp:216-346,829-1067`, both callers at `1213-1224,1377-1397`,
   and every direct Flow/surface/config/Clipper dependency listed in the O73
   ADR. Independently approve the single graph-native seam, exact included and
   deferred boundaries, and the absence of a lifecycle successor.

2. Add the first compiling behavioral RED under
   `project_slice/tests/prepare_infill/group_fills/`. Register it through
   `tests/prepare_infill.rs`. It must prepare the real post-O71 graph, borrow
   its `PreparedPostExternalSurfaces`, call
   `group_fills_base(prepared, object_index, layer_index)`, and inspect only
   `BaseGroupedFills`. Do not expose a test-only params constructor, comparator,
   caller-built layer view, private geometry function, or O46 grouping helper.

3. Add the oracle-side encoder through the same result interface. Freeze the
   pre-narrow 460-slot KSR acceptance aggregate: 477 groups, 1,882 fill
   ExPolygons, 174 fill holes, 2,056 fill paths, 107,540 fill points, and 2,547
   no-overlap ExPolygons, with metadata SHA-256
   `a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`,
   and canonical geometry SHA-256
   `062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
   plus layer-table SHA-256
   `ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`.
   Derive these through O38's audited fixed-MSVC direction order. The fill
   totals exclude the no-overlap section, while the canonical geometry digest
   includes both. Retain the full Linux libstdc++ PRE triplet—metadata
   `25a9ddd67028354ff44607a59c04a065ffa74a99b9f1a05bdc7a1adb9c15dce7`,
   canonical geometry
   `136cca449aebb9d155fd51552f51a7bb3b2f5acb42702bd84b2d2920e265d1dc`,
   and layer table
   `f45a91b4f62dabae2f2320f936b8c903ee5d8e7d8db07fb9251418c82e832bf6`—only
   as a nonnormative source variant.
   Preserve empty layer slots 260-459. Record raw order-only variants instead
   of canonicalizing production geometry. Add explicit `assert_ne!` witnesses
   for the distinct fixed-MSVC O74 post-narrow aggregate totals and each of its
   three hashes. Retain the full Linux POST triplet—metadata
   `36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff`,
   canonical geometry
   `13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c`,
   and layer table
   `15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a`—only
   as nonnormative provenance.

4. Add focused interface REDs for absent layers, borrowed-input immutability,
   repeatability, reachable surface kind/role projections, source density
   skips, first-layer bridge classification, one-based filament selection,
   role-specific nozzle lookup, automatic top width, standard and thick bridge
   Flow, role-speed percent resolution, angle/alignment f32 order, sparse
   anchors, multiline, Gyroid, and LockedZag sidecars. Exercise each case by
   preparing source-shaped graph/options; never inject raw `SurfaceFillParams`.

5. Add ordering/coalescing REDs through the entry point. Cover decreasing f32
   bridge angle, explicit pinned pattern and extrusion-role ranks, reachable
   later comparator fields, signed-zero equivalence, sticky lock/skin/symmetric
   state, first-member representative metadata, authoritative group geometry,
   and source-order append. Prove the KSR distinction of 33
   `params.bridge == true` groups versus 22 `flow.bridge == true` groups.
   Assert that Flow spacing, Flow bridge, `mm3_per_mm`, and `idx` do not enter
   group identity.

6. Add priority REDs through the entry point for first singleton, first
   multi-subject safety union, later safety difference, contour-before-holes
   flattening, raw-predecessor rather than clipped-predecessor accumulation,
   an intermediate group clipped empty, and natural coordinate-range failure.
   The error must be atomic and exactly
   `InvalidInput("fill-grouping polygon coordinate is outside the supported Clipper range")`.

7. Add option/capability REDs for empty rotation templates and model alignment,
   then prove nonempty sparse and solid templates fail with their exact existing
   option keys. Do not route through the legacy simple-list parser or add an
   RNG. Retain the public `multi_region_layer_slices` gate at its current owner;
   do not fabricate region-zero behavior or claim multi-region grouping.

8. Introduce `project_slice::group_fills` and the exact result types from the
   O73 specification. Keep one crate-private entry point and a private graph
   resolver. Add only `mod group_fills;` to `project_slice.rs`; do not add a
   public export, prepared successor, sink, callback, port, adapter, or trait.

9. Use and verify the in-process `project_slice/perimeters/flow/fill.rs`
   dependency from pinned `PrintRegion.cpp`, `LayerRegion.cpp`, and `Flow.cpp`:
   `FillFlowContext`, `FillFlowRole::{Infill, Solid, Top}`,
   `resolve_fill_flow`, `resolve_fill_bridge_flow`, and
   `resolve_configured_fill_flow`. Freeze first-layer width, role-width
   fallback, top auto width `1.0 * nozzle`, other fill auto width
   `1.125f * nozzle`, standard ratio bridge, thick round bridge, and LockedZag
   skin/skeleton behavior. Reuse the existing `Flow`; add no second model or
   compatibility constructor.

10. Implement private graph resolution and exact projection in
    `group_fills/params.rs`. Preserve source region/surface order, f32/f64 cast
    points, the layer-wide sticky params record, one-based extruder semantics,
    actual versus nominal Flow roles, independent bridge fields, role speeds,
    source skip points, InternalVoid observation, and LockedZag raw sidecars.
    Return projection/Flow errors before any geometry work.

11. Implement private explicit rank functions and the source strict-weak key.
    Use ordinary `<`/`>` and comparator equivalence, not `total_cmp`, enum
    casts, derived `Ord` on result params, `operator==` semantics, or hashing.
    Intern in `O(S log G)` and retain the first comparator-equivalent params.

12. Implement the source two-phase materialization in
    `group_fills/coalesce.rs`: build groups in key order, then rescan borrowed
    surfaces to append geometry in source order. Store only representative
    metadata, promote projected f32 bridge angle to f64, copy admitted
    single-region no-overlap state, and materialize LockedZag maps by their
    distinct density and `mm3_per_mm` comparators. Leave multi-region joining
    and no-overlap union deferred.

13. Implement `group_fills/priority.rs` with the exact existing checked Clipper
    operations. Preserve group order and empty groups, append original raw
    subjects to the prior accumulator, and leave the first singleton unchanged.
    Never sort or canonicalize the returned geometry.

14. Run the focused O73 Nextest band and its prepare-infill/project-slice
    dependency bands. Repair production only through the one interface until
    every focused and KSR expectation passes. Confirm tests do not rely on
    private helper state and that all input graph snapshots remain unchanged.

15. Prove lifecycle nonactivation structurally: `slice_project_sync` still
    reaches and disposes the O72 incomplete sink; no O73 prepared state exists;
    O46 still calls its temporary private compatibility grouping; the legacy
    `infills` scaffold is not referenced; and no new Cargo feature or public
    symbol exists. Do not replace O46 until O74 completes
    `Fill.cpp:349-827,1069-1186`.

16. Kill and byte-exactly restore compiling mutations of every reachable
    comparator clause, descending bridge order, comparator equivalence, source
    rescan, raw-prior accumulation, singleton bypass, both safety operations,
    bridge/Flow independence, sticky assignments, one-based nozzle indexing,
    top auto width, template gates, and geometry error text. Mutations must use
    the production interface tests rather than private-helper tests.

17. Run the required final gates only after implementation: focused and
    dependency `cargo nextest run`, `cargo nextest run --workspace`, strict
    workspace Clippy and rustfmt, core/browser WASM, x86_64/aarch64 Windows and
    macOS plus Linux checks, diff/LOC/static/no-staged scans, and a clean pinned
    Orca worktree. Every new or changed Rust source file must stay below 400
    LOC; split before crossing the limit.

18. Request independent source/specification and standards reviews. Apply any
    repair in the main implementation thread, rerun every invalidated gate, and
    obtain unconditional rereview. Record only actual output from the exact
    current tree in the final-evidence section.

## Final exact-tree evidence

Steps 14-18 closed on the same exact tree:

- focused `task22o73` Nextest passed 19/19 with 6,451 skipped;
- prepare-infill Nextest passed 277/277 with 26 slow and 6,193 skipped;
- workspace Nextest passed 6,508/6,508 with 27 slow and two configured skips;
- strict workspace all-target/all-feature Clippy with `-D warnings`,
  `cargo fmt --all -- --check`, and `git diff --check` passed;
- core and adapter WASM, core Windows x86_64/aarch64, and core macOS
  x86_64/aarch64 supplied all six passing Tier-1 checks;
- staged paths were zero, neither Cargo manifest nor `Cargo.lock` changed, and
  the forbidden-production and lifecycle/static scans were clean;
- pinned OrcaSlicer was clean at exact
  `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
- all changed/new Rust files remained below 400 LOC: `project_slice.rs` was the
  maximum changed file at 381 LOC and `group_fills/params/projection.rs` was
  the maximum new production shard at 369 LOC; and
- independent source/specification and standards rereviews closed
  unconditionally.

Thirty-one compiling behavioral mutations were killed and byte-exactly
restored. One additional compiling contour/hole insertion-order mutation was
behaviorally equivalent for normalized valid ExPolygons and therefore
survived; it is not counted as a kill. The oracle's explicit O74 aggregate and
three-hash `assert_ne!` witnesses passed.

| production file | restored SHA-256 |
|---|---|
| `project_slice/group_fills.rs` | `1e0c8bb628a7e587fc5a8adbb81313083db49af5c33c9c075e7bef018683f5d3` |
| `project_slice/group_fills/coalesce.rs` | `71b16ca2b2d4024cd597bd8c48964bf55e9fd8b86d49d43c82fd0fa18d1491ae` |
| `project_slice/group_fills/params.rs` | `7a3b73dd1d12a0df6dbaa53f32d04c20ebe2388f4ffa7cff79031c57d9282088` |
| `project_slice/group_fills/params/locked.rs` | `dbba0d22889347f61b11024bdcda9345cbe7340d3054bc89d5b0f287007bf020` |
| `project_slice/group_fills/params/projection.rs` | `9fac547764b34d70434db46a854ef46a2cc796d6d1aa60c967adb1a2fbf00638` |
| `project_slice/group_fills/priority.rs` | `83df27b3d976b4b5701d8a061f16a03447f6d7d3cbff8b19d99dfd82937eb4dd` |
| `project_slice/group_fills/types.rs` | `2916cc6bdd2f02175c14ca4fafc1265866b65f675a4b0bbf47edd81b160e7eb3` |
| `project_slice/perimeters/flow.rs` | `7d5138ef9c369f2872ad184e89ebd21e18eaf2867a730e1ed99bce1fe566ace3` |
| `project_slice/perimeters/flow/fill.rs` | `43837d725862a580d325cb2c53eb9ceb37fe1ca37121dfd67832066b3763ca6c` |

O74 is the next upstream owner. It will consume `BaseGroupedFills` and port
`Fill/Fill.cpp:349-827,1069-1186` for InternalVoid repair and KSR-active narrow
internal-solid splitting. Only O74 may remove the `_base` restriction and make
the complete shared grouping result eligible for O46 replacement and future
`Layer::make_fills` lifecycle activation.
