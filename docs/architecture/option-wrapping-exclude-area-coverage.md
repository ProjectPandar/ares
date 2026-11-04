# Bounded wrapping-exclude-area option coverage

Status: implemented bounded harness and wrapping-admission port.
The historical [full Option audit](2026-10-09-option-full-current-audit.md)
records `wrapping_exclude_area` INCOMPLETE, baseline-only with no injection.
The preceding harness excluded it, citing a rectangle rejection without
establishing its context. This slice is not a fresh full Option audit.
Selected context: single nozzle, single filament, no tower, by-layer.
Initially harness-only; after the fixed-pose Native -64 / Ares acceptance RED,
the slice added the production wrapping-admission port described below.

## Source boundary

OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

| Upstream boundary | Contract |
| --- | --- |
| `libslic3r/PrintConfig.cpp:4120-4125`, `PrintConfig.hpp:1358` | `wrapping_exclude_area` is coPoints, default empty; individual XY points, not coPointsGroups polygons or a numeric endpoint domain |
| `libslic3r/Config.hpp:1487-1550::ConfigOptionPoints`, `Config.cpp:976-1000` | vserialize emits one XY string per point; JSON arrays join with commas before deserialize. Preserve requested point order and coordinates |
| `libslic3r/Print.cpp:932-979::layered_print_cleareance_valid` | Add plate origin to wrapping points, scale them into a polygon, and reject its intersection with an instance volume hull when detection is enabled |
| `slic3r/GUI/PartPlate.cpp:5454-5485`, `OrcaSlicer.cpp:4761,4789,5717-5724` | Arrange treats wrapping as a virtual obstacle; an enclosing polygon can relocate the instance outside the bed and cause pre-apply -50, before Print hull admission |
| `Print.cpp:1304-1310::Print::validate` | By-layer path invokes the preceding validation and assigns `STRING_EXCEPT_OBJECT_COLLISION_IN_LAYER_PRINT` |
| `Print.cpp:1324-1333` | Detection with prime tower disabled produces a warning, not this branch's fatal rejection |
| `PrintConfig.cpp:8688-8712::normalize_fdm_2`, `Print.cpp:3290-3303::has_wipe_tower` | Tower activation is separately gated; keep enable_prime_tower=0 and verify no tower is emitted |
| `GCode.cpp:5215-5236,5302-5305` | Once-per-layer wrapping retract/template insertion gates on enable_wrapping_detection; the no-tower insertion does not require area point count >2 |
| `GCode/GCodeProcessor.cpp:2193-2202::apply_config` | Carries wrapping points into result metadata |
| `GCode/GCodeProcessor.cpp:1866-1999` | Wrapping-polygon collision code is commented out; do not claim it enforces the active Print hull contract |
| `OrcaSlicer.cpp:6083-6084`, `libslic3r/Utils.hpp:66` | Layered object collision maps to CLI -64; preserve actual Native stage, result.json, status and diagnostic |

Rust counterparts are existing `options/printer_options/gcode_source.rs`
Point2dList, `options/gcode_fields.rs`, project/effective-config resolution,
`project_slice/gcode_emit/layers/wrapping.rs`, and the public
`validate_project_plate` / `slice_project_plate_validated` boundaries.
`print/validate.rs` currently calls sequential_clearance, whose entry returns
for by-layer; the wrapping hull rejection is not evident in that path. This
read-only observation is a RED candidate, not proof of a production defect.
Legacy STL `gcode_wrapping_detection.rs` postprocessing and its point-count
suppression tests are a different boundary; they do not define project slicing
or Native expectations for this slice.

## Included finite contract

