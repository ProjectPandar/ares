# Consume Single Loop Draft Shield Design

## Goal

Consume Orca `single_loop_draft_shield` as executable Ares skirt/draft-shield behavior instead of leaving it as registry/metadata-only option coverage.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1557` declares the `PrintConfig` `single_loop_draft_shield` option tuple.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5567-5571` defines `single_loop_draft_shield` as `coBool`, labels it "Single loop after first layer", and defaults it to `false`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:4334-4390` emits all assigned skirt loops on the first layer, but when `single_loop_draft_shield == true` and the layer is not the first layer, starts at `loops.second - 1` and breaks after one loop.
- `OrcaSlicer/src/libslic3r/GCode.cpp:5112-5236` routes combined and per-object skirt emission through `generate_skirt(...)`.

## Current Ares State

- `SliceOptions::skirt_options()` does not parse `single_loop_draft_shield`.
- `SkirtOptions` carries loop count, distance, height, speed, draft shield, skirt type, and minimum skirt length, but not the single-loop draft-shield flag.
- `generate_skirts()` currently emits the configured skirt loop count on every generated draft-shield layer.
- `DraftShield::Enabled` already makes skirts generate beyond the configured `skirt_height`.
- `SkirtType::PerObject` is already rejected because Ares has no per-object contour ownership or object-specific skirt artifacts.

## Design

Add `single_loop_draft_shield` to `SkirtOptions`, defaulting to `false`. Parse `single_loop_draft_shield` at the `SliceOptions::skirt_options()` boundary as a boolean with Orca default `false`.

For combined skirts, preserve existing behavior when `single_loop_draft_shield == false`: every generated layer emits the configured loop count, subject to `skirt_height`, `draft_shield`, and `min_skirt_length` behavior.

When `single_loop_draft_shield == true`, keep the first non-empty Ares skirt layer unchanged, including any `min_skirt_length` loop extension. In Ares terms, that is the first layer in `generate_skirts()` that both passes `generates_on_layer()` and has contour bounds that emit at least one skirt path. On later generated layers, emit only one skirt loop. This mirrors the upstream G-code emission effect at Ares' current artifact boundary: later draft-shield layers contain only one real skirt path, and that single path flows through print paths, moves, extrusion moves, speed moves, and G-code.

The single loop for later layers will be the outermost configured loop available in Ares' rectangular combined-skirt representation. This corresponds to Orca's `loops.second - 1` behavior, while acknowledging that Ares has rectangular loops rather than Orca's rounded offsets.

For `SkirtType::PerObject`, keep the existing unsupported-input error. Per-object single-loop draft-shield emission remains deferred with per-object skirt generation.

## Deferred Upstream Behavior

- Per-object single-loop draft-shield behavior remains deferred until per-object skirt artifacts exist.
- Exact Orca loop assignment by extruder remains deferred; Ares currently has no per-extruder skirt loop assignment machinery.
- Exact Orca rounded offset geometry remains deferred; Ares continues using its rectangular combined-skirt bounds.
- `skirt_start_angle` remains a separate option-consumption slice.

## Acceptance Criteria

- `SliceOptions::skirt_options()` returns `single_loop_draft_shield() == false` when the option is absent.
- `single_loop_draft_shield: true` and `false` parse to the matching `SkirtOptions` value.
- Non-boolean `single_loop_draft_shield` values return `SliceError::InvalidInput` at the skirt option boundary.
- With `draft_shield: "enabled"`, `skirt_loops: 2`, and `single_loop_draft_shield == false`, existing multi-loop draft-shield behavior is preserved on later generated layers.
- With `draft_shield: "enabled"`, `skirt_loops: 2`, and `single_loop_draft_shield == true`, the first generated skirt layer still emits two loops, while each later generated layer emits one loop.
- With `draft_shield: "enabled"`, `skirt_loops: 2`, and `single_loop_draft_shield == true`, the later generated layer's single loop uses the outer loop coordinates, not the inner loop coordinates.
- With `single_loop_draft_shield == true` and `min_skirt_length` requiring extra first-layer loops, the first non-empty generated skirt layer is still extended to satisfy `min_skirt_length`, and each later generated layer emits exactly one configured outer loop rather than a min-length-extended loop.
- If earlier layers are empty or outside the configured skirt height, the first non-empty layer that actually emits skirt paths receives first-layer behavior; subsequent emitted draft-shield layers receive single-loop behavior.
- The later single loop appears as real skirt artifacts, print paths, moves, extrusion moves, speed moves, and G-code path-following commands rather than metadata-only changes.
- `skirt_type: "perobject"` still returns the existing unsupported per-object error even when `single_loop_draft_shield` is set.
- Rust source files touched by this slice stay below the repository 400 LOC split threshold.

## Files

- Modify `crates/ares-core/src/skirts/mod.rs` for the `SkirtOptions` field, accessor/builder, and non-first-layer loop count selection.
- Add focused tests in `crates/ares-core/src/skirts/tests.rs` if needed for domain-level loop selection.
- Modify `crates/ares-core/src/options.rs` to parse `single_loop_draft_shield` and pass it into `SkirtOptions`.
- Add `crates/ares-core/src/options/tests/single_loop_draft_shield.rs` for focused option-boundary tests.
- Modify `crates/ares-core/src/options/tests.rs` only by adding `single_loop_draft_shield` to the existing focused module macro while preserving the 400 LOC limit.
- Add `crates/ares-core/src/tests/single_loop_draft_shield_gcode.rs` for slice/G-code regressions.
- Modify `crates/ares-core/src/tests/mod.rs` only to register the focused G-code test module.

## Docs Impact

No architecture or roadmap update is required for this narrow runtime consumption slice. This source-cited design document, the implementation plan, and regression tests document the included combined-skirt behavior and the explicit deferrals.

## Verification

- `cargo test -p ares-core single_loop_draft_shield --lib`
- `cargo test -p ares-core single_loop_draft_shield_gcode --lib`
- `cargo fmt --check`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `wc -l` on touched Rust files to confirm none exceed 400 LOC
