# Belt Frame-Tilt Decouple (`BeltBackTransform`, `MachineFrameTransform`, `BeltKinematics`, `BeltGCode`)

Source: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).
All upstream citations below are relative to `src/libslic3r`.

## Upstream Boundary

Belt G-code coordinate conversion and `belt_frame_tilt_decouple` live across five upstream units:

1. **`BeltTransform.{hpp,cpp}`** (`BeltTransform.cpp:20–46`):
   - `BeltTransformPipeline::build_rotation_matrix` builds the 3x3 isometric pre-slice rotation matrix from `belt_slice_rotation` (`none`/`x`/`y`/`z`) and `belt_slice_rotation_angle` (`deg2rad(angle) = PI * angle / 180.0`), returning identity when axis is `None` or `abs(angle) <= EPSILON` (`1e-4`).
   - `BeltTransformPipeline::build_forward_transform` wraps that rotation in `Transform3d` (without the per-object mesh Z-shift from `BeltSliceStrategy.cpp:12–80`).
   - Neither `BeltTransformPipeline::floor_shear` nor `BeltSliceStrategy::apply_preslice_transforms` reads `belt_frame_tilt_decouple` or `belt_frame_tilt_angle`; slicing geometry, layer heights, and floor bands always follow `belt_slice_rotation` and `belt_slice_rotation_angle`.

2. **`GCode/BeltBackTransform.{hpp,cpp}`** (`BeltBackTransform.cpp:8–32`):
   - `BeltBackTransform::init_from_config` activates only when `belt_printer` is enabled and `build_forward_transform(config)` is not approximately identity (`Transform3d::isApprox(Transform3d::Identity())`, Eigen `double` precision `1e-12`). It stores `forward.inverse()`.
   - `BeltBackTransform::apply` returns `m_inverse * pos` when active and `pos` unchanged otherwise.

3. **`GCode/MachineFrameTransform.{hpp,cpp}`** (`MachineFrameTransform.cpp:11–80`):
   - `MachineFrameTransform::init_from_config` runs only when `belt_printer` is enabled and `belt_slice_rotation` is `X` or `Y` (`None` and `Z` have no physical gantry tilt).
   - Selects `angle_deg = belt_frame_tilt_decouple ? belt_frame_tilt_angle : belt_slice_rotation_angle`.
   - Returns inactive (identity) when `abs(angle_deg) <= EPSILON` or `abs(sin(angle_rad)) <= EPSILON`.
   - Computes `cot_a = cos(angle_rad) / sin(angle_rad)` and `inv_sin = 1.0 / abs(sin(angle_rad))`, then builds `combined = scale * shear`:
     - `X`: `(x, y, z) -> (x, inv_sin * y, cot_a * y + z)`
     - `Y`: `(x, y, z) -> (inv_sin * x, y, -cot_a * x + z)`
   - Returns inactive when `combined.isApprox(Transform3d::Identity())`; otherwise stores the linear matrix and applies it per point in `apply`.

4. **`GCode/MachineKinematics.{hpp,cpp}` & `GCode/BeltKinematics.{hpp,cpp}`** (`MachineKinematics.cpp:7–23`, `BeltKinematics.cpp:10–27`):
   - `BeltKinematics` composes the three stages in exact forward order:
     `machine = MachineFrameTransform(axis_remap(BeltBackTransform(logical)))`.
   - `CartesianKinematics::apply_axis_remap` maps each output axis via `gcode_remap_{x,y,z}` (`pos_*`, `neg_*`, or `rev_*` against `build_vol_max = (bed_max_x, bed_max_y, printable_height)`).
   - Structural writer contract: `must_emit_all_axes() == true`, `suppress_lift_at_unknown_position() == true`, `supports_arc_moves() == false`.

5. **`BeltGCode.{hpp,cpp}`, `Print.cpp`, `GCode.cpp`, `PrintConfig.{hpp,cpp}`**:
   - `PrintConfig.cpp:7748–7769` defines `belt_frame_tilt_decouple` (default `false`) and `belt_frame_tilt_angle` (default `45.0`, range `[-89.9, 89.9]`).
   - `Print.cpp:279` invalidates only G-code export when `belt_frame_tilt_decouple` or `belt_frame_tilt_angle` changes; `Print.cpp:3998–4003` selects `BeltGCode` when `belt_printer` is enabled.
   - `BeltGCode::init_belt_writer` (`BeltGCode.cpp:13–19`, called from `GCode.cpp:3455–3474`) installs `BeltKinematics` and forces normal lift (`m_writer.set_force_normal_lift(true)`).
   - `BeltGCode::write_belt_header` (`BeltGCode.cpp:21–32`, called from `GCode.cpp:4340`) emits `; belt_slice_rotation = ...`, `; belt_slice_rotation_angle = %.1f`, `; belt_frame_tilt_decouple = %d`, and `; belt_frame_tilt_angle = %.1f`.
   - `BeltGCode::on_set_origin` (`BeltGCode.cpp:34–48`, called from `GCode.cpp:4665,6051,6280,7672,7684,7758,7900`) projects the 2D origin through `BeltTransformPipeline::build_forward_transform(m_config)` (unaffected by `belt_frame_tilt_decouple`).

