# Pre-slice prime tower fit

Status: Accepted slice specification.

## Decision and upstream boundary

Upstream pin: OrcaSlicer v2.5.0-beta `345efec64`.

A plain slice fits the plate's prime tower to the plate before anything is
sliced. The comment at the upstream site states why: "no CLI placement site runs
on a plain slice".

| Upstream | Behavior |
| --- | --- |
| `OrcaSlicer.cpp:6480-6509` | The fit loop over plates and its applicability conditions. |
| `PartPlate.cpp:4065-4155`, `estimate_wipe_tower_polygon` | The stored position kept, snapped, or re-placed. |
| `PartPlate.cpp:3982-4034`, `estimate_wipe_tower_footprint` | Tower size and resolved brim. |
| `PartPlate.cpp:4036-4063`, `wipe_tower_obstacles` | The objects a re-placed tower keeps clear of. |
| `GCode/WipeTowerEstimate.cpp:247`, `nearest_clear_wipe_tower_anchor` | The clear spot nearest the saved position. |

Ares destination:

- The fit belongs to `ares-core`, on the shared slice path, so the CLI, the
  profile entries and WASM all reach it. `ares-cli` gains no tower logic.
- `crates/ares-core/src/plater/` owns the fit, next to the existing
  `plate_list/wipe_tower.rs` port of the estimate (`Towers`). It reuses `Towers`
  and does not reimplement the clamp.
- `project_slice/state.rs::prepare_loaded_project_slice_with_validation` calls
  it first, before `select_plate`. Upstream fits at `:6480`, before the print
  volume check at `:6562`, `Print::validate` and the config export, so all of
  them see the fitted position. Every project and profile entry passes through
  this function.

## Applicability

The fit runs when all of these hold (`OrcaSlicer.cpp:6483-6498`):

- `enable_prime_tower` is true.
- The printer is not a belt printer. Ares has no `belt_printer` key, so this
  condition is not representable yet; see Deferred behavior.
- The plate is the plate being sliced.
- The plate is not by-object with more than one printable instance
  (`:6489-6491`). The bound is `> 1`, unlike the plater's shown tower, which
  requires exactly one.

After the estimate, a plate with an empty footprint keeps its stored position
(`wt_size` below `EPSILON`, `:6497-6498`). That is a plate that prints no tower:
one filament and neither smooth timelapse, wrapping detection nor a raft.

## Multiple plates

Upstream fits every plate selected by `plate_to_slice`, each through its own
index into `wipe_tower_x` and `wipe_tower_y`. No plate reads another plate's
entry, and the footprint is sized from that plate's own objects
(`use_global_objects = false`).

Ares slices one plate per call. Fitting the selected plate is therefore
equivalent for the plate's output. The index is the selected plate's position in
the project, before `select_plate` reduces the object list. A vector shorter than
that index reads element zero (`get_at`) and grows on write, filled from its
first element (`Config.hpp` `set_at`).

## Position semantics

These follow upstream exactly and must not be simplified:

- The call is `estimate_wipe_tower_polygon(config, index, wt_pos, wt_size, 0,
  false, &relocation)`: plate extruder size 0, plate objects only, relocation
  requested. Relocation changes the position, so it is not optional.
- The position is written back on every applicable plate, not only when it
  moved (`:6505-6508`). The value passes through `float`, so a stored position
  that a `float` cannot hold exactly changes in the same way as upstream.
- A position within 5 mm outside the valid bounds snaps to the bound. One
  further out is re-placed with the comfort margin, then moved to the nearest
  clear spot if objects are in the way.
- When no clear spot exists, the tower stays at the re-placed position over the
  objects. Upstream only logs this and continues.

The fitted position replaces the stored one for the whole slice: the tower
G-code, the config block's two `wipe_tower_x` and `wipe_tower_y` renderings, and
every consumer of `print.wipe_tower_x` and `print.wipe_tower_y`.

## Type1 rib footprint sizing boundary

The B50 `bbl_type1_prime_tower_waypoints_two_categories` witness also covers the
same pre-slice fit, but its first non-clock difference is before placement
clamping. After removing only generator and estimated-time lines from the
complete outputs, the Native and Ares headers report:

