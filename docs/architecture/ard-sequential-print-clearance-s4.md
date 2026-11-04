# SPEC — S4 Print::validate sequential clearance Rust rewrite

Status: accepted; implementation follows independent current-source
RED. Base 96577fff7. This slice covers S4 validation only; by-object G-code
emission and order/transition changes are a separate slice; do not edit emit.

## Upstream source / exact boundary

OrcaSlicer2.4.2 Print.cpp:613-930 Print::sequential_print_clearance_valid (comments
at643/834 name horizontal/vertical subchecks; they are not separate functions).
Print.cpp:1284-1309 Print::validate calls it only for ByObject with >1 PrintObject
or >1 instance, after multi-filament validation and before later layer limits.
Nonempty message gets STRING_EXCEPT_OBJECT_COLLISION_IN_SEQ_PRINT at1300.
Print.hpp:1120-1122 is_all_objects_are_short; Print.cpp:3777-3797
object_skirt_offset (skirt flow width/spacing, per-object skirt/draft-shield).
Print.hpp:42 MAX_OUTER_NOZZLE_DIAMETER=4.

OrcaSlicer.cpp:6062-6096 prints err.string+newline, switches collision type to
CLI_OBJECT_COLLISION_IN_SEQ_PRINT at6080-6082. Utils.hpp:65 defines -63 (Unix193),
NOT CLI_VALIDATE_ERROR=-51. Generic JSON string cpp:150 is
"Object conflicts were detected when using print-by-object mode. Please verify the slicing of all plates in Orca Slicer before uploading."
Default no_check=false records generic text; no_check detail override is deferred
because Ares has no such CLI option. Reuse existing cpp416-463 JSON/396-413 flush
contract and already-applied metadata, selected plate1-based number.

## Current Ares audit / Rust destination

crates/ares-core/src/print/validate.rs::validate_applied_project currently covers
applied widths, layer-template addressing and bed/filament compatibility, but no
sequential hull/vertical clearance checks. validate_loaded_project and
project_slice/state.rs selected-plate validated path share this boundary.
printer_options/remaining.rs:36-39 already owns typed radius/rod/lid/nozzle_height;
process_options/print_source.rs owns typed print_sequence and per-object skirt
fields. Do not introduce dynamic config access or duplicate config snapshots.
Existing Project object/instance/volume transforms and core scaled convex-hull,
polygon offset/intersection primitives supply geometry; do not parse G-code or
run slicing to obtain bounds.

Destination new bounded print/validate/sequential_clearance.rs (split further at
400LOC if necessary), called from validate_applied_project using selected Project
and resolved typed options; callable via validate_project_plate before emission.
Introduce a distinct typed sequential-collision SliceError carrying exact message
and existing four applied metadata fields/types, so CLI failure.rs exhaustively
maps -63 rather than generic Validate -51/string matching. Reuse selected plate
source and f32 widening; filesystem-free core across WASM/Windows/macOS/Linux.
CLI only cfg-gated Linux JSON metadata+exact error channels, no emitter changes.
Configuration-only validate_print/settings cannot know model geometry; retain its
existing contract rather than inventing fake empty geometry or a fallback.

## Included algorithm and precedence

- ByObject multiple-object/multiple-instance guard. Strict per-instance rotated,
  scaled, mirrored convex hull and centered XY translation matching Print.cpp:
 659-674/Geometry::assemble_transform/ModelObject::convex_hull_2d.
- Horizontal hull dilation651-674: float-scaled round offset,0.1mm tolerance;
  normal half clearance_radius + object_skirt_offset -0.1mm; all-short uses
  max(half4mm nozzle outer diameter, skirt_offset)-0.1mm. The -0.1 shrink is
  semantically required, not replaced with arbitrary epsilon/bbox overlap.
- object_skirt_offset dependency3777-3797 included for per-object skirt controls;
  no blanket skirt0 exemption in implementation. Reuse upstream flow-derived
  width/spacing typed paths rather than approximate nozzle width.
- Each hull checked against prior hulls, first collision per current instance,
  exact names/newline concatenation; excluded-area check667-688 uses undilated
  hull and selected plate origin. Diagnostics respectively "%1% is too close
  to others, and collisions may be caused." and "%1% is too close to exclusion
  area, there may be collisions when printing." Preserve upstream evaluation
  order; no all-pairs alphabetic sort/dedup or early first-object proxy.
