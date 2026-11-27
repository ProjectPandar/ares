# Deterministic binary64 cosine for belt machine-frame transforms

## Source boundary

The pinned OrcaSlicer 2.5.0 source (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) has
three belt-coordinate stages whose floating-point results reach emitted coordinates:

- `src/libslic3r/BeltTransform.cpp:20-36` converts
  `belt_slice_rotation_angle` from degrees and builds the mesh-side rotation. The
  `Eigen::AngleAxisd` matrix is the upstream source boundary for the explicit sine
  and cosine values used by the Ares matrix builder.
- `src/libslic3r/BeltTransform.cpp:39-45` installs that rotation as the forward
  belt transform. Its floor calculation at `:90-112` explicitly evaluates
  `std::sin(angle_rad)` and `std::cos(angle_rad)` at `:96`, then uses their ratio
  for the belt-floor shear at `:98-104`.
- `src/libslic3r/GCode/BeltBackTransform.cpp:8-23` initializes the inverse of the
  forward transform, and `:26-31` applies it to each point. The per-point ordering
  is documented by `src/libslic3r/GCode/BeltKinematics.hpp:14-27` and implemented
  by `src/libslic3r/GCode/BeltKinematics.cpp:17-22`:
  `MachineFrameTransform(axis_remap(BeltBackTransform(logical)))`.
- `src/libslic3r/GCode/MachineFrameTransform.cpp:28-39` selects either the
  decoupled frame angle or the slice angle, converts it with
  `Geometry::deg2rad` (`src/libslic3r/Geometry.hpp:312-316`), and evaluates
  `std::sin` at `:35` and `std::cos` at `:38`. Lines `:50-65` build the shear and
  scale matrices from those values; `:75-86` apply the transform and its inverse.
  The angle-selection and post-remap semantics are also stated in
  `src/libslic3r/GCode/MachineFrameTransform.hpp:10-20`.

The current Ares rewrite has the matching boundaries:

| Ares site | Upstream boundary and role | Required routing |
| --- | --- | --- |
| `crates/ares-core/src/project_slice/belt.rs:92-100` | Belt admission rejects a non-tilted or effectively zero sine angle, corresponding to the belt tilt preconditions in `MachineFrameTransform.cpp:24-39`. | Replace `angle.to_radians().sin()` with `sin_f64(angle.to_radians())`. |
| `crates/ares-core/src/project_slice/belt.rs:117-145` | `BeltRotation::from_printer` builds the pre-slice rotation corresponding to `BeltTransform.cpp:20-45`; the stored values feed the rotated mesh, floor shear, and belt slice geometry at `:203-239`. | Compute the pair as `sin_f64(rad)` and `cos_f64(rad)` from one `deg2rad` result. |
| `crates/ares-core/src/project_slice/belt/back_transform.rs:33-63` | `build_forward_transform` expands the `BeltTransform.cpp:20-45` rotation matrix for the back-transform source used by `BeltBackTransform`. | Compute both values with `sin_f64` and `cos_f64`. |
| `crates/ares-core/src/project_slice/belt/kinematics.rs:22-106` | `MachineFrameTransform::from_printer` mirrors `MachineFrameTransform.cpp:11-73`, including the decoupled angle, cotangent, inverse sine, and inverse matrix. | Compute both values with `sin_f64` and `cos_f64`. |

The resulting values flow through `crates/ares-core/src/project_slice/belt/kinematics.rs:121-155`
and `:182-189` to `crates/ares-core/src/project_slice/gcode_emit/motion/belt_writer.rs:20-26`.
No belt trigonometric call outside the four rows above is present in the current Ares
belt modules. The ratio and matrix arithmetic remains unchanged; only the binary64
transcendental entry points change.

## Observed target divergence

A standalone comparison at an angle that can be represented directly by a synthetic
belt setting demonstrates the target-libm problem before this slice is implemented.
Use the exact binary64 computation already used by the source boundary:

```
degrees  = 4.258
angle    = PI * degrees / 180.0
angle    = 0x1.30661970853a6p-4
```

At that exact angle, the correctly rounded CORE-MATH result and the pinned native
`cos` result are both `0x3fefe9637fe09aa4`, while the wasm32 `f64::cos` libm result
is `0x3fefe9637fe09aa3`. The one-ULP disagreement is reproducible without a
project-specific fixture and is an exact angle used by the proposed synthetic belt
fixture (set both the slice angle and frame angle to `4.258` degrees, with an X or
Y belt axis). This is sufficient to require a correctly rounded binary64 cosine
kernel before comparing emitted coordinates. The corresponding sine result must
also be taken from the existing correctly rounded `sin_f64` implementation.

