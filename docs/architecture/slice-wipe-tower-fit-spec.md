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

## Type1 WipeTower rib plan and emission boundary

The comparisons below record successive pre-fix states that establish the
included source behavior, not accepted output differences. Exit requires complete
Native/API/CLI byte equality under the existing generated-by-only comparator,
including progress, geometry, bed-mesh commands and time headers.

The same witness shows that matching the fitted position is not sufficient for
this slice. With `wipe_tower_x = 15` and `wipe_tower_y = 219.915` matching on
both sides, the first `WIPE_TOWER_START` section still has different emitted
geometry:

| First tower section | Native | Ares |
| --- | ---: | ---: |
| X range | `12.527..47.505` | `12.391..50.589` |
| Y range | `214.705..250.420` | `214.708..252.037` |
| command lines through `WIPE_TOWER_END` | `234` | `215` |

This geometry is included by `first_layer_convex_hull` in the adaptive bed mesh
bounds (`GCode.cpp:3895-3903`, `:3945-3950`), so the resulting `G29` and the
published time estimate remain part of the same complete-output witness. The
remaining source boundary is Type1 `WipeTower` rib planning and emission,
not a comparator normalization or a placement exception. Although the stored
`wipe_tower_type` is `type2`, the BBL printer selects Type1 through
`Print::wipe_tower_type`; Ares mirrors that selection in
`project_slice/gcode_emit/wipe_tower.rs::requested`. The earlier Type2
attribution was incorrect and does not authorize changes to `WipeTower2`.

The included upstream slice is:

- `GCode/WipeTower.cpp:4685-4774`, `calc_block_infill_gap`: calculate each
  category's purge-line gap, then recompute toolchange depths and reset the
  global extra spacing to one;
- `GCode/WipeTower.cpp:4776-4785` and its following rib branch,
  `plan_tower_new`: generate blocks before squaring, sum their maximum depths,
  align the square tower width, and replan each toolchange at that width;
- `generate_wipe_tower_blocks`, called before and after rib squaring, including
  category depth aggregation and downward depth propagation;
- the remainder of `plan_tower_new`: first-layer solid-fill lengths, block
  offsets, final tower depth, height minimum, diagonal rib length and clamped
  rib width;
- the rib polygons and emission called by `generate_new`, only where the
  complete-output witness demonstrates a divergence.

Ares owns this destination in
`project_slice/gcode_emit/wipe_tower/type1/plan.rs`, `blocks.rs`, `generate.rs`
and their Type1 rib geometry consumers. Replace the existing `plan_tower`
implementation with the source-cited rib planning sequence; do not retain an
old-planner fallback. The included witness is the single-physical-nozzle BBL
rib path with categories `[100,705]` and no nozzle-change ramming. Other wall
types, carousel/nozzle-change and reverse-travel gap branches are deferred by
this slice; no new behavior for those branches is inferred from this witness.

`WipeTower.cpp:2004` initializes planning `m_extra_spacing` from
`prime_tower_infill_gap / 100`, not `wipe_tower_extra_spacing`. In the included
no-ramming path, every category retains the same purge-line gap
(`m_perimeter_width * m_extra_spacing`, `:4723-4730`) while the global spacing
is reset to one (`:4764`). The width replan and non-first-layer purge emitter
must continue consuming that retained gap (`:3108-3112`, `:4261`); resetting the
global spacing must not discard it. First-layer solid-fill lengths are derived
from the replanned depth unconditionally (`:4827-4843`). The current witness's
Native trace has width `35`, gap `0.75`, first block depths `11.25/7.5`, square
width `26`, final block depths `15/10.5`, and first-layer wipe length `750`.
The earlier preliminary Ares mapping to `wipe_tower_extra_spacing` was incorrect.

Compare the initial toolchange volumes/depths, per-category gap and maximum
depths, square width, replanned depths, block starts and final rib dimensions
before changing the rib polygon emitter. The earliest differing operand is the
fix boundary; emitted coordinates alone do not establish its cause.

After the spacing and solid-fill correction, the retained numeric comparison
matches all 50 layers and both blocks at seven corresponding Native/Ares
planning stages. The complete output still differs: the first tower travel
is missing Native `M204 S6000`, and the following extrusion is missing
`M204 S500`. The consumed emission slice therefore includes
`WipeTowerWriter::extrude_explicit` and `extrude_arc_explicit`
(`WipeTower.cpp:874-878`, `:953-957`), the writer setup
(`set_for_wipe_tower_writer`, `:2921-2932`), acceleration vector rounding
(`:4513-4525`), and `set_normal_acceleration`, `set_travel_acceleration`,
`set_acceleration_impl` (`:1489-1551`). `Print.cpp:5309-5315` supplies the
resolved object-scope vectors, including first-layer travel percentages.
The Ares destination is `type1/generator.rs` for per-physical-extruder values
and `type1/writer.rs` plus its arc emitter for the shared last-acceleration
cache, machine-extruding limit and upstream flavor formatting. Reuse typed
configuration and the existing nozzle mapping; do not add an Ares-owned
acceleration policy. The existing complete-output witness covers the BBL
Marlin command path; other flavor formatting follows the same source method,
without adding fixtures or broadening the slicing boundary. The missing
acceleration commands are not assumed to explain every residual difference.

