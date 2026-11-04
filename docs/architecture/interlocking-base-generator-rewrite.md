# Clean base InterlockingGenerator rewrite

## Source ownership and destination

Port the base normal-part region-pair behavior from
`libslic3r/Feature/Interlocking/InterlockingGenerator.{hpp,cpp}`:
`generate_interlocking_structure`, `generateInterlockingStructure`,
`getShellVoxels`, `addBoundaryCells`, `computeUnionedVolumeRegions`,
`growBorderAreasPerpendicular`, `handleThinAreas`,
`generateMicrostructure`, `applyMicrostructureToOutlines`.
Port the consumed PRISM kernel and planar walking/spread-dot operations from
`Feature/Interlocking/VoxelUtils.{hpp,cpp}`. Thin widths reuse the existing
Rust flow primitives matching `PrintRegion.cpp::flow` and
`Flow.cpp::new_from_config_width`, at the source height 0.1.

Rust destination is the new private
`project_slice/region_slices/interlocking/{mod,voxel,thin,micro}.rs` boundary,
called after region composition and before top-empty-layer removal.
The existing region/layer carriers remain temporary compatibility adapters;
they do not define a new Ares pipeline. No filesystem, viewer or terminal
behavior enters core.

## Included source behavior

- Native invalid-parameter no-op rules, region pairs `a < b`, and equal
  external-perimeter extruder skip; no valid pairs means no slice mutation.
- Source cell dimensions, float rotation, stored PRISM relative offsets,
  negative grid-coordinate division, planar boundary and skin walks using
  the existing aligned-rectilinear filler, not a new scan converter.
- Per-pair closed union and final empty ghost layer computed before thin
  mutation; air filtering and thin handling only for positive avoidance.
- Source beam-width-based detect/expand, actual nozzle-relative external
  flow width, integer scaled-width division and source morphological order.
- Precomputed local beam orientations, XY swap before voxel translation,
  union and inverse rotation, and Internal surface writeback.
- Geometry/fill failures propagate through Result to SliceError. Never use
  empty geometry as an error result. No guessed 0.42 width, default-on-error,
  no-clipping fallback, new unsupported guard, lint suppression or probe.

This slice does not replay old WIP production commits. Existing rectilinear
integer conversion and nearest /
closest-lines fixes are reused unchanged. No duplicate constructor-truncation
proposal or old WIP architecture records are included.

## Public E2E scope and real RED

The public seams are the byte-oriented core project-plate API and CLI slice.
Native CLI runs twice per frozen input before Ares. Preserve both full ordered
Native/API and Native/CLI comparisons with only the existing generator-line
normalizer, plus API/CLI raw equality. Keep complete input/launch/output/
comparison artifacts; never commit Native output or skip missing Native.

The primary real RED on main is the original default two-extruder beam-1
input. Its beam-0 counterpart and single-volume input are controls.
Same-extruder beam-0/1 cases are PASS/no-op controls on main, **not an alleged
independent no-op RED**. The positive boundary-zero case is a second producer
contract only if its fresh Native admission and RED are established before
implementation. Input hashes and existing assertions remain unchanged.

Only tests made green by this producer slice and original PASS controls enter
this slice. The five still-failing WIP cases (three-region beam-1,
odd kernel, orthogonal wipe, thin-auto gap, thin-percent gap), their inputs,
and broad WIP history are excluded. Their
remaining parity work is deferred, not fixed by normalization or guards.
No claim of full multi-region/odd-kernel/wipe/width-gap parity follows from
this base slice. CUBE/DIAMOND kernels, nonplanar voxel-line consumers and
UI/runtime integration are likewise deferred.

## Sequence / acceptance

1. Commit this source-cited spec.
2. Commit only admitted frozen input E2Es after observing actual main RED.
3. Implement production-only source rewrite in already-separated modules;
   do not import old guessed-width/error-swallowing code or its history.
4. Every changed file stays below 400 lines. A necessary carrier-function
   extraction is pure movement, separate from behavior, with body/byte
   invariance evidence; no unrelated refactor.
5. fmt clean and `cargo clippy --workspace --all-targets -- -D warnings`
   clean. Both complete official and seeded suites must have zero
   failures.
