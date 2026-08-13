# Task 22O.55 architecture decision record

## Status

Accepted, implemented, gate-verified, and independently reviewed.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `PrintObject.cpp:3127-3153`, the complete per-layer bridge-candidate presort reached immediately after O54 clustering. Direct closure is `Polygon.cpp:422-448::get_extents(Polygons)`, `MultiPoint.cpp:89-92`, `BoundingBox.hpp:21,27-35,95-108`, `BoundingBox.cpp:94-105`, Eigen 5.0.1 `Core/Dot.h:21-25,64-68`, `Core/Redux.h:99-119,443-450,488-490`, `Core/functors/UnaryFunctors.h:93-100`, and `Core/functors/BinaryFunctors.h:34-45` for squared norm, plus ARD-0024's audited MSVC STL 14.44 `std::sort` control flow (`algorithm` SHA-256 `e4cfb31da8ec07af89834d829ea72b20c7e3202476af3b0641cfe8d6ebb245d7`). The pinned Eigen archive identity from `deps/Eigen/Eigen.cmake:6-8` is SHA-256 `0dbb1f9e3aaad66f352c03227d8c983f6f0b49e0b07e71a7300f4abcc01aee12`.

The Rust destination is private `bridge_over_infill/candidate_ordering.rs`:

```rust
pub(in crate::project_slice) fn order_candidate_surfaces(
    candidates: Vec<CandidateSurface>,
) -> Vec<CandidateSurface>;
```

The module and function are visible only inside `crate::project_slice`; they are not public API. The operation consumes one owned O43 candidate vector and moves every original candidate exactly once into source-defined sorted order while preserving payload allocations. It does not clone polygon payloads, alter candidate fields, mutate O43 inventory in place, or activate a lifecycle successor.

## Required semantics

First build a task-local source-shaped bounding key `{ min, max, defined }`; do not use generic `geometry::BoundingBox::from_polygons`, whose merge semantics intentionally lack the source `defined` state. An empty outer `new_polygons` vector remains zero/undefined. Every present polygon is nonempty. Its box is defined exactly when both coordinate extents are positive; zero-width or zero-height boxes retain min/max but remain undefined, while a diagonal collinear polygon with positive X and Y extents is defined regardless of polygon area. Later defined boxes replace an undefined accumulator, and later undefined boxes are ignored. Then order the complete vector with the source comparator: compare `min.x`; when equal compare `min.y`; when both equal return comparator-equivalent with no source index or other tie breaker. Execute this non-stable sort with Ares's platform-neutral fixed MSVC STL 14.44 helper, not a host sort.

When the vector contains more than two candidates, capture `origin` from the first candidate's bounding-box `max` after the first sort and cast both coordinates to f64. Stable-sort only the tail `[1..]` by strictly increasing `(origin - candidate_bbox.min.cast<double>()).squaredNorm()`: subtract cast f64 X and Y, square each, add X then Y, and compare with `<`. Equivalent distances retain the first-sort order. The first candidate never participates in the stable sort.

The trusted domain is source-valid candidate geometry with no empty inner polygon, point counts fitting Rust collections, Clipper-bounded i64 coordinates, and finite f64 squared distances. Bounding-box subtraction occurs after coordinate-to-f64 casts, so no signed integer subtraction is reached. X-square then Y-square then addition remains a source-review/static invariant; swapping the two nonnegative operands is not falsely claimed behaviorally distinguishable. No defensive validation, fallback, saturation, host-dependent ordering, or platform branch is added.

## Consequences

O55 supplies only the deterministic candidate schedule dependency. Deep-area gathering, lower-layer subtraction, O46/O47/O49/O51/O53 composition, collision rerun, postprocessing, candidate commit, surface rewrite, TBB/time-limit/debug adapters, prepared successor, public lifecycle, extrusion, motion, G-code, and CLI parity remain deferred.

Production/tests use ordinary modules, every source stays below 400 LOC, and source-splitting `include!`, `include_bytes!`, and `include_str!` macros are prohibited.

## Completion evidence

A removed temporary driver used pinned candidate/Polygon/BoundingBox/Eigen dependencies plus exact fixed-MSVC sort replay (`b0c5afcc36e5db5a51112dd2054ce757cc9ced6c76b0e4654d300666136b5777`) and the pinned Debug archive `ec47c8b945656e0d52f7223234b80ec66068b4a9c671e71daa5030e049e2a41b`. It froze all-equal and mixed 42-item order, post-sort origin, squared-distance alternatives, and extent-defined bits. Driver/object/binary/output SHA-256 values were `3aa80f9d1ec85e3a79dfb741ac888d62f5b3fd6229baf32d18a6c8715f9db1e4`, `664ea07be4ac6c83f8b74929c65a211f943f4cda26013c21f150fe5232581112`, `30e086930f7fe87d76b120041bf577f9d0926f6d82f6f188ec0c4f4bd0c0d70f`, and `1b4339927e830e9440cf5594e09ec8c1f165d8ae4e41bdb1fbecedcc157342f9`.

Final gates pass focused 12/12, O43-O55/Clipper/Flow dependency 673/673, workspace 6,333/6,333, warning-denying workspace Clippy, core/browser wasm32, both Windows and both macOS targets, rustfmt, diff, LOC, static, clean-Orca, and no-staged checks. Eleven behavioral mutations and two structural mutations are killed; production restores byte-exact to SHA-256 `144b254aa21982bc6e04b173127615f686fc9f8f0afd1fb54e16ea7dc0ff3bcf`.