Reuse existing cube10 STL, runtime Native export to 3MF, then project slicing.
Ender-3 0.4mm, exactly one nozzle/filament/object/instance, first plate,
by-layer, 0.2mm initial/ordinary height, relative E, no support/raft/tower,
ordinary filament ratio 1 and diameter 1.75. Verify exported settings and
cardinalities; no physical mapping or tool ordering overrides. Detection uses
an inert comment template `;W6_WRAP {layer_num} {layer_z} {max_layer_z}`,
which avoids physical-tool placeholders and commanded motion. Preserve the
standard start/end templates and complete M73/footer bytes.

Canonical detection-enabled requests use one string per point (`"XxY"`),
with an actual empty array for the empty list, matching ConfigOptionPoints
vserialize. The existing generic harness array carrier uses one string for a
whole authored value; this slice adds a key-only `bounds.rs::case` conversion,
without changing any other coPoints key. Requests:

- Empty point list.
- One point `(0,0)`.
- Two points `(0,0),(10,0)`.
- Off-object triangle `(0,0),(10,0),(0,10)`.
- Off-object rectangle `(0,0),(10,0),(10,10),(0,10)`.

Prove at runtime that the exported cube is clear of the two polygon interiors.
Native must actually accept each request and retain its requested points.
If a request is rejected, rewritten, or fails to emit the wrapping witness,
retain its evidence and report it before narrowing or promoting anything.
Empty/one/two-point requests do not inherit the legacy STL suppression rule:
observe Native and require project output equality, with one actual wrapping
marker per emitted layer when detection remains enabled. Compare complete
ordered output with only the existing generated-by normalization.

Dedicated detection-disabled controls repeat the same five requests, prove
marker absence and compare whole output. The enable=1 versus enable=0 controls
provide a real emitted-path witness; a CONFIG_BLOCK alone is insufficient.
Record exact point lists, effective flag, template, positive extrusion,
ordered marker lines and layer-count evidence externally.

Rejection witness: detection enabled with bed-enclosing rectangle
`(0,0),(220,0),(220,220),(0,220)`, after proving the cube footprint lies inside
it. Use detection-disabled runtime Native export to lock its pose, then the existing
3MF fixture-adapter pattern changes only enable_wrapping_detection to 1; all
other settings and member payloads remain identical, without arrange. A
dedicated runner transport helper owns private ExportedCase paths; it reuses
slice_case and does not change cache recipes. Record Native actual Slice rejection, expected -64 (Unix 192), structured
CLI text and the Print validation diagnostic; compare the same exported 3MF
through public Ares validation and validated slicing. Repeat with detection
disabled as an accepted full-output control. Do not invent a processor -102
rejection, invoke script/template motion, or mask a mismatch with a capability
guard. Retain the original enabled-export arrange rejection as an additional
-50 control: verify its relocated footprint and actual CLI result.json, and
require the identical Ares validated-slice diagnostic. It is not a Print hull
rejection witness. Any actual Ares acceptance/error/byte discrepancy remains RED.
The Print admission repair below closes the observed fixed-pose rejection discrepancy.

## Harness destination, RED and GREEN

Only CLI Option coverage: new `wrapping_exclude_coverage` test/context files,
parent test-module registration, and this key's probes, key-local point conversion in `bounds.rs::case`,
activation/context, sweep hook and metadata expectation if runtime proof permits. Reuse existing
OrcaRunner export/slice and caches; no producer/cache/CLI-result infrastructure
changes, inputs or Native outputs committed. Keep modules below 400 LOC.

RED first executes actual Native success/rejection controls and strict Ares
comparisons, then asserts bounded classification. Genuine byte or rejection
mismatches precede the classification assertion and block GREEN. If any
positive contract lacks proof, keep INCOMPLETE; no rejected case is excused
into a compared PASS. GREEN only promotes these witnessed successful requests,
with canonical enable=1 context and separate rejection controls. Verify the
650-plan diff changes only this key; preserve all other expectations and
negative controls, and separately report any required expectation update.

GCode tool_transition/emit_helpers neighbors are out of scope; do not change
their production paths. The observed Print hull-admission difference is covered
only by the production slice below.

