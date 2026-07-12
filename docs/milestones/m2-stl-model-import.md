# M2: STL model import

## Goal
Parse STL model bytes inside `ares-core` so the first slicer API consumes real triangle geometry before later slicing pipeline milestones.

## Exit checklist
- `ares-core` exposes `load_model(input) -> Result<Model, SliceError>` without direct filesystem access.
- `Model` records `InputFormat` and imported triangle geometry.
- ASCII STL import handles LF and CRLF `vertex x y z` input.
- Binary STL import handles standard 80-byte headers, little-endian triangle counts, and 50-byte triangle records.
- Malformed STL input returns a typed `SliceError` instead of placeholder success.
- `slice` uses imported model metadata and emits deterministic placeholder G-code including `triangle_count`.
- `ares-cli` still owns filesystem reads/writes and extension validation.
- No new crates are introduced for M2.
- 3MF geometry extraction is explicitly deferred to a later archive/XML milestone.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No layer planning or path generation.
- No full 3MF geometry extraction.
- No typed Orca profile option mapping.
- No WASM binding crate.
