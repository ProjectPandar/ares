# Consume Support Ironing G-code Label Design

## Goal

Consume the existing `support_ironing` behavior further by making support-derived ironing paths carry Orca's support ironing G-code label semantics in Ares diagnostics and move comments, instead of appearing as ordinary `ironing`.

## Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:997-1000` declares `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, and `support_ironing_spacing`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6446` defines the support ironing option defaults, bounds, and user-facing meaning: support interface is printed again with small flow.
- `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:58-61` transfers support ironing settings into support parameters.
- `OrcaSlicer/src/libslic3r/Support/SupportCommon.cpp:1877-1907` generates contact-layer support ironing with `ExtrusionRole::erIroning` and support ironing flow.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6110-6140` emits support fills with distinct labels, including `support_ironing_label = "support ironing"` when the support fill role is `erIroning`.

## Current Ares State

- `crates/ares-core/src/print_paths/support_ironing.rs` already duplicates `PrintPathRole::SupportMaterialInterface` paths as `PrintPathRole::Ironing` when `support_ironing` is enabled.
- The duplicate path records `.with_extrusion_role(PrintPathRole::SupportMaterialInterface)`, which identifies it as support-derived ironing rather than ordinary top-surface ironing.
- `crates/ares-core/src/extrusions.rs` currently uses that `extrusion_role` to compute extrusion width and support ironing flow, but it does not retain the role override for later G-code diagnostics.
- `crates/ares-core/src/gcode_move_emit.rs` and `crates/ares-core/src/gcode_layer_diagnostics.rs` emit labels from `role().as_str()`, so support-derived ironing appears as `ironing`.

## Design

Add a small role-label helper in `crates/ares-core/src/print_paths.rs` for diagnostic/comment output:

- `PrintPathRole::Ironing` with `extrusion_role == Some(PrintPathRole::SupportMaterialInterface)` maps to `support_ironing`.
- Every other role maps to the existing `PrintPathRole::as_str()` value.

This maps Orca's human label `"support ironing"` into Ares' existing snake_case diagnostic token style as `support_ironing`. It does not add a public `PrintPathRole` variant because the internal motion behavior must remain the same ironing role with a support-interface extrusion override.

Propagate the optional `extrusion_role` through generated move artifacts only far enough for comments:

- `ToolpathMove` already carries `extrusion_role`.
- `ExtrusionMove` should retain `extrusion_role` from `ToolpathMove`.
- `SpeedMove` should retain `extrusion_role` from `ExtrusionMove`.

Use the diagnostic label helper for:

- `;PRINT_PATH:<role>:...`
- `;SPEED:<kind>:<role>:...`
- `;EXTRUSION:<kind>:<role>:...`
- `;MOVE:<kind>:<role>:...`

Keep all behavioral role decisions unchanged:

- Speed lookup still uses `PrintPathRole::Ironing`, so support ironing keeps `ironing_speed`.
- Extrusion geometry still uses `SupportMaterialInterface` through the existing `extrusion_role` override.
- Fan, pressure advance, role-change custom G-code, acceleration, jerk, slow-down, and path ordering behavior remain unchanged.

## Acceptance Criteria

1. With `support_ironing = true`, support interface output contains a support-interface path followed by support ironing output labeled as `support_ironing`.
2. Support-derived ironing emits `;PRINT_PATH:support_ironing:`, `;SPEED:print:support_ironing:`, `;EXTRUSION:print:support_ironing:`, and `;MOVE:print:support_ironing:`.
3. Support-derived ironing still uses the configured ordinary ironing print speed for feedrate selection.
4. Support ironing extrusion deltas remain controlled by `support_ironing_flow` through the support-interface extrusion role override.
5. Ordinary independent ironing paths still emit `ironing`, not `support_ironing`.
6. Invalid `support_ironing` and `support_ironing_flow` option behavior remains unchanged.
7. Focused verification uses `cargo nextest run -p ares-core support_ironing`.
8. Full verification uses `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, `git diff --cached --check`, and a touched Rust LOC guard.

## Deferred Behavior

- Full Orca support polygon/contact-layer generation beyond the existing Ares support-interface compatibility shell.
- Changing internal role enums or adding a public support-ironing role.
- Orca's exact space-containing label text in raw comments; Ares diagnostic tokens stay snake_case.
- Support transition labels and broader support G-code label parity.
- Role-change custom G-code placeholder semantics for support-derived ironing.
- Fan, pressure advance, acceleration, jerk, slow-down, and volumetric policy changes for support-derived ironing.
- Multi-extruder support ownership or wipe-tower interactions.

## Docs Impact

After implementation approval, update `docs/roadmap.md` to add this runtime slice near the other support ironing entries. The update must record that support-derived ironing diagnostics and move comments now emit `support_ironing`, and it must revise earlier support ironing roadmap text that currently says distinct support-ironing labels are deferred or that support ironing keeps public `ironing` comments.

## Safety and Rollback

The slice changes only in-memory Rust data and generated G-code comments/diagnostics. It adds no dependencies, file I/O, UI, terminal behavior, OpenGL behavior, or platform-specific behavior to `ares-core`. Rollback is limited to removing the diagnostic label helper, the two move-artifact `extrusion_role` fields, and the focused test assertion changes.

## Spec Self-Review

- Placeholder scan: no TBD/TODO placeholders remain.
- Scope check: single concrete G-code/comment behavior slice; no support-generation expansion.
- Ambiguity check: support-derived ironing is explicitly defined as `role == Ironing` plus `extrusion_role == SupportMaterialInterface`.
- Consistency check: internal behavior remains unchanged; only diagnostic/comment labels use `support_ironing`.
