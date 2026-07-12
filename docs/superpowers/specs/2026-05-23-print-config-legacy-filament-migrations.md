# M178 Spec: PrintConfig legacy filament migration slice

## Goal
Port the `filament_map_mode` and `filament_type` legacy branches from `libslic3r::PrintConfigDef::handle_legacy` into `ares-core` option ingestion.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8020-8045`: migrate `filament_map_mode` `Auto` and normalize `filament_type` `ASA-Aero` tokens.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8046+`: prime-tower rib migrations and all later legacy handling.
- Any changes to option definitions or registry metadata.
- Typed accessors or runtime behavior changes beyond ingestion-time key/value normalization.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/legacy.rs`: extend the existing ordered legacy normalization with only the M178 filament branches.
- `crates/ares-core/src/options/tests/legacy_filament_migrations.rs`: add focused M178 tests proving included value migrations, token list rebuilding, non-matching preservation, non-string preservation, and unknown-key preservation without growing existing test modules past the 400 LOC limit.
- `crates/ares-core/src/options/tests.rs`: register the M178 test module.
- `docs/roadmap.md` and `docs/milestones/m178-print-config-legacy-filament-migrations.md`: milestone sequencing docs.

## Included legacy rewrites

`filament_map_mode` (`PrintConfig.cpp:8020-8022`):

- string `Auto` becomes `Auto For Flush`
- all other values remain unchanged

`filament_type` (`PrintConfig.cpp:8023-8045`):

- split the string value on semicolons using upstream `std::getline` semantics: preserve leading and middle empty tokens, but do not emit a final empty token for a trailing semicolon
- for each token, if it starts and ends with `"` and has at least two bytes, strip the surrounding quotes before comparison
- if a token equals `ASA-Aero`, rewrite it to `ASA-AERO` and mark the value for rebuilding
- preserve other tokens after the same quote-stripping step only when rebuilding is needed
- if any token was rewritten, rebuild the whole list as `"token"` entries joined by `;`
- if no token was rewritten, preserve the original string exactly
- non-string values remain unchanged

## Functional requirements

1. Apply the included rewrites when `SliceOptions` is deserialized from JSON.
2. Rewrite only exact `filament_map_mode` string `Auto`; do not rewrite case variants or substrings.
3. Rewrite only exact `filament_type` token `ASA-Aero`; do not rewrite case variants or substrings.
4. Rebuild the full `filament_type` list with quotes only if at least one token was rewritten.
5. Preserve non-matching `filament_type` strings exactly when no token is rewritten.
6. Preserve non-string values for covered keys unchanged.
7. Preserve non-legacy unknown options exactly as today.
8. Preserve existing `SliceOptions::values()` API shape and all option accessor behavior except that covered legacy inputs are stored with migrated values according to the source-cited branch.
9. Do not add new public API, crates, dependencies, option definitions, registry metadata, pipeline stages, filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior.
10. Do not implement any `handle_legacy` behavior from `PrintConfig.cpp:8046+` in this milestone.

## Acceptance checks

- Tests prove `filament_map_mode: "Auto"` becomes `Auto For Flush`.
- Tests prove non-matching string and non-string `filament_map_mode` values remain unchanged.
- Tests prove unquoted `filament_type` token `ASA-Aero` becomes rebuilt quoted token `"ASA-AERO"`.
- Tests prove quoted `filament_type` token `"ASA-Aero"` becomes `"ASA-AERO"`.
- Tests prove mixed token lists are rebuilt with every token quoted when at least one `ASA-Aero` token is present.
- Tests prove trailing semicolon behavior matches upstream `std::getline` semantics: `ASA-Aero;` rebuilds as `"ASA-AERO"` without a quoted trailing empty token.
- Tests prove non-matching `filament_type` strings remain exactly unchanged when no rewrite is needed.
- Tests prove non-string `filament_type` values remain unchanged.
- Tests prove unknown non-legacy keys remain preserved.
- Existing legacy/registry/option tests continue to pass.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:8046+` behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
