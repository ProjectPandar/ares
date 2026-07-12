# M852: OrcaSlicer source crate partition checkpoint

## Source boundary

This milestone is a source-structure documentation slice grounded in these upstream files:

- `OrcaSlicer/src/CMakeLists.txt`
- `OrcaSlicer/src/libslic3r/CMakeLists.txt`
- `OrcaSlicer/src/libvgcode/CMakeLists.txt`
- `OrcaSlicer/src/slic3r/CMakeLists.txt`

It preserves the project direction: Rust rewrites of `libslic3r` and rendering-neutral `libvgcode`, not an Ares-owned pipeline or speculative crate split.

## Included behavior

- Record that `libslic3r` is the unconditional core slicing-library source boundary.
- Record that `libvgcode` is GUI-build-scoped upstream and mixes rendering-neutral data with OpenGL/viewer runtime.
- Record that `slic3r` is native GUI/application integration and should be treated as API-consumer evidence, not as current Rust crate ownership.
- Confirm the active Rust workspace remains four crates: `ares-core`, `ares-vgcode`, `ares-cli`, and `ares-wasm`.
- Record rejected speculative crates: `ares-geometry`, `ares-config`, `ares-gcode`, `ares-support`, `ares-ui`, and `ares-slic3r-gui`.
- Preserve the rule that future crate creation must cite an upstream source/API boundary, prove current pressure, update `Cargo.toml`, and update `AGENTS.md` `Workspace Crates`.

## Scope stop

Do not create new crates, move Rust modules, add dependencies, change public APIs, port additional `libslic3r` behavior, port `libvgcode` viewer runtime, port wxWidgets/OpenGL UI behavior, or implement new slicing/G-code functionality in this milestone.

## Exit criteria

- `docs/architecture/ard-0022-orcaslicer-source-crate-partition-checkpoint.md` exists and records the accepted crate-partition decision.
- `docs/architecture/orcaslicer-source-structure.md` includes build-system evidence for `libslic3r`, `libvgcode`, and `slic3r`.
- `docs/roadmap.md` includes M852.
- `AGENTS.md` requires no active crate list change because no new crate is created.
- Independent planning/implementation review starts with `APPROVE`.
- Verification passes with `git diff --check`.
- Commit and push the documentation milestone.
