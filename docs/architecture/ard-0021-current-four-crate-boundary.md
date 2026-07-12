# ARD-0021: Current four-crate boundary for OrcaSlicer rewrite

## Status
Accepted

## Context
Ares is a Rust rewrite of OrcaSlicer's `libslic3r` and rendering-neutral `libvgcode` data. The current workspace already contains four active crates in `Cargo.toml` and `AGENTS.md`:

- `crates/ares-core`
- `crates/ares-cli`
- `crates/ares-vgcode`
- `crates/ares-wasm`

The OrcaSlicer source tree has clear top-level ownership boundaries:

- `OrcaSlicer/src/libslic3r` owns platform-neutral slicer logic: model data, geometry, configuration, print lifecycle, support, extrusion, G-code planning/writing, and SLA/FDM print behavior.
- `OrcaSlicer/src/libvgcode` contains both rendering-neutral G-code viewer data and native/OpenGL viewer implementation.
- `OrcaSlicer/src/slic3r` contains application/UI/config-facing integration and should not be folded into the core slicer crate.

## Decision
Keep the active workspace at four crates:

1. `ares-core` owns platform-neutral Rust rewrites of `libslic3r` concepts and the core byte-oriented slicing API.
2. `ares-vgcode` owns rendering-neutral Rust rewrites of `libvgcode` data concepts such as G-code input data, path vertices, layer/range/color data, and role vocabulary.
3. `ares-cli` owns filesystem and terminal behavior while calling `ares-core`.
4. `ares-wasm` owns browser/WASM bindings around `ares-core` and rendering-neutral data exposure.

Do not create additional workspace crates for geometry, config, support, G-code, profile loading, or UI-facing APIs unless a future source-cited milestone proves the concrete Rust API boundary and shows that keeping the code inside the existing crate is worse.

## Consequences
- Future `libslic3r` milestones default to `ares-core` modules.
- Future rendering-neutral `libvgcode` milestones default to `ares-vgcode` modules.
- Filesystem, terminal, native process, and direct OS integration stay in `ares-cli` or future adapters, not `ares-core`.
- Browser-specific exports stay in `ares-wasm`, not `ares-core`.
- UI/OpenGL viewer runtime from `libvgcode` remains out of scope for `ares-core` and `ares-vgcode`; only rendering-neutral data is portable now.
- Candidate geometry/config split crates remain non-members until a milestone approves them and updates `AGENTS.md` plus `Cargo.toml`.

## Rejected
- Splitting `libslic3r/Geometry` into a standalone crate now | no milestone has proven a stable public Rust boundary or reuse pressure.
- Splitting `PrintConfig`/profiles into a standalone crate now | option metadata and profile composition are still being ported incrementally inside `ares-core`.
- Porting `libvgcode` OpenGL viewer runtime into `ares-vgcode` | the crate must stay rendering-neutral and WASM-safe.
- Adding UI-facing crates based on `OrcaSlicer/src/slic3r/GUI` now | the current goal is low-coupled core APIs first, with UI adapters later.