## Acceptance and deferred scope

Required seeded/official directed cold/hot, fresh unfiltered selected-key
sweep and full base/candidate controls; list every status change and common
complete Ares raw hashes. Native-only variation needs separate input/producer/
output evidence. Default-environment workspace, strict clippy, fmt, wasm.
Shared native_environment optional visible SKIPPED / required FAIL policy applies.

Deferred: towers, SEMM, multiple nozzles/filaments, physical mappings and G4
IDs; by-object/G2 clearance, nonzero plate origins, extra plates/instances,
arbitrary/self-intersecting polygons, arbitrary coordinates, touching-only
and epsilon collision boundaries, real detection motion, physical-tool
placeholders, APA, GUI warning policy, commented processor collision code,
and legacy STL postprocessing. Emission scaffolding remains unchanged; this slice establishes only the cited
project/Print boundary's finite contract.

## Production admission slice

The fixed-pose RED invokes Native `Print.cpp:932-979/1304-1310`, which rejects
with -64/192 and the exact clumping diagnostic; Ares public validation accepts
and validated slicing emits 86190 bytes. This is independent of tool-transition
emission. The original enabled arrange case instead relocates the cube outside
the bed and both slicers reject with -50; preserve it as a separate control.

Port only the enable_wrapping_detection + by-layer wrapping polygon collision
branch into `ares-core/src/print/validate/layered_wrapping.rs`, with minimal
`validate.rs` wiring. Add plate origin before scaling wrapping points. Iterate
model-part volumes of printable instances, produce each transformed XY convex
hull, and preserve the upstream scaled linear-hull/translation order
(`Model.cpp:2625-2644`). Intersection returns the exact object-name diagnostic
and the layer-collision error family; map it to CLI -64 and the upstream
result metadata/generic error (`OrcaSlicer.cpp:6083-6084`, `:138`).
Deferred adjacent exclusion-area rejection, inter-object warnings, arbitrary
transform/touching precision probes and all previously deferred emission paths
remain outside this port. No tool_transition/emit_helpers changes.

Native-backed RED precedes production. GREEN must compare public validation,
validated slicing and CLI status/channels/result.json, retain whole successful
streams, and close only this key's harness classification. Fresh workspace and
full seeded/official base/candidate controls run on the production tree, with
raw changes listed. Earlier positive streams
are directed evidence only, not a substitute for the final production gates.

## Directed GREEN observations

RED `69a015f6d` retains the actual fixed-pose Native -64 / Ares acceptance
difference. The production port now returns the same named diagnostic through
public validation and validated slicing; CLI exit/status/channels/result.json
match Native, using only the shared prepare/export wall-clock normalization
for result.json. The separately arranged -50 control remains active.

Seeded and official each completed cold and hot directed runs, and separate
canonical sweep runs. Each producer/temperature has eleven full successful
streams in the directed tests (ten enable/disable witnesses plus the disabled
intersection control) and five in the sweep, totaling 64 comparisons with only
the existing generated-by normalization. Declared request values must equal
the five runtime witnesses. No existing numeric-metadata lock asserted this
key; other keys' expectation assertions remain unchanged. Only wrapping
changes among the 650 option plans; the other 649 are identical, and the five
canonical arrays equal the runtime witnesses.

Every test-status change, in both modes, is:

- `option_coverage::sweep::option_coverage`: FAIL → PASS.
- `option_coverage::wrapping_exclude_coverage::wrapping_exclude_area_bounded_project_output`: absent → PASS.
- `option_coverage::wrapping_exclude_coverage::rejections::original_orca_and_ares_reject_arranged_wrapping_object_outside_bed`: absent → PASS.
- `option_coverage::wrapping_exclude_coverage::rejections::original_orca_and_ares_reject_wrapping_polygon_intersection`: absent → PASS.

Every common Ares raw output is unchanged, with no Ares normalization.
