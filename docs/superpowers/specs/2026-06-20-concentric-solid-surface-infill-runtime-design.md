# Concentric Solid Surface Infill Runtime Design

## Goal

Consume Orca's `ipConcentric` solid/surface infill option as a concrete Ares slicing behavior instead of rejecting it as unimplemented metadata.

This slice implements deterministic rectangle-only concentric solid/surface infill paths for the Ares infill scaffold. It does not claim full Orca polygon offset or Arachne parity.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:88-96` declares `ipConcentric` in `InfillPattern`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1090-1092` stores `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` as `ConfigOptionEnum<InfillPattern>`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:244-253` maps serialized `"concentric"` to `ipConcentric`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1986-2025` registers `"concentric"` for top surface, bottom surface, and internal solid infill pattern options.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:887-899` selects the configured top, bottom, or internal solid pattern for solid surfaces.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:40-44` dispatches `ipConcentric` to `FillConcentric`.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1247-1282` wires `FillConcentric` config and enables Arachne for `ipConcentric`.
- `OrcaSlicer/src/libslic3r/Fill/FillConcentric.cpp:12-72` generates concentric loops by contracting polygons, ordering outside-in, splitting loops, and clipping loop ends.
- `OrcaSlicer/src/libslic3r/Fill/FillConcentric.hpp:29` marks concentric fill as `no_sort()`.

## Ares Destination Boundary

- `crates/ares-core/src/options/infill.rs`
- `crates/ares-core/src/options/infill/patterns.rs`
- `crates/ares-core/src/infills.rs`
- New focused helper module under `crates/ares-core/src/infills/` if needed to keep `infills.rs` below the 400 LOC limit.
- Existing focused tests under `crates/ares-core/src/options/tests/`, `crates/ares-core/src/infills/tests/`, and `crates/ares-core/src/pipeline/tests/`.

## Included Behavior

1. Add an Ares `InfillPattern::Concentric` variant for user-selected Orca `ipConcentric`.
2. Parse `"concentric"` for:
   - `top_surface_pattern`
   - `bottom_surface_pattern`
   - `internal_solid_infill_pattern`
3. Keep `"concentric_internal"` generated-only and rejected for public option parsing.
4. When an active layer role uses `InfillPattern::Concentric`, generate concentric paths instead of scanline candidates.
5. The first runtime implementation is rectangle-only:
   - Accept exactly one axis-aligned rectangular adjusted contour.
   - Generate outside-in rectangular loops using the same two-point `InfillPath` segment contract as the current Ares infill pipeline.
   - Start each loop at `solid_line_width / 2` from the adjusted rectangle bounds.
   - Step inward by the role spacing already computed for bottom, top, or internal solid infill. This preserves existing `top_surface_density`, `bottom_surface_density`, and internal solid spacing behavior.
   - Emit each loop as bottom, right, top, and left edge segments in that order.
6. Preserve role selection:
   - Bottom surface concentric paths become bottom-surface print paths downstream.
   - Top surface concentric paths become top-solid-infill print paths downstream.
   - Internal solid concentric paths become internal-solid print paths downstream.
7. Preserve existing area suppression: `minimum_sparse_infill_area` is checked before concentric generation.
8. Preserve the existing narrow internal solid behavior: generated `ConcentricInternal` override remains separate and continues to apply only to narrow internal solid rectangles when enabled.
9. Preserve WASM and platform neutrality: no filesystem, terminal, native viewer, OpenGL, or non-WASM-safe dependency is introduced.

## Deferred Behavior

- Full Orca `FillConcentric` polygon contraction for arbitrary polygons, holes, islands, and non-axis-aligned contours.
- Arachne variable-width concentric toolpaths.
- Loop end clipping from `loop_clipping` / seam gap.
- Nearest-neighbor loop splitting and full travel optimization.
- Multi-line concentric fill.
- Sparse infill `"concentric"` runtime behavior.
- Hilbert curve, Archimedean chords, and octagram spiral runtime behavior.

If a user-selected `concentric` role reaches a non-rectangle or multi-contour adjusted area in this slice, Ares must return `SliceError::InvalidInput` naming `concentric` and the rectangle-only limitation instead of silently falling back to scanlines.

## Acceptance Criteria

1. Option tests prove `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` parse `"concentric"` to `InfillPattern::Concentric`.
2. Option tests prove `internal_solid_infill_pattern = "concentric_internal"` remains rejected with an error naming the key.
3. Infill tests prove a 4 mm by 4 mm rectangular dense internal solid layer with `solid_line_width = 0.5`, `minimum_sparse_infill_area = 0`, and concentric internal solid pattern emits outside-in solid loop segments instead of scanlines:
   - `(0.25,0.25) -> (3.75,0.25)`
   - `(3.75,0.25) -> (3.75,3.75)`
   - `(3.75,3.75) -> (0.25,3.75)`
   - `(0.25,3.75) -> (0.25,0.25)`
   - `(0.75,0.75) -> (3.25,0.75)`
   - `(3.25,0.75) -> (3.25,3.25)`
   - `(3.25,3.25) -> (0.75,3.25)`
   - `(0.75,3.25) -> (0.75,0.75)`
   - `(1.25,1.25) -> (2.75,1.25)`
   - `(2.75,1.25) -> (2.75,2.75)`
   - `(2.75,2.75) -> (1.25,2.75)`
   - `(1.25,2.75) -> (1.25,1.25)`
   - `(1.75,1.75) -> (2.25,1.75)`
   - `(2.25,1.75) -> (2.25,2.25)`
   - `(2.25,2.25) -> (1.75,2.25)`
   - `(1.75,2.25) -> (1.75,1.75)`
4. Infill tests prove bottom and top surface roles use their configured concentric pattern independently of internal solid pattern selection.
5. Pipeline/G-code tests prove `bottom_surface_pattern = "concentric"` and `top_surface_pattern = "concentric"` change rectangular print paths and emitted `;PRINT_PATH` comments to concentric edge segments.
6. A targeted non-rectangular concentric infill test proves unsupported geometry returns `SliceError::InvalidInput` rather than scanline fallback.
7. Verification uses nextest:
   - Focused `cargo nextest run -p ares-core <filters>`
   - Full `cargo nextest run --workspace`
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `git diff --check`
   - Rust source LOC guard from `AGENTS.md`

## Safety and Rollback

The change is isolated to option parsing, infill generation, and tests. Rollback is deleting `InfillPattern::Concentric`, restoring the public parser rejection for `"concentric"`, removing the concentric generation branch, and removing the focused tests. No public crate boundary, dependency, file I/O, or platform-specific behavior changes are required.

