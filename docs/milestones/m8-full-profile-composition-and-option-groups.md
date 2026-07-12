# M8: Full profile composition and option groups

## Goal
Compose resolved process, filament, and machine profile groups into a single `SliceOptions` value suitable for the current slicer API.

## Exit checklist
- `ares-core` exposes `ProfileSelection`, `ComposedProfile`, and `compose_profile_fragments`.
- Selection validation rejects empty process, machine, or filament selections.
- Composition resolves process, machine, and one or more filament fragments through the M7 inheritance resolver.
- Composition order is deterministic: machine, process, then filament values.
- Multi-filament JSON-level composition uses deterministic union-key collection, flattens mixed scalar/array values, and preserves current typed hardware accessors.
- Profile-local conflict keys are removed from final composed options.
- Final options include profile ID keys, Orca-aligned default filament map, inherits group, compatibility group metadata, and filament IDs for the M8 subset.
- Unknown keys are preserved unless explicitly removed as profile-local keys.
- `slice` works with composed profile options.
- `ares-core` remains WASM-safe and filesystem-free.
- No new crates or dependencies are introduced.
- Modified Rust files remain under 400 LOC.
- `cargo test`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## Non-goals
- No profile directory scanning, vendor bundle loading, compatibility expression evaluation, placeholder substitution, aliases, project config loading, CLI multi-profile arguments, or legacy fallback.
- No complete Orca typed option registry or extruder-variant normalization.
- No new workspace crates.