## Rust Destination Boundary

- `crates/ares-core/src/project_slice/belt/back_transform.rs`:
  - Ports `BeltTransformPipeline::build_forward_transform` and `BeltBackTransform` (`init_from_config` / `apply`) plus Eigen-compatible `is_approx_identity`.
- `crates/ares-core/src/project_slice/belt/kinematics.rs`:
  - Ports `MachineFrameTransform`, `CartesianKinematics::apply_axis_remap`, `BeltKinematics` (`from_printer`, `to_machine`), and `BeltGCode::on_set_origin` origin projection.
- `crates/ares-core/src/project_slice/belt.rs`:
  - Declares submodules `back_transform` and `kinematics`.
  - Lifts only the `belt_printer: belt_frame_tilt_decouple` admission rejection; retains all other belt admission checks (`instances`, `volumes`, `materials`, `brim`, `support`, `belt_purge`, `skirt`, `spiral_mode`, `contoured_toolpaths`, `belt_slice_rotation`, `gcode_remap`) unchanged.
  - Replaces the temporary `BeltRotation::inverse` and `BeltRotation::machine_frame` scaffold methods with the upstream-aligned `BeltBackTransform` and `MachineFrameTransform` types, keeping `BeltRotation` strictly for pre-slice mesh/floor planning in `BeltSlice::new`.
- `crates/ares-core/src/project_slice/gcode_emit/motion/belt_writer.rs`:
  - Delegates `BeltWriter::origin` and `BeltWriter::apply` directly to `belt::kinematics::BeltKinematics`, removing the inline rotation-coupled machine-frame calculation.
  - Header output in `crates/ares-core/src/project_slice/gcode_emit/header.rs` already emits all four `BeltGCode::write_belt_header` lines verbatim and requires no modification.
- `crates/ares-cli/tests/orca_parity/belt_frame_tilt_decouple.rs` (registered in `crates/ares-cli/tests/orca_parity/object_emission_cases.rs`):
  - Public CLI whole-G-code comparison cases against OrcaSlicer v2.5.0.

## Included vs Deferred Behavior

### Included
- `belt_frame_tilt_decouple = true` and `false` across X-axis and Y-axis belt rotations.
- Positive, negative, fractional, right-angle slice, and zero/epsilon (`abs <= 1e-4`) decoupled machine-frame tilt angles (returning identity `MachineFrameTransform` when inactive, matching `MachineFrameTransform.cpp:31–37,66–67`).
- Exact upstream `inv_sin = 1.0 / abs(sin_a)` and `scale * shear` matrix application order in `MachineFrameTransform::apply`.
- Translated-instance origin projection staying coupled to `build_forward_transform` while emitted machine coordinates follow decoupled `BeltKinematics::to_machine`.
- Existing belt writer contract (`must_emit_all_axes`, normal lift, arc disable, and header comments).

### Deferred
- `MachineFrameTransform::apply_inverse` inside `GCodeProcessor` printable-area checks (`GCodeProcessor.cpp:2958–2975,3063`).
- `BeltTransformPipeline::build_gcode_back_transform` (`BeltTransform.cpp:48–84`) and `GCodeProcessor` preview/mass-extent `m_belt_linear` (`GCodeProcessor.cpp:3262–3325,7672–7678`).
- `BeltKinematics` `world_coordinates` mode for PA calibration (`BeltKinematics.hpp:23–27`, `calib.cpp:891–904`).
- Belt brim, support, purge tower, multi-instance, multi-material, and `belt_slice_rotation` values `None` or `Z`.

## Verification

- Compare whole generated G-code via the public CLI against official and seeded OrcaSlicer v2.5.0 (generator line normalized only) for:
  - X-axis `45°` slice with `30°` decoupled frame vs `belt_frame_tilt_decouple = 0` (`belt_frame_tilt_angle = 30` ignored in coupled mode);
  - X-axis `30°` slice with `45°` decoupled frame;
  - Y-axis `45°` slice with `30°` decoupled frame vs coupled baseline, and Y-axis `30°` slice with `60°` decoupled frame;
  - zero and sub-epsilon (`0.00005°`) decoupled frame angles yielding identity machine-frame transform;
  - negative decoupled frame angle (`-30°`) and negative slicing rotation (`-45°` slice with `30°` frame);
  - right-angle slicing (`90°`) with active `30°` decoupled frame, fractional angle (`37.25°`), and translated instance (`Y + 20 mm`).
- Run workspace formatting, all-target Clippy with `-D warnings`, `wasm32-unknown-unknown` check for `ares-core` and `ares-wasm`, and pre-push hooks.
