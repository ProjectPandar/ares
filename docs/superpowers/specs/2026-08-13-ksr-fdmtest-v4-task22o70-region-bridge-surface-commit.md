# Task 22O.70 — region bridge surface commit

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintObject.cpp:3385-3386`, stable removal in
`SurfaceCollection.cpp:127-138`, and append ownership/order in
`SurfaceCollection.hpp:78` and `Surface.hpp:259-261`.

Destination seam:

```rust
pub(in crate::project_slice) fn commit_region_bridge_surfaces(
    fill_surfaces: Vec<RegionSurface>,
    new_surfaces: &[RegionSurface],
) -> Vec<RegionSurface>;
```

Required behavior: consume the original vector and borrow the named-lvalue
`new_surfaces`; remove every original `InternalSolid` and `Internal` element
stably; preserve every other original surface bit and relative order; copy-append
every rebuilt surface in caller order; return the owned combined vector. The
future composer guarantees O67 results followed by O68 then O69 results. No
geometry, error path, validation, sorting, deduplication, fallback, option lookup,
or fixture branch.

Tests must freeze alternating removable/retained order, every currently
representable pre-second-pass `RegionSurfaceKind`,
metadata/topology preservation, empty/original-only/new-only cases, duplicate
preservation, append order, copy independence/allocation distinction where
observable, repeatability, and no source-splitting macros. Compiling mutations
must kill wrong/one-kind filters, keep/remove inversions, unstable/reversed/
sorted/deduplicated retained or appended sequences, prepend/interleave, and
dropped/duplicated elements; production must restore byte-exactly. A structural
type audit records that borrowed `&[RegionSurface]` plus an owned safe-Rust
return makes moving from or aliasing source elements unavailable without unsafe.
Orca surface variants not yet representable in `RegionSurfaceKind` remain
outside this seam and are deferred with the second-pass vocabulary.

Final gates: focused/dependency/workspace Nextest, strict Clippy/rustfmt, wasm32,
x86_64/aarch64 Windows and macOS, diff/LOC/static/include/pinned-Orca/no-staged,
and independent read-only six-axis review/repair loop.

Deferred: traversal/scheduling, second bridge pass `3393+`, prepared lifecycle,
extrusion, motion, G-code, CLI, and complete golden parity.
