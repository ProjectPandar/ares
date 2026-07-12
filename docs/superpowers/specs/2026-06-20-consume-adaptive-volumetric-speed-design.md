# Consume Adaptive Volumetric Speed Design

## Scope

Consume the existing OrcaSlicer `filament_adaptive_volumetric_speed` and `volumetric_speed_coefficients` options into concrete Ares speed/G-code behavior.

This is a source-cited Rust rewrite slice of:

- `OrcaSlicer/src/libslic3r/GCode.cpp:6253-6268`, `GCode::calc_max_volumetric_speed`
- `OrcaSlicer/src/libslic3r/GCode.cpp:6484-6492`, `_extrude` adaptive filament volumetric cap selection
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1318-1319`, `filament_adaptive_volumetric_speed` and `volumetric_speed_coefficients` option tuple declarations
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2557-2568`, option defaults and labels

The Rust destination boundary is the existing `ares-core` speed option parsing and volumetric speed cap compatibility path:

- `crates/ares-core/src/options/volumetric_speed.rs`
- `crates/ares-core/src/options/speed.rs`
- `crates/ares-core/src/extrusions.rs`
- `crates/ares-core/src/speeds/config.rs`
- `crates/ares-core/src/speeds/config/accessors.rs`
- `crates/ares-core/src/speeds/volumetric.rs`
- focused tests under `crates/ares-core/src/options/tests/`, `crates/ares-core/src/extrusions/tests.rs`, `crates/ares-core/src/speeds/volumetric/tests.rs`, and `crates/ares-core/src/pipeline/tests/filament_max_volumetric_speed.rs`

## Included Behavior

When `filament_adaptive_volumetric_speed` is true for the active first filament, Ares computes the fitted max volumetric speed with Orca's six-coefficient polynomial:

```text
c0 * layer_height^2 + c1 * line_width^2 + c2 * layer_height * line_width + c3 * layer_height + c4 * line_width + c5
```

In Orca `GCode.cpp`, this fitted value is applied to the `_extrude` autospeed branch where a path speed of `0` is converted into `filament_max_volumetric_speed / _mm3_per_mm`; the later hard cap still uses the original `filament_max_volumetric_speed`. Ares does not yet have Orca's zero-speed autospeed sentinel because current role speed parsing resolves positive speeds before the speed stage. This slice therefore maps the fitted value into Ares' existing `filament_max_volumetric_speed` cap compatibility shell instead of adding a new Ares speed pipeline: for print moves with geometry, the current Ares volumetric cap uses `min(filament_max_volumetric_speed, fitted_value)` as its per-move cap. If `filament_max_volumetric_speed` is zero, adaptive volumetric speed is disabled, the move has no adaptive geometry, or the coefficient string/result is unusable, existing cap behavior stays unchanged.

This compatibility mapping is intentionally narrow. It consumes the already-scaffolded option into the only current Ares destination that represents `filament_max_volumetric_speed`; it does not claim exact Orca `_extrude` speed-selection parity until the zero-speed autospeed sentinel and role speed selection path are ported.

`filament_adaptive_volumetric_speed` accepts a boolean scalar or boolean list and uses the first filament entry. Missing values default to false.

`volumetric_speed_coefficients` accepts either a string scalar or a string list and uses the first filament entry. A usable coefficient string contains exactly six complete finite `f64` values separated by the literal ASCII space character `U+0020`; repeated spaces are ignored. Examples:

- Usable: `"0 0 0 0 0 1"` and `["0  0 0 0 0 1"]`
- Unusable: `""`, `"0 0 0"`, `"0 0 0 0 0 0"`, `"0 0 0 0 0 NaN"`, `"0\t0 0 0 0 1"`, `"0\n0 0 0 0 1"`, and `"bad 0 0 0 0 1"`

Omitted, default empty, malformed, tab/newline-delimited, non-finite, wrong-count, or six-zero coefficient strings disable only the adaptive fitted cap and leave the existing `filament_max_volumetric_speed` cap in place. This follows the upstream shape of `calc_max_volumetric_speed` returning `numeric_limits<double>::max()` when it cannot build a six-value nonzero coefficient vector, while preserving Ares' API boundary rule that invalid external option text must not produce invalid feedrates.

The polynomial result must be finite and greater than zero to affect speed. A fitted result that is `0`, negative, `NaN`, or infinite disables only the adaptive fitted cap for that move and leaves the existing `filament_max_volumetric_speed` cap in place.

The fitted cap uses per-move layer height and line width available in Ares:

- `ExtrusionMove` carries the `effective_layer_height_mm` used by `generate_extrusion_moves` for that print move. This is the toolpath move's carried effective height when present, otherwise the current `Layer::height()`.
- `ExtrusionMove` carries the `effective_line_width_mm` selected by the same layer-aware width branch used by `ExtrusionOptions::extrusion_per_mm_for_layer`: first-layer line width overrides eligible non-bridge roles on layer zero, otherwise the role width comes from `ExtrusionOptions::width_for_role(role)`. This keeps adaptive geometry aligned with the current extrusion volume calculation.
- Travel moves carry no adaptive geometry and are never adaptive-capped.

## Deferred Behavior

This slice does not implement full Orca `PressureEqualizer`, `max_volumetric_extrusion_rate_slope`, `max_volumetric_extrusion_rate_slope_segment_length`, `extrusion_rate_smoothing_external_perimeter_only`, arc fitting disabling, multi-filament active extruder switching, wipe tower behavior, support generation, or any new public Ares pipeline design.

It also does not implement Orca's zero-speed autospeed sentinel or exact `GCode::_extrude` role speed selection. Those remain deferred to a future source-cited `GCode::_extrude` speed-selection slice.

It also does not change existing metadata-only `PrintConfig.hpp` milestone modules except where tests prove runtime behavior for existing option keys.

No user documentation, architecture decision record, or roadmap update is required beyond this SDD spec and its implementation plan; this is a narrow runtime consumption slice.

## Acceptance Criteria

- A focused option test proves `filament_adaptive_volumetric_speed` and `volumetric_speed_coefficients` are parsed into `SpeedOptions`.
- Focused option tests prove omitted/default empty coefficients do not fail, malformed, tab/newline-delimited, wrong-count, non-finite, and six-zero coefficients do not fail, and these unusable coefficients disable only the adaptive fitted cap.
- A focused extrusion test proves generated print moves carry effective layer height and role-derived line width, including layer-height fallback, while travel moves carry no adaptive geometry.
- Focused speed tests prove an adaptive fitted max below `filament_max_volumetric_speed` lowers the print move speed through the existing cap path, while six-zero coefficients and non-positive/non-finite fitted results leave the existing filament max cap unchanged.
- A pipeline/G-code test proves enabling adaptive volumetric speed with coefficients below the configured filament max emits a lower `;SPEED:print:...` feedrate than the same print with adaptive disabled.
- Existing `filament_max_volumetric_speed` behavior remains unchanged when adaptive volumetric speed is omitted or false.
- New tests and verification commands use `cargo nextest run`, not `cargo test`.
- Touched Rust source files remain at or below 400 LOC.
- `ares-core` remains platform-neutral and WASM-compatible: no filesystem, terminal, UI, OpenGL, or native-only behavior.

## Verification

Required fresh verification before commit:

- `cargo fmt --check`
- focused RED/GREEN with `cargo nextest run -p ares-core adaptive_volumetric`
- `cargo nextest run -p ares-core filament_max_volumetric_speed`
- `cargo nextest run -p ares-core volumetric`
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- touched Rust LOC guard