The next retained-output differences include merge travel `F5400` instead of
`F5400.000000`, followed by a missing `M220 B` at the first real toolchange.
Include `merge_tcr` (`WipeTower.cpp:3353-3371`) with its six-decimal configured
`m_max_speed` (`:2007`), and the Marlin speed-override backup/restore methods
(`:1271-1285`) called at `:3545-3546` and `:3602`. Their Rust destinations are
`type1/result.rs::merge`, its Generator callers, and `type1/tool_change.rs`.
`get_next_pos` (`:2235-2262`) also consumes the retained purge gap for the
initial Y offset; the current Ares equivalent still uses bare perimeter width.
Keep this position calculation in the same retained-gap correction, without
including deferred nozzle-change, reverse-travel or PETG pre-extrusion branches.
The configured maximum also owns finish/purge speed caps (`:3825`, `:4018`,
`:4265`), replacing the current fixed `5400`/`4800` values.

With those emission corrections the complete model/total time headers match,
but the witness still fails on progress placement. The next non-progress
mismatch is the first real toolchange's wipe tag and extrusion flow: Native
emits `CP_TOOLCHANGE_WIPE CT0 FL1` and width `0.500000`, while Ares emits
`CP TOOLCHANGE WIPE` and width `0.575000`. Include the consumed
`toolchange_wipe_new` setup (`WipeTower.cpp:4229-4239`) and
`Print.cpp:5290-5296::set_first_layer_flow_ratio`: use the typed region's
`first_layer_flow_ratio` only when object `set_other_flow_ratios` is enabled,
otherwise one; do not hard-code `1.15`. Emit the upstream standalone tag,
contact/first-layer flags and resulting width. Destination: `type1/wipe.rs`.
The complete comparator remains unchanged and progress commands remain in scope.
The subsequent non-progress difference is wipe line six (`Native F2675`,
unchanged thereafter; Ares continues `F2725`, `F2775`, ...). Port the bounded
five-entry `WipeSpeedMap` and last-entry retention (`WipeTower.cpp:4267-4269`,
`:4335-4336`) in the same `type1/wipe.rs` boundary. Replace the old indefinitely
increasing `next_speed`; do not retain it as a fallback. Adjacent interface
cooling slowdown remains deferred: upstream initializes its controlling boolean
to false and never enables it in this path (`:4258`, `:4318`).
After the bounded speed-map port the next physical-stream mismatch is Native's
`M400` / `M104 T0 S220 N0` between the entry ironing and the first full wipe line
on layer two. Include the active `should_heating` / `add_M104_by_requirement`
branch (`WipeTower.cpp:4300-4306`, `:4368`, `:4386`, `:4390`) and the source
`format_line_M104` formatting (`:1443-1458`). Consume the already typed nullable
`filament_cooling_before_tower` (witness `10,10`); enable only above EPSILON for
non-contact, non-first-layer wipes. Emit `M400`, the optional physical-extruder
map `T` argument, target normal nozzle temperature, and `N0` at the source
location. Destinations: `type1/wipe.rs` and the source-owned Writer formatter
`type1/writer.rs::format_line_m104`; no CoolingBuffer changes, inferred
slowdown, new temperature model, or deferred nozzle-change behavior is included.

The next geometry divergence is the second layer's rib perimeter entry:
Native `(42.882,246.065)` versus Ares `(43.016,245.931)` after placement.
A focused Native return trace of `get_all_wall_skip_points` captures its layer
one skip point `(26,25.25)`. All planner operands match Ares, but
`type1/wall_gaps.rs::prepare` still recomputes a `7.0` wipe depth from bare
perimeter spacing instead of using planned `10.5`; it also subtracts `0.5`
instead of the retained gap `0.75`. Its current arithmetic therefore yields
`(26,22)`. Include the rib witness's `get_wall_skip_points` arithmetic
(`WipeTower.cpp:3413-3451`): use required depth less nozzle-change depth and
subtract layer spacing times the retained block gap. Destination:
`type1/wall_gaps.rs`; leave unrelated ramming/TPU/contact-access branches deferred.
The focused Native trace's full G-code equals the complete retained Native
witness after generated-by normalization only (`534100` bytes each).

The common root-cause check is the typed configuration boundary. Upstream
`PrintConfig.cpp:8953-8968` defines the per-filament
`filament_prime_volume` and `filament_prime_volume_nc` vectors, while the Ares
Type1 estimator and Type1 generation consume typed settings. The witness
contains `filament_prime_volume = 30,45`, but the previous typed path exposed
only the registry/default `45`, producing `[45,45]` where the raw input is
`[30,45]`. The implementation must add the evidenced
`filament_prime_volume` field to the typed filament G-code options and make both
estimation and generation consume that same typed value. It must not retain two
independent raw overlays. Once typed materialization is complete, the
`new_with_raw` overlay used only by `slice_tower_fit.rs` is removed.
`filament_prime_volume` is not in upstream `filament_options_with_variant`
(`PrintConfig.cpp:10150` onward): neither variant expansion nor active-variant
reduction may touch this per-filament vector. Type1 allocation reads the new
filament's EC volume (`Print.cpp:5386-5399`), not the global `prime_volume`.

Adjacent `filament_prime_volume_nc` is documented by the same upstream
registration but is deferred unless the witness proves that this path reads
it; no unobserved configuration keys are added in this slice.

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
