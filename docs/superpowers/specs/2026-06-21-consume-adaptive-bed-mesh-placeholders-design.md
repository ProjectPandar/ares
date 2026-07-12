# Consume Adaptive Bed Mesh Placeholders Design

## Goal

Consume the existing OrcaSlicer adaptive bed mesh options into concrete Ares `machine_start_gcode` placeholder output. This slice must make `bed_mesh_min`, `bed_mesh_max`, `bed_mesh_probe_distance`, and `adaptive_bed_mesh_margin` affect generated startup G-code, not add more option metadata.

## Upstream Source Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1639-1644` declares the adaptive bed mesh `PrintConfig` tuple lines: `bed_mesh_min`, `bed_mesh_max`, `bed_mesh_probe_distance`, and `adaptive_bed_mesh_margin`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2162-2200` defines the same option defaults and bounds: `bed_mesh_min = (-99999, -99999)`, `bed_mesh_max = (99999, 99999)`, `bed_mesh_probe_distance = (50, 50)`, and `adaptive_bed_mesh_margin = 0`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:2871-2963` computes print-bed and first-layer print bounds, clamps the first-layer bounds by the configured bed mesh min/max after applying `adaptive_bed_mesh_margin`, then sets `adaptive_bed_mesh_min`, `adaptive_bed_mesh_max`, `bed_mesh_probe_count`, and `bed_mesh_algo` placeholders before processing `machine_start_gcode`.

## Current Ares Boundary

- The former source-line-only bed-mesh tuple modules were removed by the Option pinning cleanup; runtime behavior derives directly from the cited upstream boundary.
- `crates/ares-core/src/gcode_placeholders.rs` currently replaces only reserved file-start placeholders, layer placeholders, finish placeholders, filament placeholders, and auxiliary fan machine-start placeholders.
- `crates/ares-core/src/gcode.rs` has access to `SlicingPipeline::layer_print_paths()` and already renders `machine_start_gcode` before first-layer output. It is 395 LOC, so this slice must keep new logic in smaller modules and touch `gcode.rs` only for wiring.

## Ares Destination Boundary

- Add a small `crates/ares-core/src/options/adaptive_bed_mesh.rs` parser for the four existing options.
- Add a small `crates/ares-core/src/gcode_adaptive_bed_mesh.rs` runtime helper that computes Orca-style adaptive mesh placeholder values from the first layer's current Ares generated print paths.
- Extend `crates/ares-core/src/gcode_placeholders.rs` so machine start placeholder rendering can consume an optional adaptive mesh placeholder value bundle while preserving existing auxiliary fan placeholder behavior.
- Wire `crates/ares-core/src/gcode.rs` and `crates/ares-core/src/gcode_start_custom.rs` through one pre-rendered machine-start G-code string. Startup temperature/chamber suppression checks and the emitted machine-start G-code must use the same rendered text and adaptive mesh bundle.

## Included Behavior

- Parse `bed_mesh_min`, `bed_mesh_max`, and `bed_mesh_probe_distance` from Orca point forms accepted at the `SliceOptions` boundary:
  - absent values use the upstream defaults;
  - JSON arrays `[x, y]`;
  - JSON arrays with one nested pair `[[x, y]]`;
  - strings in `XxY` form.
- Parse `adaptive_bed_mesh_margin` as a finite non-negative number, defaulting to `0`.
- Compute the first-layer print bounds from all points in `layer_print_paths()[0]`, which is Ares' current closest boundary to Orca's first-layer extrusion hull because it includes object paths plus generated skirt/brim paths.
- If Ares has no first layer or the first layer has no print-path points, use the configured `bed_mesh_min` and `bed_mesh_max` directly as the adaptive mesh bounds. This avoids panics and keeps output deterministic for empty-layer edge cases until full Orca print-object handling is ported.
- Compute:
  - `adaptive_bed_mesh_min = max(bed_mesh_min, first_layer_min - adaptive_bed_mesh_margin)`
  - `adaptive_bed_mesh_max = min(bed_mesh_max, first_layer_max + adaptive_bed_mesh_margin)`
  - `bed_mesh_probe_count.x = max(3, ceil((adaptive_max.x - adaptive_min.x) / max(1, bed_mesh_probe_distance.x)) + 1)`
  - `bed_mesh_probe_count.y = max(3, ceil((adaptive_max.y - adaptive_min.y) / max(1, bed_mesh_probe_distance.y)) + 1)`
  - `bed_mesh_algo = "lagrange"` when `probe_count_x * probe_count_y <= 6`, otherwise `"bicubic"`.