- Active order is model object_index sort826-829, NOT disabled# if0 arrangement
  sort740-824. Do not mutate emission ordering or arrange_order.
- Vertical870-928: inspect instances in that model order; reverse-inflate the
  horizontal bbox by half_radius+skirt_offset for current Y span; overlap with
  any later inflated Y bbox (>0 strict) sets limit rod height. Otherwise use lid
  height except last instance uses printable_height. Compare actual max_z, not
  model height or all-but-last unconditional limit. Exact "%1% is too tall, and
  collisions will be caused." plus upstream newline aggregation.
- Preserve earlier applicable multi-filament rejections and positioning before
  later height/width checks per Print::validate. Include ByObject multi-instance
  smooth-timelapse/clumping preconditions1285-1295 using existing typed
  print_source.timelapse_type and gcode_source.enable_wrapping_detection;
  those precondition messages stay generic Validate/-51, NOT collision/-63.

## Deferred explicitly

By-layer clearance and STRING_EXCEPT_OBJECT_COLLISION_IN_LAYER_PRINT/-64;
UI collision polygons/height overlays/pointers and selection behavior; disabled
arrangement algorithm; emitter by-object ordering/arrange_order mutation;
no_check CLI option, unrelated warnings, prime-tower checks, full future Print
validation parity, success JSON. No Ares-owned planning or progress feature.
Existing generic applied validation is a temporary compatibility shell around
cited Print::validate slices; S4 replaces the missing sequential behavior only.

## Fresh Native proposed RED inputs / observed evidence

Stock config base tests/parity/orca_cli_ender3.3mf (Creality Ender-3 0.4 nozzle):
radius47mm, rod25mm, lid25mm, printable_height250mm. Two DISTINCT objects/meshes
S4_first/S4_second, actual package relationships and ids distinct; print_sequence
must be upstream serialized "by object" (not "by_object"), skirt_loops0 to
isolate the initial minimal witnesses; tower off. Other clearance fields stock.

Fresh seed0 single-core captures with the existing Native, exit0 session.
Per-input/binary SHA/args metadata,
whole channels/result.json retained; no native rebuild.
1. close:10mm cubes at(110,110)/(125,110), gap5mm. Unix193; exact stderr
   "S4_second is too close to others, and collisions may be caused.\n".
2. rod-tall:first40mm cube at(110,110), last10mm at(180,110), no horizontal
   collision but Y overlap. Unix193; exact stderr
   "S4_first is too tall, and collisions will be caused.\n".
3. lid-tall:first40mm at(110,110), last10mm at(110,180), Y disjoint. Unix193,
   same tall detail, exercises lid limit rather than rod.
4. last-tall:same Y-disjoint placement, first10mm and last40mm. Nativeexit0;
   emitted plate_1.gcode confirms last-instance printable-height exemption.
Every rejection stdout "run found error, return -63, exit...\n"; exact full JSON
return_code-63,plate_index1,layer_height0.20000000298023224,wall_loops2,
sparse_infill_density15.0 and generic cpp150 error_string; no final G-code.
Prior-built Ares binary (bff-based -100 gates, SHA recorded) accepts all four
exit0. This is a baseline runtime witness,
NOT current main/new RED hash execution; independent current-source RED follows.

Authoring setup-invalid attempts retained/excluded: invalid enum serialized to
by-layer; zip directory entries rejected by Ares; shared mesh hit pre-existing
shared_mesh_centering; missing package relationship caused Native to ignore the
second object. Canonical Y8RAZn has distinct source files/ids+relationships, both
Native rejection and Ares successful slicing; none of setup attempts count RED.

## TDD / gates / workflow

Land this spec and the native fixture with independent RED:
process E2E exact193/stdout/stderr/full JSON/no final export, plus passing far/low,
last-tall/ByLayer/single-instance and rod-vs-lid discriminating controls.
Add radius contact(-0.1), rotated/mirrored and per-object skirt/excluded-area
cases, selected nonminimum plate metadata and error-precedence controls before
production. Geometry failures listed above; no unit tests written after code.
GREEN is typed geometry/diagnostic/CLI mapping only; preserve RED separately.
Affected validate API/process E2Es and existing CLI family guards, fmt, clippy
workspace/all-target-Dwarnings, actual CLIwasm32-Dwarnings; workspace nextest.
Fresh base/candidate seeded + official runs with
independent targets/caches.
Same-label raw/Native comparisons, real per-group exit codes and setup/NativeND
residual disclosure. Land fast-forward only.
