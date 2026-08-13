# Task 22O.69 architecture decision record

## Status

Accepted and implemented after independent pre-RED approval.

## Upstream boundary

Task 22O.69 ports pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintObject.cpp:3368-3374` — solid recomposition;
- `SurfaceCollection.cpp:45-51` — stable type filtering;
- `Surface.hpp:159-184` — stable Surface/SurfacePtr to ExPolygon copying and
  `Surface.hpp:55-58` — the exact default-metadata const-reference Surface
  construction used by `emplace_back`;
- `ClipperUtils.hpp:364-370,442-455`, `ClipperUtils.cpp:438-572,737-755` —
  exact ExPolygon safety-offset union and default no-safety NonZero `diff_ex`.

## Rust destination seam

Add ordinary module
`project_slice/prepare_infill/bridge_over_infill/internal_solid_recomposition.rs`
with this crate-private interface:

```rust
pub(in crate::project_slice) fn recompose_internal_solids(
    fill_surfaces: &[RegionSurface],
    additional_ensuring: &[ExPolygon],
    cut_from_infill: &[Polygon],
) -> Result<Vec<RegionSurface>, ClipperError>;
```

## Decision

The operation:

1. stably selects every `InternalSolid` ExPolygon in region-surface order;
2. appends `additional_ensuring` in caller order;
3. calls the existing no-safety `difference_ex_polygons` exactly once with that
   complete subject and `cut_from_infill`, even when either is empty;
4. passes the exact ExPolygon difference result, preserving component topology
   and order, directly to the ExPolygon safety-union overload;
5. calls that safety union exactly once, even when the result is empty; and
6. emits fresh default-metadata `InternalSolid` surfaces in engine result order.

A difference error prevents union; a union error prevents output. The first
error returns atomically without mutating borrowed inputs.

## Trusted domain and portability

All three inputs belong to the same object/layer/region bridge transaction.
They are normalized outputs of the preceding region/O65/O66 operations and
contain Clipper-safe coordinates. This operation performs no validation,
normalization, fallback, sorting, deduplication, option lookup, or hardcoded
fixture dispatch.

The seam remains `ares-core`-private and production-unwired. It introduces no
filesystem, terminal, OS, thread, UI, OpenGL, unsafe, or platform-specific
behavior.

## Included and deferred

Included: exact behavior at `PrintObject.cpp:3368-3374`. Debug-only drawing at
`3376-3383`, region removal/append at `3385-3386`, enclosing traversal, second
bridge pass, prepared lifecycle activation, extrusion, motion, G-code, CLI, and
complete KSR golden parity remain deferred.

All Rust source and test files stay below 400 physical lines and use ordinary
modules; `include!` and `include_bytes!` are forbidden for source splitting.

## Evidence

Behavioral RED: `/tmp/task22o69-behavioral-red.log`. Implementation passes
focused 6/6, dependency 794/794, workspace 6,454/6,454 with two skipped,
warning-denying Clippy, rustfmt, wasm32 core/WASM, and x86_64/aarch64 Windows and
macOS checks. All 26 compiling mutations are killed; production restores to
SHA-256 `d170b25cb69d48a4befba3cb766eede5387109308ce0452961b3dc174f4bde3d`,
and the mutation output SHA-256 is
`c32353f173f79f84aaa67ace8bb15243071a2f80d67498b88dbeaf3c6c05d91e`.
Independent six-axis implementation review approved with no blockers or major repairs.
