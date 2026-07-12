# M852 OrcaSlicer source crate partition checkpoint Spec

## Source boundary

This is a documentation/architecture rewrite-planning slice grounded in upstream build and source ownership files:

- `OrcaSlicer/src/CMakeLists.txt`
- `OrcaSlicer/src/libslic3r/CMakeLists.txt`
- `OrcaSlicer/src/libvgcode/CMakeLists.txt`
- `OrcaSlicer/src/slic3r/CMakeLists.txt`

It exists to keep the Rust rewrite aligned with `libslic3r` and rendering-neutral `libvgcode`, not to design an Ares-owned pipeline or speculative crate split.

## Required documentation behavior

The milestone must preserve these decisions:

- `libslic3r` is the unconditional core slicing-library source boundary.
- `libvgcode` is GUI-build-scoped upstream and mixes rendering-neutral data with OpenGL/viewer runtime.
- `slic3r` is native wxWidgets/OpenGL application and GUI integration; it is API-consumer evidence, not a current Rust crate owner.
- Active Rust workspace remains exactly four crates:
  - `crates/ares-core`
  - `crates/ares-vgcode`
  - `crates/ares-cli`
  - `crates/ares-wasm`
- No new workspace crate is created by M852.
- `AGENTS.md` `Workspace Crates` requires no active-list change because no crate is created.
- Candidate crate creation remains gated by a future source-cited milestone that proves the upstream source/API boundary, current pressure, and updates `Cargo.toml` plus `AGENTS.md`.
- Rejected speculative crates are recorded: `ares-geometry`, `ares-config`, `ares-gcode`, `ares-support`, `ares-ui`, and `ares-slic3r-gui`.

## Destination boundary

Documentation only:

- Create `docs/architecture/ard-0022-orcaslicer-source-crate-partition-checkpoint.md`.
- Update `docs/architecture/orcaslicer-source-structure.md` with build-system evidence for `libslic3r`, `libvgcode`, and `slic3r`.
- Create `docs/milestones/m852-orcaslicer-source-crate-partition-checkpoint.md`.
- Update `docs/roadmap.md` with M852.
- Create this spec and matching plan under `docs/superpowers/`.

## Scope stop

Do not create crates, move modules, change `Cargo.toml`, change `AGENTS.md`, add dependencies, add public APIs, port new `libslic3r` behavior, port `libvgcode` OpenGL/viewer runtime, port `slic3r` wxWidgets/OpenGL UI behavior, or implement slicing/G-code functionality.

## Acceptance criteria

- Independent planning review starts with `APPROVE`.
- Documentation records the four-crate decision and rejected speculative crates.
- Verification passes with `git diff --check`.
- Independent implementation review starts with `APPROVE` before commit.
- Commit and push the documentation milestone.
