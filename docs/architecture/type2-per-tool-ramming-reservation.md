# Type2 per-tool ramming reservation

## Upstream boundary

This slice rewrites OrcaSlicer v2.5.0
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/GCode/WipeTower2.cpp:2289-2300`,
`WipeTower2::set_toolchange`, and its predicates in
`WipeTower2.hpp:323-327`.

The Rust destination is
`project_slice::gcode_emit::wipe_tower::plan::Plan::set_toolchange`.
The existing planner is a compatibility shell around the upstream plan;
this correction replaces its global-only reservation guard with the
upstream per-tool guard. It does not introduce a new planning API.

## Required behavior

Use the existing predicate exactly once:

```text
do_ramming = (semm && enable_filament_ramming) || filament.multitool_ramming
boundary_wipe_start = do_ramming && !semm && use_gap_wall
```

Both `ramming_depth` and `first_wipe_line` must use `do_ramming`.
`first_wipe_line` additionally requires `!boundary_wipe_start`, as upstream
already does. A multi-tool printer can ram with global SEMM ramming disabled;
its active ramming band must still be reserved. Reservation and unload
execution must agree on whether that tool rams.

Only move and reuse the existing Rust predicate for the depth guard. Preserve
all arithmetic, spacing, volume, boundary-start, and row-quantization logic.
In particular, this slice does not change the existing ramming-time factor.

## Public E2E witnesses

Reuse the existing native-backed integration cases without renaming or
weakening them:

- `tiq2_footer_dispatch_whole_file`: compares the complete generated stream
  through `OrcaRunner::slice_case` and `compare_case`, including ordered G-code
  produced through the public slicing API. Its name identifies the broader
  whole-file footer-dispatch contract, not a private planner method.
- `tiq2_restores_retained_role_fan_complete_sequence`: generates bytes through
  the public `slice_project` adapter and compares every fan command with its
  layer and adjacent role context against Native. Its name identifies the
  complete observable fan-restoration contract.

Their shared TiQ2 corner input has global ramming disabled and per-filament
multi-tool ramming enabled. The Native initial plan reserves 1.5 mm for each
actual ramming band; the old Rust guard reserves zero. The first actual
change at Z=0.8 therefore reserves 6.75 mm in Native versus 5.25 mm in Rust.
These future-layer reservations affect the tower dimensions and first skirt
because tower planning precedes skirt generation.

Confirm both existing public E2E cases are red before changing the planner.
Then rerun them unchanged and retain full streams and comparison artifacts.
A shared input does not replace either observable contract.

## Validation and exclusions

Compare a source-cited wipe-tower E2E subset at the unmodified base and fixed
tip in both retained Native environment profiles, recording the exact filter,
test identities, and outcomes. No previously passing selected case may fail.

Check tower bounds, skirt hull, and per-layer cooling time again. If cooling
time and the fan baseline converge, record them as downstream consequences.
If another difference remains, report its first location before proposing
another product change. Do not weaken the whole-stream or fan assertions.

Footer dispatch, retained-role-fan restoration, cooling algorithms, precision,
comparators, fixture settings, producer binaries, and adjacent ramming-time
selection are outside this slice.
