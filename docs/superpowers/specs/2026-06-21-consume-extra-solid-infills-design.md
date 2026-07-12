# Consume extra_solid_infills Design

## Objective

Consume the existing OrcaSlicer `extra_solid_infills` option in Ares infill planning so configured sparse-infill layers become internal solid infill layers instead of remaining sparse. This is a concrete slicing/G-code behavior slice, not option metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2898-2904` defines `extra_solid_infills` as a string option. Its tooltip specifies accepted forms: `N` for every Nth layer, `N#K` for K consecutive solid layers every N layers where missing K means 1, and comma-separated explicit 1-based layer numbers.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3714-3719` applies the option in `PrintObject::discover_horizontal_shells()`: when `extra_solid_infills` is non-empty and `check_layer_id_pattern(value, i)` matches the current 0-based layer index, it converts surfaces of type `stInternal` to `stInternalSolid`.
- `OrcaSlicer/src/libslic3r/utils.cpp:1736-1818` implements `check_layer_id_pattern()`: it converts the current 0-based layer to 1-based, strips whitespace and one pair of surrounding quotes, treats comma-separated tokens as an explicit union, supports `base#count` inside comma lists as a non-repeating explicit range, and treats non-comma `N#K` as the repeating interval form.

## Rust Destination Boundary

- Runtime parsing belongs in `crates/ares-core/src/options/infill.rs` or a focused child module under `crates/ares-core/src/options/infill/`.
- Layer role selection belongs in `crates/ares-core/src/options/infill/layer_role.rs`, because Ares already decides between sparse, bottom surface, internal solid, and top surface there.
- Existing infill generation in `crates/ares-core/src/infills.rs` should keep using `InfillLayerRole`; the new option should change the role before pattern, spacing, print path, extrusion, speed, and G-code generation.

## Included Behavior

- Add a parsed `extra_solid_infills` schedule to `InfillOptions`.
- Empty or absent `extra_solid_infills` preserves existing behavior.
- For non-empty schedules, a matching layer that would otherwise be `InfillLayerRole::Sparse` becomes `InfillLayerRole::InternalSolid`.
- Bottom and top shell layers remain bottom/top shell roles even if the schedule also matches them, matching the upstream behavior where the option only converts `stInternal` surfaces.
- Matching layers use the existing internal-solid pattern, solid line width, solid direction/template, `solid_infill` print-path role, extrusion role, speed role, and G-code comments.
- The string grammar is limited to the upstream tooltip forms:
  - `N` means every Nth 1-based layer.
  - `N#K` means K consecutive 1-based layers every N layers.
  - `N#` means `N#1`.
  - comma-separated tokens combine as a union.
  - explicit layer numbers are 1-based.
  - within a comma-separated union, `N#K` means the explicit range `N..N+K`, matching upstream `check_layer_id_pattern()`, not a repeated interval.
  - whitespace and one pair of surrounding quotes are ignored before parsing.
- Invalid schedule tokens return `SliceError::InvalidInput` naming `extra_solid_infills`.

## Deferred Behavior

- Do not port unrelated horizontal shell discovery behavior from `PrintObject.cpp:3721+`.
- Do not implement new surface geometry, support interaction, UI behavior, profile compatibility, or Ares-owned pipeline design.
- Do not change `sparse_infill_density = 0`: existing no-infill behavior remains unchanged because there are no sparse internal surfaces to convert.
- Do not change bottom/top shell propagation semantics beyond preserving their existing role priority.

## Acceptance Criteria

- A focused options test proves the schedule parser accepts empty, explicit layer list, every-Nth, non-comma repeating `N#K`, non-comma `N#`, comma-list explicit `N#K` ranges, whitespace stripping, and one-pair surrounding quote stripping, and rejects invalid tokens.
- Parser tests prove non-comma `N#K` repeats every N layers, while `N#K` inside a comma-separated union is a one-time explicit range.
- An infill unit test proves a sparse-density middle layer matched by `extra_solid_infills` becomes `InfillRole::Solid` while adjacent unmatched sparse layers remain `InfillRole::Sparse`.
- A pipeline/G-code test proves `extra_solid_infills = "2"` emits `;PRINT_PATH:solid_infill:` and `;EXTRUSION:print:solid_infill:` on the second layer while preserving sparse infill on unmatched interior layers.
- A shell-priority test proves bottom/top shell layers remain bottom/top roles when the schedule matches every layer.
- Verification uses `cargo nextest run`, not `cargo test`.

## Verification

- RED: `cargo nextest run -p ares-core extra_solid_infills`
- GREEN: `cargo nextest run -p ares-core extra_solid_infills`
- Full: `cargo fmt --check`
- Full: `cargo nextest run --workspace`
- Full: `cargo clippy --workspace --all-targets -- -D warnings`
- Full: `cargo check -p ares-core --target wasm32-unknown-unknown`
- Full: `git diff --check`
- Full: touched Rust files stay at or below 400 LOC.

## Docs Impact

- Update `docs/roadmap.md` only if needed to mark the `extra_solid_infills` behavior as consumed in the relevant PrintConfig chain. Do not rewrite historical milestone evidence.
