# ARD: BuildVolume circle containment numeric boundary

The immutable source owner is `libslic3r::BuildVolume`, called by
`ModelInstance::calc_print_volume_state` and CLI:5717-5723. See
`../superpowers/specs/2026-10-08-stock-circle-print-volume.md` for the bounded
rewrite and exits.

- Circle fitting is `Geometry/Circle.cpp::circle_ransac`, not the existing Ares
  centroid approximation. Sampling uses its own default mt19937 instance.
  Reuse the core's existing portable generator; do not consume slicing RNG.
- `BuildVolume.cpp:394` adds SceneEpsilon to the **scaled** circle radius and
  only then unscales to f32. `Point.hpp:688-705` casts both the scaled value
  and SCALING_FACTOR to f32 **before** multiplying (not a double product cast
  afterward). Preserve this upstream unit order, f32 transformed
  vertices, and f32 containment arithmetic; do not expand by 1e-4 mm or add
  tolerances to make the synthetic test pass.
- Rust destination: `project_slice/state/print_volume.rs`, with circle fitting
  in a small sibling file if needed to keep module size below 400 lines.
  No UI, filesystem, host RNG, terminal, new pipeline, or adapter logic.
- The stock-circle Model rejection is the existing exact -50 message, not the
  processed-G-code -102 family. No input-specific exception or new guard.
- Sparse synthetic input-read crashes are separately classified, not parity
  oracles; retain their artifacts without repairing their constructor here.
- Keep timelapse changes isolated from this slice.

Debugger witness: fitted radius
54999990.378506288 in fixed coordinates, near-zero center, mesh vertex
[4,0,-5] transformed by translation [51,0,5] to [55,0,0], Circle type. The
Native float effective radius is below 55 mm; the current mm epsilon changes
that eligibility decision. This is not a plate-ownership failure.
