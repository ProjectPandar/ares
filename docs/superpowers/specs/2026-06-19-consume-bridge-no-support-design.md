# Consume `bridge_no_support` in Bottom Bridge Role Selection

## Problem

Recent Ares milestones have parsed many Orca options without making every option affect slicing behavior. `bridge_no_support` is already parsed into `BridgeOptions`, but no slicing stage consumes it. This slice must make the option change generated paths and G-code roles, not add another option registry milestone.

## Upstream Boundary

This is a source-cited Rust rewrite slice of OrcaSlicer `libslic3r`:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:928` declares `PrintObjectConfig::bridge_no_support`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1847-1852` registers `bridge_no_support` as "Don't support bridges", default `false`.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1515-1529` uses `bridge_no_support` while selecting bottom surface classification in `PrintObject::detect_surfaces_type()`: normal auto support with `bridge_no_support == false` can treat bottom surfaces as fully supported `stBottom`; otherwise the unsupported bottom classification remains `stBottomBridge`.
- `OrcaSlicer/src/libslic3r/Support/SupportMaterial.cpp:1395` and `1508` also use `bridge_no_support` while filtering support contacts. Full support contact generation is not present in Ares yet and is explicitly deferred.

## Ares Destination Boundary

Target files:

- `crates/ares-core/src/bridges.rs`: keep existing parsing/accessor behavior.
- `crates/ares-core/src/print_paths.rs`: consume `BridgeOptions::bridge_no_support()` during solid bottom path role selection.
- `crates/ares-core/src/pipeline.rs` and `crates/ares-core/src/pipeline/test_support.rs`: pass the parsed option into print-path generation.
- Focused tests under `crates/ares-core/src/print_paths/tests/` and `crates/ares-core/src/pipeline/tests/`.

## Design

Ares currently represents bottom solid surfaces by mapping dense infill paths to `PrintPathRole::BottomSurface` in `generate_print_paths()`. It has no support generator and no per-surface support-contact model, but it does have layer contours and rectangular unsupported-layer tests. This slice ports the part of Orca's `detect_surfaces_type()` behavior that Ares can represent now:

- Default `bridge_no_support = false` preserves existing `BottomSurface` output.
- When `bridge_no_support = true`, a non-first bottom-shell solid infill path on a fully unsupported layer maps to `PrintPathRole::Bridge`.
- A "fully unsupported layer" means the current layer has contours, a previous layer exists, and none of the current layer contour bounds has positive-area overlap with any previous layer contour bounds. Mixed supported/unsupported layers remain `BottomSurface` until Ares tracks infill-path ownership by source contour.
- First-layer bottom solid paths remain `BottomSurface`; there is no lower printable/supporting layer to classify as a bridge.
- Supported layers, including repeated rectangular layers with positive overlap, remain `BottomSurface` even when `bridge_no_support = true`.

This makes the option affect the concrete output pipeline: bridge paths emit `;PRINT_PATH:bridge`, use bridge speed, bridge flow, bridge acceleration/jerk/fan handling already wired for `PrintPathRole::Bridge`, and no longer emit as `bottom_surface`.

## Included Behavior

- Parse and consume existing `bridge_no_support` from `SliceOptions::bridge_options()`.
- Extend `generate_print_paths()` inputs enough to compare current and previous layer contour bounds.
- Add unit tests proving role selection for unsupported, supported, and first-layer bottom solid paths.
- Add a pipeline/G-code regression proving `bridge_no_support = true` changes an unsupported second-layer dense infill path from `bottom_surface` to `bridge`.

## Deferred Behavior

- No new option keys, registry metadata, generated option modules, or new crates.
- No full support generation.
- No `SupportMaterial::remove_bridges_from_contacts()` rewrite.
- No tree-support, normal-auto support-type, soluble support, support-interface layer, max-bridge-length, or support contact area behavior.
- No mixed-contour per-path support classification until Ares infill paths retain their source contour/surface identity.

## Acceptance Criteria

- `bridge_no_support` has at least one non-test use outside option parsing.
- Existing default output remains unchanged for bottom solid surface tests.
- With `bridge_no_support = true`, a non-first, fully unsupported bottom-shell dense infill path emits as `PrintPathRole::Bridge` and G-code contains `;PRINT_PATH:bridge:` for that layer.
- With `bridge_no_support = false`, the same path emits as `PrintPathRole::BottomSurface`.
- With `bridge_no_support = true`, supported bottom-shell dense infill paths still emit as `BottomSurface`.
- First-layer dense bottom paths still emit as `BottomSurface`.
- Rust source files remain at or below 400 LOC.
- Verification includes targeted tests, `cargo fmt --check`, `cargo test -p ares-core --lib`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and the LOC guard.
