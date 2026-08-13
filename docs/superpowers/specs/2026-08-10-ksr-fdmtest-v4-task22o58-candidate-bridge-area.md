# Task 22O.58 — candidate bridge area filtering

## Status

Completed and independently approved.

## Goal and source boundary

Port pinned `PrintObject.cpp:3215-3224`: for one source-ordered candidate and composer-supplied `Coord`, expand by thick bridge spacing, intersect deep area, retain polygons individually by internal-unsupported overlap, and compute limiting union. Loop orchestration/Flow conversion at 3213-3214 and the empty `continue` at 3226-3227 remain composer behavior; O58 returns an empty `area_to_be_bridge` plus the successfully computed limiting union.

`Flow.hpp:69` conversion is composer-deferred. Direct O58 closure is `ClipperUtils.hpp:19,27,375-383,496-498,543-546`, `ClipperUtils.cpp:267-414,671-673,702-703,722-735`, and inherited accepted Task 22F/22G Clipper kernels. Destination: ordinary private `prepare_infill/bridge_over_infill/candidate_bridge_area.rs` and ordinary test children.

## Interface

```rust
pub(in crate::project_slice) struct CandidateBridgeArea {
    pub area_to_be_bridge: Vec<Polygon>,
    pub limiting_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn prepare_candidate_bridge_area(
    candidate_polygons: &[Polygon],
    deep_infill_area: &[Polygon],
    internal_unsupported_area: &[Polygon],
    expansion_area: &[Polygon],
    scaled_spacing: Coord,
) -> Result<CandidateBridgeArea, ClipperError>;
```

The composer owns O55 candidate order, O43 source geometry, O57 context provenance, and candidate-region O48 Flow conversion. O58 accepts exact borrowed values and returns owned output only.

## Behavior

1. Compute `scaled_spacing as f32` directly and call one Miter/3 candidate expansion.
2. Call one flat intersection with expanded candidates as subject and deep area as clip.
3. Visit intersection polygons in order. For every polygon call one flat intersection with that polygon alone as subject and the full internal-unsupported list as clip. Keep nonempty results' original polygon, not the predicate output. Preserve order.
4. Clone survivors, append expansion-area polygons, and call one unconditional flat union over the concatenated vector.
5. Return the original ordered survivors and ordered union result, including an empty survivor list. The deferred composer performs lines 3226-3227's empty gate after observing the successful union.
6. Propagate first engine error in exact call order and preserve every borrowed field/allocation.

Spacing is strictly positive source `coord_t` with a finite positive direct f32 cast. Geometry is valid source flat closed input. No public validation, fallback, saturation, host sorting, or output canonicalization is added.

## Deferrals

Deferred: loop/Flow provenance at 3213-3214; source empty continue at 3226-3227; boundary polylines 3229-3233; O49/O51/O53; lightning clipping; collision rerun; opening/closing and total-fill/top postprocessing; expansion-area mutation; candidate commit; region surface rewrite; successor/lifecycle; extrusion, motion, G-code, CLI parity.

## Acceptance

Begin with compiling RED. Build a removed actual-source driver against the audited fixed-MSVC-order archive and freeze exact ordered flat outputs, empty/union-before-continue behavior, predicate traversal, topology, and spacing cast bits.

Tests must discriminate:

- exact direct `coord_t`-to-f32 spacing bits; a static/source audit bans f64 intermediates because the source requires no f64 intermediate;
- one candidate expansion before deep intersection and exact subject/clip roles;
- per-polygon predicate order, one-polygon subject ownership, full unsupported clip ownership, original survivor retention, and hole/component output;
- survivor order and allocation preservation;
- survivor-first then expansion-area concatenation and one unconditional union;
- unconditional union before the composer-deferred empty gate, including an empty candidate plus invalid expansion area;
- empty candidate/deep/unsupported/expansion combinations;
- first natural range error at expansion, deep intersection, the first predicate, and union; injected private operation seams prove later predicate ordinal propagation and short-circuiting;
- complete input/allocation nonmutation and repeatability.

Use private internal operation-order seams where mathematically commutative kernels would otherwise hide role/concatenation mutations. Statically audit the direct cast; run reversible mutations for expansion omission, deep role/order, predicate omission/reversal/output substitution, unsupported role/list ownership, survivor reorder, union concatenation reversal/repeated union/early return, and error order; restore current source byte-exact.

Private ordinary registration is required; every file contains at most 399 lines. `include!`, `include_bytes!`, and `include_str!` are prohibited for source splitting. Final gates: focused O58, O43-O58/Clipper/Flow dependency, workspace, rustfmt, strict Clippy, wasm32, Windows/macOS, diff/LOC/static/clean-Orca/no-staged checks, and independent six-axis repair/re-review until unconditional approval.

## Completion evidence

The removed source oracle is byte-repeatable and records spacing `0x1p+24`, ordered expanded/split survivors, limiting unions, and empty-survivor union behavior. Its fixed archive/driver/object/binary/output/link-command SHA-256 values are respectively `b643964e681e9435680b78fdd743dcb12c9c07cd16ef164e353d370add8132a1`, `1255ab783e35bc33e06844bf762cc7338303c56e73f53ca80175a993016f60f6`, `209f83c8d2f699827f201f880b99c744f0c5d532f9d9c9eaaa633dbb5b4393a0`, `923a769f1489997df87a9055d854f7e0bac9999e2630df280d81e61ea6594c9f`, `7701da24182d6cb4532bdaaafadf37ef5930c63f7f36a570c56de30f563eeec3`, and `a0564814087e754d1868be0022512540b5b3197001a74bd59fc699794405396b`.

All fifteen reversible mutations were killed, including named repeated-union and two competing-error-order variants; source restored at SHA-256 `7a3637253b9ae84dc50e7cabb35a4c83a555aab2259df9c36349296fcd6387f4`, and the cast audit passed. Final runtime/portability results are focused 10/10, dependency 708/708, workspace 6,368/6,368 with two skipped, strict Clippy, wasm32, four Windows/macOS cross checks, rustfmt, diff/LOC/static, clean Orca, and no staged files. Independent final six-axis re-review approved without blockers.
