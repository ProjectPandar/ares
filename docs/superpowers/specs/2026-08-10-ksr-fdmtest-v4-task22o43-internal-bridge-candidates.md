# Task 22O.43 — Gather internal-bridge candidates

## Status

Approved. Every acceptance gate and both final independent reviews pass. The
normalized KSR golden remains RED at the CLI `--options` boundary, so the
broader KSR pipeline is still incomplete.

## Goal and upstream boundary

Port OrcaSlicer 2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/PrintObject.cpp:2467-2591`, the first coherent section of
`PrintObject::bridge_over_infill`. The source caller orders this after
`process_external_surfaces` and `clip_fill_surfaces` at
`PrintObject.cpp:640-675`.

The intervening `clip_fill_surfaces` call is an identity operation in the
pinned program: `PrintObjectSlice.cpp:22` is the only definition or assignment
of `PrintObject::infill_only_where_needed`, and it is `false`; the body returns
before reading any project state at `PrintObject.cpp:3869-3872`. O43 records
that source fact but adds no shallow no-op stage and invents no Option.

The Rust destination is a crate-private
`project_slice::prepare_infill::bridge_over_infill` successor after
`PreparedPostExternalSurfaces`. A deep candidate-gathering module hides the
polygon morphology and filtering behind one in-process interface. Its owned
successor retains candidates by stable object/layer/region/surface indices for
the later bridge-angle and surface-commit slices; it never retains references
into a vector that later stages will mutate.

## Direct dependencies

- `PrintObject.cpp:2471-2489::CandidateSurface` for source identity, layer,
  candidate polygons, region identity, and the initial zero bridge angle;
- `PrintObject.cpp:2491-2591` for lightning-pattern detection, lower-layer
  support construction, policy-dependent morphology, candidate filtering, and
  ordered layer grouping;
- `ClipperUtils.hpp:19,23-34,373-405` and
  `ClipperUtils.cpp:264-293,361-410,592-603,670-719` for ShortestEdgeLength,
  winding-sensitive Miter-3 expansion, shrinking, closing, NonZero
  intersection, and difference without a safety offset;
- `Polygon.hpp:127-135` and `ExPolygon.hpp:300-317` for signed area and flat
  contour-then-hole path conversion semantics;
- `Flow.hpp:69::Flow::scaled_spacing`, `libslic3r.h:52,93-96` for integer
  spacing, `EPSILON`, `SCALED_EPSILON`, and `scale_(12.0)`;
- `PrintConfig.hpp:87-98,231-233,988,1103-1104` and
  `PrintConfig.cpp:227-257,379-384,1990-2016,2969-2977,3017-3074` for the
  `sparse_infill_density`, `sparse_infill_pattern`, and
  `dont_filter_internal_bridges` enums, maps, and defaults.

## Required behavior

For each prepared object, Ares must:

1. report whether any effective region uses Lightning infill, without entering
   the later Lightning generator branch;
2. skip only when the physical lower-layer link is absent; when that link
   targets an aligned empty record, process it as empty lower geometry so the
   `nofilter` branch still retains its empty candidate;
3. use the current record's integer-scaled solid-infill spacing;
4. build the initial unsupported area from every lower fill ExPolygon and the
   lower solid mask from every non-`Internal` surface, plus `Internal` when the
   lower region density is exactly 100%;
5. close unsupported paths by `SCALED_EPSILON`, open the lower solid mask by
   shrinking one spacing then expanding `(1 + multiplier)` spacings, shrink
   unsupported paths by `multiplier` spacings, then subtract the solid mask;
   multiply spacing in `f64` and cast each completed offset to `f32` once;
6. choose multiplier 3 only for `dont_filter_internal_bridges=disabled`, and 1
   for both `limited` and `nofilter`;
7. inspect only current `InternalSolid` surfaces, preserving source order;
8. for `nofilter`, expand the unsupported intersection by four spacings and
   retain a candidate even when that geometry is empty;
9. otherwise classify partial support with the unscaled comparison
   `unsupported_area < source_area - EPSILON`, then retain a nonempty
   unsupported intersection when it is wholly unsupported or has area strictly
   greater than `9 * spacing^2`; expand it by four spacings, merge qualifying
   source leftovers whose signed area is strictly between `spacing^2` and
   `spacing * scale_(12 mm)`, close by `SCALED_EPSILON`, and intersect with the
   source ExPolygon; and
10. group owned candidates by ascending layer key while preserving candidate
    order within each layer, initialize every angle to zero, and propagate the
    first Clipper error without fallback or predecessor mutation.

The current prepared graph has the previously documented single-compatible-
region invariant. O43 preserves the real region index in candidate identity;
it does not add a speculative multi-region view abstraction or clone the O42
graph.

## KSR path and deferred behavior

The committed KSR project uses CrossHatch at 15%,
`dont_filter_internal_bridges=disabled`, Normal coordinate scale, later-layer
solid spacing 377079, automatic internal bridge angle, and no extra bridge
layer. Its golden contains 30 `FEATURE: Internal Bridge` runs across 17 layer
heights, so this transform is behavior-bearing.

Included is candidate discovery and its active O42 successor. Deferred are the
Lightning mutation/generator block, exact CrossHatch anchor generation,
adaptive infill data, candidate-layer clustering, depth classification,
bridge-angle selection, anchored-polygon construction, final surface mutation,
the optional second bridge layer, `combine_infill`, fill/toolpath/motion/G-code
generation, and CLI activation. Orca's TBB scheduling, compile-disabled
`PRINT_OBJECT_TIME_LIMIT_MILLIS` timing instrumentation, separate caller
cancellation checks, logging, and debug SVG instrumentation are deferred host
concerns; O43 preserves their semantic candidate grouping and failure boundary
in sequential platform-neutral core code. Existing Ares infill code remains a
temporary compatibility shell and must not substitute for Orca's exact
CrossHatch anchor generator.

## Acceptance

Focused tests must distinguish all three filter policies, exact 100% lower
density behavior, partial-area strict thresholds and leftover bounds, first
layer/no-source behavior, Lightning reporting, source ordering and stable
identity, holes/path conversion, both coordinate scales, and first/later
Clipper errors. A real-KSR lifecycle test must prove nonempty retained candidate
inventory and provenance from composed 3MF options without reading the golden
G-code. Every Rust file remains below 400 LOC; verification uses Nextest,
rustfmt, warning-denying workspace Clippy, native/WASM checks, static audits,
the normalized golden progress probe, and a fresh six-dimensional independent
review.

## Verification record

The compiling empty candidate stub produced the intended behavioral RED when
the first nonempty source emitted no candidate. Review-driven tests then
reproduced two implementation defects before their fixes: Lightning was false
when configured only on a region without a populated record, and an aligned
empty lower record panicked. The repaired stage scans all retained effective
regions and treats that lower record as empty geometry.

Focused coverage is 35/35. It includes exact policy, density, scale, signed-
area, hole, ordering, lifecycle, object-identity, and error behavior. Reversible
mutations proved the signed leftover and partial-area gates, close-before-
shrink order, f64-before-f32 cast order, unscaled `EPSILON`, per-object option
lookup; production was restored byte-for-byte after each mutation. A separate
exact-inventory test proves that toggling the Lightning report does not alter
candidate discovery. The final `candidates.rs` SHA-256 is
`542a5ba2b515894ad0f21e5f02d55aeda1a49a6f2590d9423a4a45037e6d5c6f`.

The final O24-O26/O40-O43 predecessor band passes 154/154. Workspace Nextest
passes 6,161/6,161 with 27 slow and two skipped in 166.277 seconds. Workspace
all-target/all-feature warning-denying Clippy, rustfmt, the ares-core/ares-wasm
`wasm32-unknown-unknown` check, diff/whitespace/LOC/include/fixture/source-pin
audits all pass. The ignored normalized golden probe remains the expected RED
because the CLI still requires `--options`; it is a progress probe, not an O43
failure or a claim of end-to-end KSR parity.

The final independent standards and specification/upstream reviews both
approve the repaired slice unconditionally with no remaining findings.
