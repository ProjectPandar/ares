# Task 22K: Post-Region Top Empty Layer Removal

## Status and objective

This specification is a draft. Production or tracked test implementation may
begin only after the exact specification and implementation-plan bytes receive
independent fixed-source/specification, current-Ares/plan, and default-model
approval.

Task 22K is the next bounded source rewrite in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`fc248673cbfda7552b3fe7cba9eeff0c36345b17` produces the complete Task 22J
post-region stream. Task 22K ports only OrcaSlicer's immediately adjacent loop
that removes the maximal suffix of empty object layers after volume geometry
has been assigned to region surface collections.

The stage consumes only Task 22J post-region state. It introduces no Option,
default, parser, filename, fixture identity, digest, reference G-code input, or
environment dependency. It remains platform-neutral and deterministic on
WASM, Windows, macOS, and Linux.

Task 22K stops before cancellation, `apply_conical_overhang`, painted
segmentation, compensation, surface classification, perimeters, fill,
supports, extrusion paths, G-code assembly, metadata, or post-processing. The
public project API executes Task 22K and continues to return
`SliceError::ProjectSlicingIncomplete`.

## Fixed identities and source blobs

The fixed Ares baseline is commit
`fc248673cbfda7552b3fe7cba9eeff0c36345b17`, tree
`6305eed1ff3a753d4ec91c1ba89f558d0514d709`. Exact-SHA Tier-1 run
`29699174614` passed format, Ubuntu/Linux, Windows, macOS, and WASM/browser.

All upstream citations refer only to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored upstream checkout is
read-only evidence; tracked tests never inspect it.

Fixed source blobs are:

- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/Layer.cpp`,
  `5bdc156d0172ec19894b630cc70d73b5aef8f82d`;
- `src/libslic3r/Layer.hpp`,
  `cb2e6c7c1a166a028ac8fceffaf9f42f3c2426b0`;
- `src/libslic3r/SurfaceCollection.hpp`,
  `1895516aa2eb1fa30be3cf63bb211f7db420f3af`;
- `src/libslic3r/Print.hpp`,
  `c69c5b6570a79cb750c08805e4907eeec5c834f5`.

## Exact upstream rewrite boundary

The owning boundary is:

- `PrintObjectSlice.cpp:1149-1193` for the Task 22J caller context, including
  dense layer-region allocation, the separately retained
  `firstLayerObjSliceByVolume` copy, surface append, and the point immediately
  before trimming;
- `PrintObjectSlice.cpp:1194-1201` for repeatedly deleting only the last layer
  while that layer is empty;
- `PrintObjectSlice.cpp:1202-1203` for clearing the surviving final layer's
  upper pointer;
- `Layer.cpp:13-18` for destruction of a removed layer's owned region objects;
- `Layer.cpp:21-29` and `Layer.hpp:163-170` for layer emptiness as the absence
  of slices in every present region;
- `SurfaceCollection.hpp:49-51` for collection emptiness as
  `surfaces.empty()`, independent of geometric area;
- `Print.hpp:570` for the retained volume-slice sidecar that the trimming loop
  does not modify.

`PrintObjectSlice.cpp:1204` cancellation and
`PrintObjectSlice.cpp:1206,1394+` `apply_conical_overhang` are the next caller
context, not part of Task 22K.

## Required semantics

For each post-region print object, Task 22K finds the longest prefix ending at
the last layer for which at least one registered region owns at least one
surface. It then removes the remaining suffix from both the planned-layer
vector and every region's dense layer vector.

The following rules are non-negotiable:

1. Only a consecutive suffix is removed. Leading and interior empty layers
   remain whenever a later layer is nonempty.
2. A layer is nonempty when any region's `surfaces` vector has at least one
   element.
3. Surface geometry is not inspected. One surface holding an empty
   `ExPolygon` keeps its layer.
4. A layer with zero registered regions is empty.
5. A layer whose every registered region has zero surfaces is empty.
6. An all-empty object becomes a zero-layer object at this boundary; Task 22K
   adds no error or fallback.
7. Surviving `PlannedLayer` values and IDs remain byte-for-byte unchanged and
   are not renumbered.
8. Region identities, Options, ordering, and surviving surfaces remain
   unchanged.
9. Every region layer vector is truncated to exactly the same retained length
   as the planned-layer vector.
10. The occurrence-keyed `VolumeSlices` sidecar remains complete and is never
    inspected or truncated.
11. Objects are trimmed independently.
12. Applying the stage twice is identical to applying it once.

Ares has no layer adjacency pointers. Keeping only the dense surviving prefix
is the Rust equivalent of upstream deleting the suffix and assigning the new
last layer's `upper_layer` to null.

## Rust destination boundary

The production implementation belongs in the real module
`crates/ares-core/src/project_slice/top_empty_layers.rs` with one internal API:

```rust
pub(super) fn remove_project_top_empty_layers(
    objects: &mut [PostRegionPrintObject],
)
```

For each object, the implementation reverse-scans planned-layer indices using
only region surface-vector cardinality, truncates `plan.layers`, and truncates
every `PostRegion.layers` to the same length. It returns `()` and cannot fail.

`prepare_post_top_empty_layers` calls the released `prepare_post_regions`,
applies this function once, and owns the public incomplete stop. Task 22K does
not duplicate Task 22J composition, add a second state model, or introduce an
Option/config dependency.

## Option and input ownership

Task 22K introduces and reads no Option. All geometry and all effective Options
have already been derived from the supplied 3MF by released Task 22J. The trim
decision is purely structural and depends only on the post-region surface
containers created from that input.

Production code may not read:

- project filenames or paths;
- `ksr_fdmtest_v4` names or fixture bytes;
- committed or generated digests;
- reference G-code;
- process-global state, clocks, environment variables, or filesystem data;
- raw 3MF metadata or a parallel Option map.

No executable source-pinning test may inspect Orca commits, trees, blobs,
paths, or line numbers. Source identity belongs only in documentation and
review evidence.

## Native structural acceptance vectors

The pure-stage tests must include:

- two regions and layer occupancy `[nonempty, empty, nonempty, empty, empty]`,
  producing the first three layers only;
- an arbitrary non-dense surviving ID prefix, proving no renumbering;
- a final layer kept by a nonempty surface vector in only one region;
- a surface containing an empty `ExPolygon`, proving container-cardinality
  semantics;
- zero regions, producing zero planned layers;
- multiple all-empty regions, producing zero planned and region layers;
- at least two objects with different retained lengths;
- repeated application producing the same state;
- full sidecar preservation across trimming.

The released ten-object Task 22J synthetic vector is also a required complete
checkpoint. Objects 0 through 8 remain structurally identical. Object 9 has a
nonempty first retained layer and an empty final retained layer; Task 22K keeps
only its first planned/region layer while retaining both sidecar layers.

The independently derived pre-implementation K checkpoint candidate is 5,848
bytes with SHA-256
`037b5e1b5aa9eb2f5c9c38f00a8d7a23768217fd7cc7ec13bb71f21d9edb3b07`.
This identity must be registered before production implementation and must not
be changed to accommodate Ares output.

## Real 3MF anti-hardcoding vectors

Tests must construct two complete in-memory 3MF archives using the existing
deterministic archive helper and profile/config entries. Both archives contain
one `ModelPart` box over Z `0..0.4` and one full-XY `NegativeVolume`; the only
semantic difference is the negative volume's Z interval:

- top slab `0.2..0.4`: Task 22J retains `[nonempty, empty]`; Task 22K retains
  one layer;
- bottom slab `0..0.2`: Task 22J retains `[empty, nonempty]`; Task 22K retains
  both layers.

Both projects must pass through the real loader, effective config, planning,
intersection, Task 22J composition, and Task 22K trim. Tests must prove the
loaded volume kinds are exactly `ModelPart` then `NegativeVolume`, the J input
differs only as implied by the 3MF geometry, and both occurrence sidecars keep
their complete two-layer vectors. No test-only production toggle or injected
Option is allowed.

The top/bottom pair is mandatory because the committed KSR archive is a no-op
at this boundary and cannot by itself prove the stage executed.

## KSR checkpoint contract

Released Task 22J produces a 2,008,706-byte KSR checkpoint with one object,
460 planned/retained layers, one 460-layer sidecar, and a nonempty final layer
at index 459. Task 22K therefore removes zero KSR layers.

The Task 22K checkpoint uses magic `ARES22K\0` and otherwise keeps the exact
Task 22J wire layout. For KSR, every byte after the eight-byte magic must equal
the released J checkpoint. The independently derived expected K identity is:

- length: `2,008,706` bytes;
- SHA-256: `c101e0f9ff863c7abe72cd1cb792fcd8e0074d8d6d2e77d3bb56c32eedba13be`.

The committed project and reference G-code files remain unchanged. Public
`slice_project` must execute Task 22K and still return exactly
`ProjectSlicingIncomplete`; Task 22K does not claim G-code parity.

## WASM and browser boundary

The previous non-default `task22j-browser-oracle` feature is replaced, not
aliased, by `task22k-browser-oracle`. Default core and adapter builds expose no
Task 22 hook. The feature build exposes exactly:

- `task22kBrowserInputOracle`, returning the complete `ARES22J` input;
- `task22kBrowserOracle`, returning the complete `ARES22K` output.

Native Task 22J regressions remain available under `cfg(test)`; no obsolete J
browser export remains.

The browser parser must accept both J and K magic while preserving exact EOF,
safe-integer, dense retained-layer, dense region-ID, Internal-surface, and
record-identity checks. It must allow a complete sidecar to contain more layers
than the post-K planned/retained prefix.

Before any fixture fetch, independent J/K known-answer vectors must prove that
an empty final retained layer is present in J and absent in K while the
sidecar remains complete. Real Chromium must then prove:

- the public KSR boundary remains incomplete;
- feature exports are exactly the two K functions;
- exact KSR J input and K output identities;
- exact synthetic suffix trimming;
- real top-slab trimming and bottom-slab preservation;
- repeatability and complete AST/record parsing.

## Structural constraints and deferrals

Every changed Rust production and test file must remain below 400 physical
LOC. Tests live in real modules. Rust source splitting may not use `include!`,
`include_bytes!`, or related embedding macros. The pre-existing fixture
`include_bytes!` remains fixture embedding, not source splitting, and Task 22K
does not add another occurrence.

No new dependency, crate, unsafe code, native-only API, file I/O, process API,
thread API, or fallback is introduced. The deterministic test ZIP creator
metadata released by Task 22J remains unchanged.

Explicitly deferred behavior includes:

- cancellation at `PrintObjectSlice.cpp:1204`;
- `apply_conical_overhang` at `PrintObjectSlice.cpp:1206,1394+` and its
  `make_overhang_printable*` Options;
- material and painted segmentation;
- XY, hole, and elephant-foot compensation;
- `make_slices`, surface typing, perimeters, fill, supports, extrusion paths,
  G-code assembly, metadata, post-processing, and normalized reference-G-code
  comparison.

## Acceptance and review gate

Task 22K is complete only when:

1. fixed-source, current-Ares, and default-model reviewers approve the exact
   spec/plan bytes before implementation;
2. tests record focused RED evidence before production code exists;
3. all pure, synthetic, real-3MF, KSR, public-boundary, WASM, and Chromium
   contracts pass;
4. Task 22A-J predecessor checkpoints remain exact;
5. rustfmt, strict workspace Clippy, workspace checks, full nextest, wasm32,
   export, LOC, macro, hardcoding, and diff gates pass;
6. an independent read-only reviewer assesses requirement completeness,
   logical correctness, edge cases, code quality, test coverage, and actual
   execution, returning a repair checklist;
7. the main thread repairs every finding and the same review loop repeats until
   all P0-P3 lists are empty;
8. exact reviewed bytes are committed, pushed normally, and pass exact-SHA
   Tier-1 before the next source slice starts.

Expected constants do not change to accept implementation output. Any mismatch
is an implementation defect until fixed-source evidence and independent review
prove otherwise.