| Value | Native | Ares |
| --- | ---: | ---: |
| fitted `wipe_tower_y` | `219.915` | `217.415` |
| estimated footprint width/depth | `31.6569` | `34.156853` |

The `2.5` mm position difference is therefore the footprint difference, not a
clamp tolerance or nearest-clear relocation result. The Ares intermediate
values for the two Type1 filaments are:

| `estimate_rib_tower_bbox_side` input/result | Value |
| --- | ---: |
| filament IDs | `[0, 1]` |
| adhesiveness categories | `[100, 705]` |
| `tower_blocks_depth(purges, 35)` | `23` |
| aligned square | `28.5` |
| `tower_blocks_depth(purges, 28.5)` | `27.5` |
| `rib_tower_bbox_side` | `34.156853` |

The Native footprint log reports `31.6569`; with the pinned Native
`rib_width = 8`, `extra_rib_length = 0`, and the same rib-footprint formula,
that corresponds to the Native kernel's smaller aligned body (`26`) and
second depth (`25`) rather than Ares's `28.5` and `27.5`. The first changed
intermediate is consequently the Type1 category purge-depth calculation, and
its propagated effects are alignment, second depth, and final rib side. The
fix must compare the per-category purge inputs and `tower_blocks_depth` before
changing `estimate_wipe_tower_polygon` clamp behavior.

The source boundary is the single upstream footprint chain:

- `WipeTower::estimate_tower_blocks_depth` and
  `WipeTower::estimate_rib_tower_bbox_side`, `GCode/WipeTower.cpp:1760-1819`.
- `estimate_wipe_tower_footprint`, `GCode/WipeTowerEstimate.cpp:86-245`.
- `PartPlate::estimate_wipe_tower_polygon`, `slic3r/GUI/PartPlate.cpp:4065-4155`.

The Ares destination is `project_slice/gcode_emit/wipe_tower/estimate/` for the
footprint kernel, called through `plater/plate_list/wipe_tower.rs::Towers::polygon`
and `plater/slice_tower_fit.rs`. The existing two-category Native stream
witness is the red test boundary; no separate fixture is required.

## Deferred behavior

- Belt printers, along with the rest of the belt path.
- The upstream warning and info log lines (`:6500-6506`). Ares has no equivalent
  logging channel on this path.
- Arrange's tower placement and `set_default_wipe_tower_pos_for_plate`, which
  already live on the plater session.

## Verification

- **Moved tower.** `tests/parity/validation/timelapse-unprintable-gcode.3mf`
  stores `wipe_tower_y = 220` on a 220 mm bed. Native moves the tower and
  succeeds. Ares must succeed and match Native's complete output, with only the
  generated-by line normalized, through both the byte and processed entries.
- **Premises read live.** The test confirms that the stored position is outside
  the bounds and that Native's output carries a different one, instead of
  storing Native's fitted value.
- **Residual difference.** Forcing `wipe_tower_y = 180.313` in Ares leaves 22
  G-code lines differing from Native in their last digit. That input rounds the
  position to three decimals. The fit must remove this difference by carrying the
  exact `float`; any line that remains is localized, not attributed to rounding.
- **Non-zero plate index.** A multi-plate project whose tower needs fitting on a
  later plate covers the index handling, if an existing fixture can express it.
  If none can, that gap is reported rather than filled with a constructed input.
- **Regression.** The `P->F` delta against the base must be empty in official and
  seeded modes. The fit runs on every prime tower slice, so a fixture that changes
  is examined, not assumed to be an improvement.

## Post-emission rejection coverage

`GCodeProcessor::check_multi_extruder_gcode_valid` and the `-102` exit
(`OrcaSlicer.cpp:7195-7205`) are unchanged and keep their Ares port. An off-bed
stored tower can no longer reach them, so their coverage moves to an input that
does: the same project on a 150 mm bed with the tower disabled and
`skirt_distance = 40`, whose skirt extrudes past the bed edge while the object stays
inside the print volume. Native returns `-102` for it in official and seeded modes.
The test derives that input at runtime from the tracked fixture and confirms the
Native result live.

`timelapse_original_smooth_sweep_retains_minus102_control` is removed. It never
called Ares, and its premise is gone: at this pin Native moves the tower and
succeeds. `option-timelapse-type-bounded-coverage.md` records the correction.
