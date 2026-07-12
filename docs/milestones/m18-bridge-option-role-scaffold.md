# M18: Bridge option and role scaffold

## Goal
Add typed Orca bridge options plus bridge print-role flow and speed behavior before bridge geometry detection is implemented.

## Exit checklist
- `ares-core` exposes `BridgeOptions`.
- `SliceOptions` parses `bridge_flow`, `internal_bridge_flow`, `bridge_speed`, `internal_bridge_speed`, `bridge_no_support`, and `thick_bridges` with Orca defaults and validation.
- `PrintPathRole::Bridge` exists with `as_str() == "bridge"`.
- Bridge extrusion applies `bridge_flow` while existing roles keep prior extrusion behavior.
- Bridge speed uses `bridge_speed` while existing roles keep prior speed behavior.
- No bridge geometry or support generation is introduced.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No bridge detection, bridge direction optimization, unsupported-region geometry, support generation, or exact Orca bridge parity.
- No new workspace crates.
