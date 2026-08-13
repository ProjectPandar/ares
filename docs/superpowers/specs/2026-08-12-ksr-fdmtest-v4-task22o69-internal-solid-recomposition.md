# Task 22O.69 — internal solid recomposition

## Source and destination boundary

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `PrintObject.cpp:3368-3374`;
- stable filter/copy behavior in `SurfaceCollection.cpp:45-51` and
  `Surface.hpp:55-58,159-184`;
- default no-safety NonZero difference and the exact ExPolygon safety union in
  `ClipperUtils.hpp:364-370,442-455` and
  `ClipperUtils.cpp:438-572,737-755`.

Destination:
`project_slice/prepare_infill/bridge_over_infill/internal_solid_recomposition.rs`:

```rust
pub(in crate::project_slice) fn recompose_internal_solids(
    fill_surfaces: &[RegionSurface],
    additional_ensuring: &[ExPolygon],
    cut_from_infill: &[Polygon],
) -> Result<Vec<RegionSurface>, ClipperError>;
```

## Required behavior

- stably copy only `InternalSolid` ExPolygons;
- append ensuring ExPolygons in caller order;
- call `difference_ex_polygons` once with exactly that combined subject and the
  exact cut operand, including empty cases;
- pass exactly its returned ExPolygons, with component/hole topology and order
  intact, to the ExPolygon safety-union overload;
- call that union once with exactly the difference result, including empty
  results;
- convert each union result to a fresh default-metadata `InternalSolid` surface
  in engine order;
- skip union after difference error, skip output after union error, propagate the
  first error, and preserve every borrowed input bit.

Inputs are normalized, Clipper-safe values from the same object/layer/region
transaction and preceding O65/O66 operations. No validation, normalization,
fallback, sorting, deduplication, option lookup, or fixture dispatch is added.
The unwired `ares-core`-private seam introduces no filesystem, terminal, OS,
thread, UI, OpenGL, unsafe, or platform-specific behavior.

## Acceptance

Focused tests capture exact operation operands and call cardinality. They must
prove selected-solid then ensuring subject order, exact cut forwarding,
intact ExPolygon topology forwarding, difference-before-union precedence, union engine
order, fresh default metadata, empty-input calls, first-error atomicity,
repeatability, and complete input nonmutation.

Compiling mutation tests must kill early returns; selection bypass/wrong kind;
subject/ensuring reversal, sorting, or deduplication; reversed difference
operands; per-item/batched/repeated difference or union calls; difference bypass;
union on pre-difference input; component/hole omission or reordering; swallowed
errors; wrong output kind/metadata; and output sorting. Production must restore
byte-exactly.

Final gates: focused/dependency/workspace Nextest, warning-denying Clippy,
rustfmt, wasm32 core/WASM, x86_64/aarch64 Windows and macOS, diff/LOC/static/
include/pinned-Orca/no-staged audits, then independent read-only six-axis review
and main-thread repair/re-review until approval.

## Deferred

Debug-only `PrintObject.cpp:3376-3383`, region removal/append at `3385-3386`,
composer/traversal, second bridge pass, lifecycle, extrusion, motion, G-code,
CLI, and complete golden parity.
