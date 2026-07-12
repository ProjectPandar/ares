# Consume support_interface_not_for_body filament selector design

## Source boundary

This slice ports the first concrete downstream behavior of OrcaSlicer's `support_interface_not_for_body` option into Ares' current extrusion-hardware selector boundary.

Upstream sources:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:961` declares `support_interface_not_for_body` in `PrintObjectConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6036-6041` defines the option as `Avoid interface filament for base`, category `support`, default `true`, and describes the behavior as avoiding support-interface filament for the support base if possible.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1200` treats changes to `support_interface_not_for_body` as support-affecting invalidation input.
- `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp:740-770` chooses a support-base extruder when support material is present, support material is "don't care", support interface material is fixed, no object material is already on the layer, and both support and interface material are present. When `support_interface_not_for_body` is true, the helper skips the fixed interface extruder while choosing a support-base extruder.
- `OrcaSlicer/src/libslic3r/GCode/ToolOrdering.cpp:1688-1704` keeps flush-into-support overrides from using a fixed support-interface filament for support body when `support_interface_not_for_body` is true.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4768-4784` contains disabled BBS fallback code with the same intent: do not print support base with the interface filament when avoid-interface-for-body is enabled.
- `OrcaSlicer/src/libslic3r/Preset.cpp:1086` includes the option in print preset compatibility.

## Ares destination boundary

The Rust destination is Ares' current role-hardware selector plus a support-material extrusion-only hardware override used by extrusion generation:

- `crates/ares-core/src/options/flow_ratios.rs`
- existing `crates/ares-core/src/options/support_interface_not_for_body.rs` parser/accessor reused unchanged
- `crates/ares-core/src/extrusions/tests/support_filament_extrusion.rs`
- `crates/ares-core/src/pipeline/tests/support_interface_not_for_body.rs`
- `docs/roadmap.md`

Ares currently rejects different-extruder slicing at the public slicing boundary and does not emit Orca tool-ordering `T` changes. Existing support and support-interface filament selectors already affect role extrusion width and E deltas through `ExtrusionOptions`. This slice must therefore consume `support_interface_not_for_body` only by choosing support-body extrusion hardware when Ares resolves `support_filament` and `support_interface_filament`, while keeping support geometry-width resolution on the original support selector.

## Behavior

When `support_filament` is fixed to a positive Orca selector, keep that explicit support-body selector unchanged.

When `support_filament` is omitted or `0`, `support_interface_filament` is fixed to a positive Orca selector, `support_interface_not_for_body` is true, and Ares has another role-hardware selector available, use the first available Ares selector that is not the fixed interface selector for `PrintPathRole::SupportMaterial` extrusion output. This is Ares' temporary compatibility-shell stand-in for Orca's min-flush candidate ranking until flush matrices and soluble-filament filtering are ported.

When `support_interface_not_for_body` is false, keep the existing support-body fallback selector, which can share the fixed interface selector when the first role-hardware selector is the interface selector.

When the fixed interface selector is not Ares' current support-body fallback selector, the resolved support-body hardware remains unchanged. When no other selector exists, the resolved support-body hardware remains unchanged.

The fixed `support_interface_filament` selector continues to own `PrintPathRole::SupportMaterialInterface`; this slice must not change interface extrusion hardware, support geometry-width resolution, support path roles, support spacing, support interface layer classification, tool ordering, flushing, or emitted tool-change commands.

Because Ares permits nozzle and filament hardware vectors with independent fallback, "available Ares selector" means a zero-based selector in `0..max(nozzle_diameter.len(), filament_diameter.len())`.

## Included

- Distinguish raw Orca support filament selectors (`0` means "don't care"; positive values are fixed one-based selectors) before converting them to Ares' zero-based role-hardware selectors.
- Preserve explicit positive `support_filament` precedence.
- Use `support_interface_not_for_body` to avoid a fixed first interface selector for support-body extrusion output when support body is "don't care" and another selector exists.
- Keep false, omitted, no-interface, non-first-interface, and single-selector cases behavior-compatible.
- Prove support-body E changes while support-body geometry width stays unchanged and support-interface width/E remains owned by `support_interface_filament`.
- Prove the selector change reaches emitted G-code extrusion values for current support-body paths.
- Keep touched Rust files at or below 400 LOC.
- Update `docs/roadmap.md` to mark this filament-selector behavior consumed and keep full Orca tool-ordering parity deferred.

## Deferred

- Orca `ToolOrdering` parity and `T` command emission.
- Flush-volume matrix selection, soluble-filament filtering, and min-flush candidate ranking.
- Flush-into-support override behavior from `ToolOrdering.cpp:1688-1704`.
- Layer/object-aware detection of `has_support`, `has_interface`, and `layer_tools.has_object`.
- Support invalidation graph parity.
- Full support material generation, support-layer storage, support projection, support interface/base classification beyond existing Ares roles, and support geometry changes.
- UI, CLI, WASM option-surface changes, registry metadata changes, preset migration behavior, and Orca binary E2E parity.

## Acceptance criteria

- With `support_filament: 0`, `support_interface_filament: 1`, `support_interface_not_for_body: true`, and two available hardware selectors, `SupportMaterial` extrusion output uses selector `1` while `SupportMaterial` geometry width and `SupportMaterialInterface` remain selector `0`.
- The same configuration with `support_interface_not_for_body: false` keeps `SupportMaterial` geometry and extrusion output on selector `0`.
- A positive `support_filament` is never overridden by `support_interface_not_for_body`.
- A fixed non-first `support_interface_filament` leaves the current support-body fallback selector unchanged.
- Missing or `0` `support_interface_filament` leaves current support-body behavior unchanged.
- Single-selector hardware leaves current support-body behavior unchanged.
- Invalid `support_interface_not_for_body` values still fail before model loading and now also fail when building `ExtrusionOptions`.
- Current geometry, print paths, diagnostics, and G-code stay unchanged for existing valid true/false configurations that do not meet the fixed-interface selector condition.
- A focused G-code test proves support-body extrusion output changes when the fixed first interface selector is avoided.
- Fresh verification includes targeted support-interface-not-for-body and support-filament tests, relevant role-filament/pipeline regressions, `cargo fmt --check`, `git diff --check`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, and touched Rust file LOC checks.