- For Klipper flavor, preserve Orca's bicubic minimum of four probe points per axis when the selected algorithm is bicubic.
- Replace the following machine-start placeholders in bracket form: `[adaptive_bed_mesh_min]`, `[adaptive_bed_mesh_min_0]`, `[adaptive_bed_mesh_min_1]`, `[adaptive_bed_mesh_max]`, `[adaptive_bed_mesh_max_0]`, `[adaptive_bed_mesh_max_1]`, `[bed_mesh_probe_count]`, `[bed_mesh_probe_count_0]`, `[bed_mesh_probe_count_1]`, and `[bed_mesh_algo]`.
- Vector placeholders render comma-separated decimal or integer values, matching Ares' current comma-list formatting style.
- Unknown placeholders remain unchanged.
- Existing auxiliary fan machine-start placeholders continue to work.
- The single rendered `machine_start_gcode` string must be used both for startup command suppression checks and for final emission before `filament_start_gcode`.

## Deferred Behavior

- Full Orca placeholder parser parity, including expression syntax, indexed `name[0]` syntax, brace replacements for these new placeholders, `ConfigOptionFloats` / `ConfigOptionInts` object semantics, and placeholder failure reporting.
- `first_layer_print_convex_hull`, `first_layer_print_min`, `first_layer_print_max`, `first_layer_print_size`, `in_head_wrap_detect_zone`, `max_print_z`, and `first_layer_center_no_wipe_tower` placeholders.
- Full Orca convex hull geometry, empty-print validation, wipe tower, support generation, multi-object hull union, calibration-mode bbox substitution, object projections, plate offsets, and custom user G-code extrusion awareness.
- Any automatic bed probing G-code emission. This slice only replaces placeholders in user-supplied `machine_start_gcode`.

## Acceptance Criteria

- A focused E2E test proves `machine_start_gcode` placeholders render adaptive mesh min/max, probe counts, and algorithm before the first layer.
- A focused E2E test proves `adaptive_bed_mesh_margin` expands the first-layer bounds before bed mesh min/max clamping.
- A focused E2E test proves `bed_mesh_min` and `bed_mesh_max` clamp the rendered adaptive mesh bounds.
- A focused E2E test proves Klipper bicubic meshes use at least four probe points per axis.
- A focused unit or E2E test proves empty first-layer paths render deterministic bed mesh min/max fallback values instead of panicking.
- A focused E2E test proves startup temperature/chamber suppression sees the same adaptive-rendered `machine_start_gcode` text that is emitted.
- A focused E2E test proves malformed `bed_mesh_min`, `bed_mesh_max`, `bed_mesh_probe_distance`, or `adaptive_bed_mesh_margin` returns `SliceError::InvalidInput` with the option key in the message.
- Existing custom G-code, auxiliary fan, startup temperature, and pressure advance tests continue to pass.
- All touched Rust files remain at or below 400 LOC.

## Documentation

Update `docs/roadmap.md` with a completed runtime-slice note naming the same Orca `PrintConfig` and `GCode.cpp:2871-2963` source boundary and listing the deferred placeholder/parser behavior.

## Verification

- `cargo fmt --check`
- Focused nextest checks for adaptive bed mesh placeholder tests.
- `cargo nextest run --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC guard for touched Rust files.
