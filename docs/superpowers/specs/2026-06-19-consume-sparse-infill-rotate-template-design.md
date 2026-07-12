# Consume Sparse Infill Rotate Template Design

## Goal

Port a concrete `libslic3r` sparse infill behavior slice into Ares: `sparse_infill_rotate_template` must alter generated sparse infill paths and the resulting G-code artifacts instead of being only recorded as option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3873-3884` registers `sparse_infill_rotate_template` as a string option. Its tooltip says plain comma-separated degree values repeat by layer and that a non-empty template ignores the standard infill direction.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:52-190` implements `calculate_infill_rotation_angle`. When the template string is empty it returns `infill_direction`. When the template has no metalanguage symbols, it deserializes the string as floats and picks `values[layer_id % values.len()]`.
- `OrcaSlicer/src/libslic3r/Config.hpp:868-886` is the `ConfigOptionFloats` deserializer used by the simple-list branch. It splits on commas and trims each comma-separated item.
- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:935-937` wires sparse internal infill to `calculate_infill_rotation_angle(..., region_config.infill_direction.value, region_config.sparse_infill_rotate_template.value)` and sets `fixed_angle` when the sparse template is non-empty.

## Ares Boundary

- Parse the runtime option in `crates/ares-core/src/options/infill.rs`.
- Store the parsed plain sparse rotation angle template on `InfillOptions`.
- Move sparse infill pass angle selection out of the nearly-full `crates/ares-core/src/infills.rs` into a small `crates/ares-core/src/infills/rotation.rs` module.
- Consume the parsed template when `generate_infills` creates `InfillPasses`, so sparse infill paths and downstream G-code comments change for layers selected by the template.

## Included Behavior

- Empty or missing `sparse_infill_rotate_template` preserves current Ares sparse infill angle behavior.
- Plain angle templates accept unsigned, finite numeric degree tokens separated by commas, for example `0,90` and `0, 90`.
- Plain angle templates trim whitespace around comma-separated tokens, but whitespace alone is not a separator.
- Plain angle templates reject empty tokens, trailing commas, signs, percentages, and every advanced metalanguage character handled by Orca's non-simple branch.
- Plain angle templates repeat by layer using `layer_id % template_len`, matching Orca's simple-list branch.
- A non-empty parsed template overrides `infill_direction` for sparse infill angle selection and suppresses the current rectilinear odd-layer `+90` fallback for the template-selected base pass, matching Orca's `fixed_angle` wiring.
- Grid sparse infill still adds a perpendicular pass from the template-selected base angle because Ares' existing grid scaffold represents grid as paired perpendicular sparse passes.
- Invalid runtime values fail during option parsing with `SliceError::InvalidInput` and name `sparse_infill_rotate_template`.

## Deferred Behavior

- Orca's advanced rotation metalanguage in `Fill.cpp:61-176`, including relative tokens such as `+5`, repetition/range syntax such as `+5#5`, units, random joints, one-time tokens, and bottom/top shell counts.
- `solid_infill_rotate_template`, ironing angle reuse, and non-sparse fill roles.
- `align_infill_direction_to_model`.
- Sparse infill patterns that Ares currently rejects or only scaffolds differently than Orca's full fill classes.

## Acceptance Criteria

1. A parser test proves `sparse_infill_rotate_template = "0, 90"` is accepted and stored as two angles, while invalid strings such as `"+5"`, `"0 90"`, `"90,"`, or `"bad"` return `SliceError::InvalidInput` naming `sparse_infill_rotate_template`.
2. A sparse infill unit test proves rectilinear layer 1 uses template angle `0` from `"90,0"` instead of the current no-template odd-layer rotation derived from `infill_direction`.
3. A pipeline/G-code test proves the template changes real artifacts: two-layer rectangular slicing with `sparse_infill_rotate_template = "90,0"` produces the template-selected layer 1 sparse infill path and matching `;INFILL:sparse:` / `;PRINT_PATH:sparse_infill:` comments.
4. Existing no-template sparse infill behavior remains covered by existing tests.
5. All touched Rust files remain at or below 400 LOC.
6. Verification must include targeted tests, `cargo test -p ares-core --lib`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and the repo LOC gate.

## Safety and Documentation Impact

The change is local to `ares-core` option parsing and sparse infill generation. It introduces no new dependencies and no filesystem, UI, terminal, OpenGL, or platform-specific behavior. Documentation impact is this spec plus the implementation plan; milestone docs do not need another option-only entry because this task exists specifically to convert an already-recorded option boundary into executable slicing behavior.
