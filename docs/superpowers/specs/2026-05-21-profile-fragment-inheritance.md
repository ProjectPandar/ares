# Profile Fragment Inheritance Spec

## Goal
Advance Orca profile parity by adding a WASM-safe, in-memory profile fragment model that can resolve simple `inherits` chains into `SliceOptions` without introducing filesystem preset loading or a new crate.

## OrcaSlicer structure research summary
- `OrcaSlicer/src/libslic3r/Preset.hpp` defines preset directories and types: `process`, `filament`, and `machine`, plus JSON keys including `type`, `name`, `inherits`, `from`, `setting_id`, and `instantiation`.
- `OrcaSlicer/src/libslic3r/Preset.cpp::PresetCollection::load_presets` loads JSON profiles, reads `inherits`, finds the parent preset, and applies parent values before child values.
- `OrcaSlicer/src/libslic3r/Preset.cpp::PresetCollection::get_preset_parent` and `get_preset_base` traverse parent chains by the `inherits` field.
- `OrcaSlicer/resources/profiles/**` stores process, filament, and machine JSON fragments. Child fragments such as `OrcaSlicer/resources/profiles/OrcaArena/process/fdm_process_arena_0.20.json` inherit common fragments such as `fdm_process_arena_common`, which itself inherits `fdm_process_common`.
- Ares keeps `ares-core` filesystem-free, so M7 ports the in-memory inheritance semantics but not resource-directory discovery.

## Scope
Milestone 7 adds `ProfileFragment` parsing from JSON bytes and deterministic resolution of same-kind profile inheritance chains. It produces `SliceOptions` by merging parent values first and child values second. Unknown keys remain preserved for later option milestones.

## Functional requirements
1. `ares-core` exposes:
   ```rust
   pub enum ProfileKind { Process, Filament, Machine }
   pub struct ProfileFragment { ... }
   pub fn merge_profile_fragments(
       fragments: &[ProfileFragment],
       target_kind: ProfileKind,
       target_name: &str,
   ) -> Result<SliceOptions, SliceError>;
   ```
2. `ProfileFragment::from_json_bytes(input)` parses one Orca-style JSON object from bytes.
3. Required fragment fields:
   - `type`: must be one of `process`, `filament`, or `machine`.
   - `name`: must be a non-empty string.
4. Optional metadata fields:
   - `inherits`: non-empty string means the parent fragment name.
   - `from`, `setting_id`, and `instantiation` are parsed as optional strings but also remain in the raw values map.
5. `ProfileFragment` exposes `kind()`, `name()`, `inherits()`, and `values()` accessors.
6. `merge_profile_fragments` resolves `(target_kind, target_name)`, follows same-kind `inherits` parents, and returns a `SliceOptions` containing the merged key/value map.
7. Merge order is parent first, child second. Child values override parent values for identical keys. Metadata keys such as `type`, `name`, and `inherits` remain present from the final child unless overridden by normal merge order.
8. Missing target, missing parent, duplicate `(kind, name)` fragments, cross-kind parent matches, and inheritance cycles return `SliceError::InvalidInput`.
9. The resolver is deterministic and independent of input fragment ordering.
10. Existing `SliceOptions` typed accessors work on merged profile output.
11. `slice(input, merged_options)` works with merged options and emits metadata from inherited hardware/layer options.
12. No new crates or dependencies are introduced.
13. `ares-core` remains platform-neutral and performs no filesystem I/O.
14. Modified Rust files remain under 400 LOC.

## Non-goals
- No filesystem resource scanning, profile directory discovery, CLI multi-profile arguments, vendor bundle metadata loading, alias/renamed profile handling, compatibility filtering, or substitution logging.
- No cross-kind composition of process + filament + machine into one full print config; M7 resolves one same-kind inheritance chain at a time.
- No implementation of additional typed options beyond those already supported by `SliceOptions`.
- No new workspace crates.

## Acceptance criteria
- Core tests cover parsing process/filament/machine fragments, rejecting malformed JSON and invalid/missing required fields, deterministic parent-child-grandchild merge, input-order independence, child override behavior, duplicate names, missing target, missing parent, cross-kind parent rejection, cycle rejection, unknown-key preservation, and typed `SliceOptions` accessors on merged output.
- A core async `slice` test uses merged profile output and proves inherited `layer_height` and hardware metadata affect generated G-code.
- Docs include `docs/milestones/m7-profile-fragment-inheritance.md`, this spec, an implementation plan, roadmap update, and an ARD for in-memory profile inheritance before filesystem profile loading.
- Independent plan/spec review returns APPROVE before implementation.
- Independent implementation review returns APPROVE before commit.
- `cargo fmt --check`, `cargo test`, and `cargo clippy --workspace --all-targets -- -D warnings` pass.
