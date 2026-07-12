# M20: libslic3r crate boundary foundation

## Goal
Study OrcaSlicer's source structure and record Ares workspace crate boundaries before larger `libslic3r` ports.

## Exit checklist
- `docs/architecture/orcaslicer-source-structure.md` summarizes `libslic3r` and `libvgcode` source boundaries with concrete paths.
- `docs/architecture/ard-0019-workspace-crate-boundaries.md` decides the active workspace remains `ares-core` plus `ares-cli` for now.
- `AGENTS.md` `Workspace Crates` distinguishes active crates from candidate non-member crates.
- `ares-core` preserves its async byte-in/options to byte-output API shape.
- `ares-cli` preserves `ares slice --options option.json -o output.gcode input.stl` as the filesystem-facing STL slicing command.
- No new workspace crates, dependencies, or slicing behavior are introduced.
- Later geometry/model/config implementation work is tracked in M21 or later milestones.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
