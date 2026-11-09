# Estimated prime-tower printable clearance

Rewrite the pre-generation tower/printable-region rejection in OrcaSlicer
v2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c` literally. The existing
applied-print validator is a temporary compatibility shell around `Print::validate`,
not an independently designed validation pipeline. A printable-region decision
must use the same estimated tower that Native validates, not configured width
alone, an axis-aligned approximation of its rotated hull, or a new placement rule.

## Owning source and Rust destination

- `src/libslic3r/Print.cpp:1584-1737`,
  `layered_print_cleareance_valid`: applied plate origin, tower body,
  `tower_polys_checked` and shared-printable `diff`.
- `Print.cpp:1650-1695`: `extruders(true)`, per-plate position, float-stored
  width/depth/brim and rotation; pre-generation explicit-brim checked hull.
- `Print.cpp:1728-1733`: live shared printable polygons translated by plate
  origin; nonempty difference returns the tower-outside diagnostic.
- `Print.cpp:2235-2273`, `Print::validate`: sequential multi-instance branch
  versus layered-clearance branch and layered collision error classification.
- `Print.cpp:5021-5034`, `get_extruder_shared_printable_polygon`:
  printable-area fallback versus ordered intersection of extruder printable areas.
- `Print.cpp:5183-5226`, `has_wipe_tower` and `wipe_tower_data`:
  eligibility and the shared pre-generation footprint producer.
- `src/libslic3r/GCode/WipeTowerEstimate.cpp:86-245`,
  `estimate_wipe_tower_footprint`, with its called rib/brim calculations in
  `GCode/WipeTower.cpp`; reuse their existing Rust translation.

Add a small private module under `crates/ares-core/src/print/validate/`, invoked
from `validate_applied_project` at the applied layered-clearance boundary. Keep
configuration resolution, geometry and rejection in the byte-oriented core, with
no filesystem, terminal, UI or platform-specific dependency. Browser WASM,
Windows, macOS and Linux share the same implementation.

Reuse `project_slice/gcode_emit/wipe_tower/estimate.rs`,
`estimate/footprint.rs::estimate_footprint` and
`estimate/config.rs::DynamicPrintConfig`. Adapt the existing applied project
inputs to these crate-private functions; do not copy their sizing algorithm into
validation or tests, widen the public API for a fixture, or create another tower
planner. Reuse `SliceError::LayeredPrintCollision` and the existing CLI `-64`
family in `crates/ares-cli/src/failure.rs:298-322` / `failure/report.rs:18-55`.

## Included behavior

1. Follow Native's caller branch, including by-object with a single object and
   instance entering the layered branch. Do not replace its branch condition with
   a by-layer-only guard. Preserve existing earlier error precedence; do not move
   unrelated validation checks or change their error families.
2. Collect Native-equivalent `extruders(true)`, including custom G-code
   MultiAsSingle tool changes. This is neither configured slot cardinality nor
   nozzle count. Preserve slot ids and mixed-filament expansion in the existing
   estimator. Bind the effective applied config to the Native `PrintConfig`
   inputs; do not silently substitute an unresolved raw project-settings view.
3. Mirror `wipe_tower_data(filaments_count)` before `psWipeTower`:
   maximum applied `PrintObject::size().z()` converted from scaled units,
   minimum object layer height and existing early-return rules. Pass the actual
   filament ids and resolved tower type to the existing footprint producer.
   Narrow resulting width, depth and resolved brim to f32 at the Native cache
   boundary. Raw `prime_tower_width` is not the effective rib-tower width.
4. Preserve `has_wipe_tower`: belt exclusion, enabled-prime flag, wrapping
   detection with its exclusion polygon, smooth timelapse, spiral mode and
   configured filament-diameter cardinality. Do not add an independent gate
   around the final difference test or lose a single-filament wrapping tower.
5. Read selected-plate x/y with Native vector `get_at` semantics, add plate
   origin before assignment to f32, and narrow rotation angle to f32. Build the
   four scaled corners with Native's f32 `x+width` / `y+depth` operations before
   scaling. Rotate about the scaled anchor in the same order. Bind
   `Geometry::deg2rad` promotion and `Polygon`/`Point` rotation rounding to the
   pinned source; do not use an all-f64 rectangle, f32 trigonometry substituted
   for promoted operations, a bounding-box test or a numeric epsilon.
6. For pre-generation `prime_tower_brim_width >= 0`, offset the body by the
   resolved brim using Native's `float(scale_(brim_width))` delta and existing
   polygon-offset semantics. For automatic brim, the checked hull is the body,
   not the estimated brim/cone outline. Preserve Native empty/degenerate hull
   behavior instead of inventing internal input validation or positive-size
   defaults.
7. Derive the live printable polygons exactly as `5021-5034`: fewer than two
   nozzles, no extruder areas, or any area with fewer than three vertices uses
   scaled `printable_area`; otherwise intersect all configured extruder areas
   in source order. Do not select only active-tool zones or use an intersected
   region's bounding rectangle. Translate polygons by the scaled plate origin.
8. Apply the existing integer polygon difference operation to the full checked
   hulls and live printable polygons. On nonempty difference return exactly
   `Prime tower is partially outside the printable area, and it cannot be printed.\n`
   in the layered collision family. Keep plate numbering, applied f32-widened
   layer height/density and wall-loop metadata consistent with that family.
   The CLI owns its existing stdout/stderr/result serialization and OS exit
   conversion; the core does not print.

## Adjacent behavior inventory

| Native behavior | This slice |
| --- | --- |
| Pre-generation body and explicit resolved-brim checked hull, `1650-1695` | Included |
| Automatic brim: body-only hard printable check, `1689-1695` | Included; no estimated-brim hard rejection |
| Type2 cone: actual width/depth from shared sizing, same checked-body rule | Included; no cone-base hard rejection added |
| Estimated first-layer outline/brim/cone warning geometry, `1697-1705,1722-1726,1735-1736` | Deferred warning translation; never promoted to error |
| Body/object proximity warning, `1706-1711` | Deferred |
| Tower/exclusion and tower/wrapping hard intersections, `1712-1721` | Deferred tower-specific checks; existing object/wrapping logic unchanged |
| Post-generation `wipe_tower_mesh_data->bottom`, `1677-1685` | Deferred; no substitution of an estimated rectangle for a generated bottom |
| Exact generated-footprint check in `_make_wipe_tower`, `5627` onward | Deferred with its generated-data owner |
| PartPlate pre-slice fit/relocation, `PartPlate.cpp:4065-4155` | Unchanged; not repaired or bypassed here |
| Purge planning, G-code paths, fans, footer dispatch, elapsed-time calculation | Unchanged |

This is explicitly a pre-generation fragment, not complete layered-clearance or
post-generation parity. No alternate-position retry, newly conservative cone
rejection, legacy geometry fallback or feature flag is part of the port.
Overlapping deferred diagnostics need a separate source slice to reproduce their
full precedence; this slice must not report them as matched behavior.

## Failure-first verification

Use the unchanged `tests/parity/validation/tiq2-type2-enabled.3mf` input
(SHA256 `0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942`)
as a new failing public E2E control. Do not move its tower, alter its angle,
change its settings, or rewrite the five fan/footer tests into expected errors.
Run Native first with source/input/catalog/resources/actually executed ELF
identities retained, then the same input through Ares. Before the port Native
returns OS192 / result `-64` with the tower-outside stderr and no G-code, while
Ares returns0 / Success and emits G-code. This is the failing behavior the port must change.

The accepted seeded read-only trace executes fit with `moved=false`, then reaches
`Print.cpp:1733`; its rotated checked hull escapes the live printable polygon.
Stored positions, dimensions, margin results or trace hull vertices are not
replacement expectations. The optimized-away `exact_footprint` operand remains
unmeasured; this spec selects the pre-generation source boundary by lifecycle,
not by claiming that unavailable observation was recovered.

Failure inventory before implementation:

- using configured width instead of effective rib footprint;
- counting configured slots, omitting custom tool changes, or wrong config view;
- wrong plate origin/index, f64 arithmetic replacing f32 narrowing, or wrong
  scale/rotation/offset rounding;
- automatic brim or cone estimate incorrectly becoming a hard error;
- disabled, spiral, belt or single-filament wrapping eligibility changing;
- wrong shared-zone intersection/fallback or a rectangular admission shortcut;
- returning `-51` rather than the existing layered `-64` diagnostic family;
- emitting G-code after rejection or hiding raw stream/result differences;
- accidentally changing a passing fan/footer/Type1/Type2 control.

Retain exact exit/stdout/stderr, full raw result.json and G-code presence/absence
for the original negative and Native-successful controls in both oracle modes.
Keep the complete ordered G-code comparator and existing result policy unchanged.
Native's captured export-time field is not established as deterministic; do not
copy its observed value into Rust or add a new normalization/filter to obtain a
pass. Any unresolved whole-result inequality stays visible and prevents a
whole-output parity claim.

After the failing test is committed, implement only the included source slice, rerun the same
E2E/control matrix, then fmt and locked/offline strict workspace/all-targets/
all-features Clippy without suppressions. Keep changed files at most400LOC.
Require original input rejection with exact diagnostic/family and no G-code,
Native-successful controls remaining accepted, no pass-to-fail transitions and
unchanged unrelated output bytes. Newly exposed production differences need
another source-first spec, not expanded scope or comparator weakening.

## Independent semantic-fixture input setup

After this spec is published, derive each of the five existing fan/footer
fixtures' tower anchors from the live printable rectangle and the object's
bed-projected bounding box, using public 3MF/instance data. Among corners whose
inward quadrant follows the original tower rotation, prefer the quadrant farthest
from that bounding box. Inset only by the brim resolved from the same config
bytes, including Native's automatic-brim rule and actual-width quantization
(`WipeTower.cpp:1733-1750`); choose an inward-representable anchor without an
added margin. Change only selected-plate wipe_tower_x/y; retain the original
angle and all other model/settings/template intent. No footprint estimation,
public API expansion, core-internal fixture generator, stored positions,
committed3MF, tower disabling or fitted expected output. Run official Native once
per fixture; successful export is the printable/conflict validity premise. If any
fixture returns nonzero, report its exit code and stdout; do not retry another
position. This setup does not replace the unchanged original negative control
and is not production placement or validation logic.
