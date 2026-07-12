# M7: Profile fragment inheritance

## Goal
Resolve OrcaSlicer-style in-memory profile fragments into `SliceOptions` through deterministic same-kind inheritance chains.

## Exit checklist
- `ares-core` exposes `ProfileKind`, `ProfileFragment`, and `merge_profile_fragments`.
- `ProfileFragment::from_json_bytes` parses process, filament, and machine JSON fragments from bytes.
- Required `type` and `name` fields are validated.
- Optional `inherits`, `from`, `setting_id`, and `instantiation` metadata are accessible or preserved.
- Parent values merge before child values, with child overrides.
- Missing targets, missing parents, duplicate names, cross-kind parents, and cycles return typed `SliceError::InvalidInput`.
- Merged output is a `SliceOptions` preserving unknown keys and supporting existing typed option accessors.
- `slice` works with merged profile options.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No profile directory scanning, vendor bundle loading, compatibility filtering, aliases, substitutions, or CLI multi-profile arguments.
- No cross-kind full print config composition.
- No new workspace crates.
