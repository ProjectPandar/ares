# Consume Support Line Width Design

## Goal

Consume OrcaSlicer's existing `support_line_width` option in concrete Ares extrusion behavior for already-constructed support paths.

## Upstream Rewrite Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:960`
  - Declares `support_line_width` as `ConfigOptionFloatOrPercent` on `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6043-6053`
  - Defines label, `ratio_over = "nozzle_diameter"`, range `0..=1000`, and default `0`.
- `OrcaSlicer/src/libslic3r/Flow.cpp:54-55`
  - Maps `support_line_width` to `frSupportMaterial` for extrusion-width validation semantics.
- `OrcaSlicer/src/libslic3r/Flow.cpp:214-250`
  - `support_material_flow` uses `support_line_width` when positive and falls back to `line_width`.
  - `support_material_1st_layer_flow` uses `initial_layer_line_width` when positive, otherwise `support_line_width`, otherwise `line_width`.
  - `support_material_interface_flow` uses `support_line_width` when positive and falls back to `line_width`.

## Ares Destination Boundary

- `crates/ares-core/src/options/flow_ratios.rs`
  - Parse `support_line_width` through the existing `SliceOptions::extrusion_width` helper and pass it into `ExtrusionOptions`.
- `crates/ares-core/src/extrusions/options.rs`
  - Store support line width in `ExtrusionOptions`.
- `crates/ares-core/src/extrusions/options/accessors.rs`
  - Use support line width for `PrintPathRole::SupportMaterial` and `PrintPathRole::SupportMaterialInterface` width selection.
  - Preserve the existing first-layer `initial_layer_line_width` override for both support roles.
- Existing support path test modules under `crates/ares-core/src/pipeline/tests/`.

## Included Behavior

- Parse numeric and percent-string `support_line_width` over nozzle diameter, using the existing Ares extrusion-width parser.
- Keep Orca default `support_line_width = 0`, meaning support roles fall back to the configured `line_width` and then Ares' existing automatic width fallback when `line_width = 0`.
- Make non-first-layer `PrintPathRole::SupportMaterial` extrusion deltas change when `support_line_width` changes.
- Make non-first-layer `PrintPathRole::SupportMaterialInterface` extrusion deltas change when `support_line_width` changes.
- Keep `initial_layer_line_width` taking precedence over `support_line_width` on first-layer support material and support interface paths.
- Preserve existing `support_flow_ratio`, `support_interface_flow_ratio`, `first_layer_flow_ratio`, speed, fan, role mapping, and support-interface behavior apart from the intended width selection.
- Preserve existing validation behavior for invalid `support_line_width` values through `SliceOptions::extrusion_options()`.
- Keep `ares-core` platform-neutral and WASM-safe; no file I/O, terminal behavior, UI, OpenGL, or new dependencies.
- Keep every touched Rust file at or below 400 LOC.

## Deferred Behavior

- Full support generation from overhang geometry.
- Tree-support geometry, line spacing, roof/tip placement, and `TreeSupport*` behavior.
- `support_transition_line_width`, `support_roof_line_width`, `support_bottom_line_width`, and transition extrusion roles.
- Multi-extruder nozzle selection for support/support-interface filaments beyond Ares' current first-value path.
- Full Orca `Flow` class parity and support extrusion-width validation diagnostics beyond the existing Ares validation path.
- Any UI, preset, generated config class, or metadata-only milestone changes not needed to consume this option in slicing/G-code behavior.

## Acceptance Criteria

- `support_line_width` changes `;EXTRUSION:print:support_material:` E deltas for a manually constructed support material path without changing speed markers.
- `support_line_width` changes `;EXTRUSION:print:support_material_interface:` E deltas for a manually constructed support interface path without changing speed markers.
- `"150%"` with a `0.4` mm nozzle behaves like `0.6` mm support width.
- Omitted and explicit zero `support_line_width` preserve line-width fallback behavior.
- First-layer support material and support interface paths still use `initial_layer_line_width` when it is positive, even if `support_line_width` is set.
- Invalid `support_line_width` values still surface as `SliceError::InvalidInput`.
- Focused tests run with `cargo nextest run -p ares-core support_line_width`.
- Adjacent support tests run with `cargo nextest run -p ares-core support_speed_flow support_interface_speed_flow`.
- Full verification runs with `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and a touched Rust LOC guard.