The CORE-MATH source used for the comparison is the MIT-licensed binary64 cosine
routine at revision `3270006242ceef2885767918533ac5446807f2cf`,
`src/binary64/cos/cos.c`. Its provenance and MIT notice are in the source header
(`cos.c:1-25`) and the project license (`LICENSE:1-20`). The entry point dispatches
tiny inputs, the moderate range, infinities/NaNs, and the large range at
`cos.c:943-970`; the moderate correctly rounded path is `:827-869`, the large
Payne-Hanek path is `:872-939`, and the accurate path is `:764-825`.

## Rust destination boundary

- Add `cos_f64` beside the existing `sin_f64` under
  `crates/ares-core/src/scalar_math/`, with a crate-internal export from
  `crates/ares-core/src/scalar_math.rs:1-9`.
- Port the CORE-MATH binary64 cosine dispatch and its moderate, accurate, and
  large-argument paths to safe Rust while preserving binary64 evaluation order,
  special-value behavior, signed-zero handling where applicable, and correctly
  rounded round-to-nearest-even results. Retain the CORE-MATH copyright and MIT
  notice in every file containing copied or mechanically translated substantial
  portions.
- Share the existing CORE-MATH-derived sine tables and integer helpers rather than
  introducing duplicate table bytes. The cosine source explicitly identifies the
  shared `pih`, `pil`, `_T`, `PS`, `PC`, `U1`, `U2`, `S1u`, `S2u`, and `C1u` tables in
  `cos.c:27-34`; the existing Ares sine provenance is recorded in
  `docs/architecture/scalar-math-binary64-sin.md:25-48`.
- Route the four belt sites in the table above through `sin_f64` and `cos_f64`.
  Do not add a public API, JavaScript math import, target-specific branch, or
  approximation fallback. A separate `sincos_f64` API is not required by this
  boundary; paired call sites may invoke the two scalar kernels in the established
  source order.

## Included behavior

- Correctly rounded binary64 cosine for normal and subnormal inputs, tiny inputs,
  moderate and large arguments, signed zero, infinities, and NaNs, following the
  CORE-MATH binary64 contract.
- Correctly rounded binary64 sine and cosine at belt admission, pre-slice/back
  rotation, floor shear inputs, and decoupled machine-frame construction.
- Existing belt axis selection, angle validation, matrix layouts, transform order,
  remapping, rounding, and G-code formatting remain unchanged.
- A synthetic belt project using the `4.258` degree angle above, with Native/API/CLI
  comparison and a browser wasm run. The final comparison must cover both native
  reference modes and prove byte-identical emitted G-code after the scalar routing.

## Deferred behavior

This slice does not route non-belt trigonometry. The current non-belt call-site
inventory is listed here so that later scalar-math work does not silently expand
this boundary:

- Arachne and arrangement geometry: `crates/ares-core/src/arachne/**`,
  `crates/ares-core/src/arrangement/**`.
- Brims and fill generation: `crates/ares-core/src/brims/**`,
  `crates/ares-core/src/fill/**`.
- Geometry helpers: `crates/ares-core/src/geometry/arc_welder.rs`,
  `geometry/clipper2/core/ellipse_point64.rs`,
  `geometry/clipper2/offset/{do_group_offset,do_square}.rs`,
  `geometry/mutable_polygon/duplicates.rs`, `geometry/support_grid.rs`, and
  `geometry/voronoi/rotation.rs`.
- Flush-volume and plater code: `crates/ares-core/src/options/flush_volume/**`,
  `crates/ares-core/src/plater/**`.
- Print validation: `crates/ares-core/src/print/validate/{compacted_tower,estimated_tower}.rs`.
- Non-belt project-slice geometry and transforms:
  `project_slice/centering/**`, `contour_z/path.rs`,
  `gcode_emit/motion/**`, `gcode_emit/skirt/**`, `gcode_emit/wipe_tower/**`,
  `group_fills/**`, `hole_to_polyhole/**`, `perimeters/classic/counterbore/**`,
  `prepare_infill/bridge_over_infill/anchored_polygon.rs`,
  `raft/fills/support_base.rs`, `region_slices/**`, `seam_placement/**`, and
  `support_material/**`.
- The existing binary32 `scalar_math/sincosf.rs` callers and any non-belt binary64
  `sin`, `cos`, or `sin_cos` sites remain unchanged.
- `sincos_f64`, tangent, inverse trigonometric functions, changes to angle parsing,
  and changes to the machine-frame formulas are outside this source boundary.

## Verification plan

1. Compare `cos_f64` bit patterns against the pinned CORE-MATH `cr_cos` source on
   its hard-to-round vectors, special values, and a stratified binary64 sample.
2. Run the synthetic `4.258` degree belt project through Native, Ares CLI, and the
   wasm entry point. Compare emitted coordinates and whole-file G-code after the
   generator header, for both native reference modes.
3. Exercise X and Y axes, coupled and decoupled frame angles, and the existing belt
   frame/back-transform parity cases. Confirm the transform order and inverse remain
   unchanged.
4. Run the required non-parity checks, wasm32 compilation, browser comparison, and
   the required core-count invariance matrix. No implementation or test changes are
   part of this specification commit.
