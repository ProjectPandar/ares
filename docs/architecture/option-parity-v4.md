# KSR FDM Test V4 option parity

## Status

Tasks 16 through 20A.2, Tasks 22A through 22N, and Tasks 22O.1 through 22O.32
are released. O28 implements the bounded ClipperZ-backed
`Algorithm::wave_seeds` prerequisite; its final reviews approved and exact-SHA
Tier-1 run `31156094839` passed at
`be334375be871eb12ca98c98d889b65a92d13a37`. O29 source-taking propagation was
released as implementation commit `55c2c23` and documentation commit `118f6a7`;
its exact-SHA Tier-1 run `31168584784` passed all format, WASM/browser, Linux,
Windows, and macOS jobs at
`118f6a72b33926efe41ced1c931f9a51b26b2945`. O29 added no lifecycle wiring;
the current local lifecycle progression is summarized at the end of this
status section.
O30 direct supplied-seed `propagate_waves_ex` was released as commits
`0a19939`/`6ccb145`; exact-SHA Tier-1 run `31184069746` passed all five jobs at
`6ccb145dbb1867e5724538fb071795a7fd4179f0`. O31 source/scalar
`propagate_waves_ex` composition was released as commits `7113f7c`/`1f89dd3`;
exact-SHA Tier-1 run `31196271880` passed all five jobs at
`1f89dd34c9226a96b92ddc1711c317ff6ce7b7b0`. O32 `expand_expolygons` was
released as commits `2e7168f`/`699f02b`; exact-SHA Tier-1 run `31213611275`
passed all five jobs, including both browser runs, at
`699f02b2bbc3d797f53edf5f8c65dd2614830ecb`. O33 expansion merging was released
as commits `b9e65fd`/`0f6f801`; exact-SHA Tier-1 run `31228800274` passed all
five jobs, including both browser runs, at
`0f6f80130d28c0cc629e8561e46d187b137a8206`. O34
`expand_merge_expolygons` was released as commits `f499058`/`25460c2`;
exact-SHA Tier-1 run `31259140846` passed all five jobs, including both browser
runs, at `25460c2abfc5bf94104f41b05df5af2dfac419ee`. O35
`expand_merge_surfaces` was released as commits `984bc01`/`c6f23ce`;
exact-SHA Tier-1 run `31269521736` passed all five jobs and both browser runs at
`c6f23ce1a9350ca76241d007f804f3fcfa22c352`. O36 bridge-zone expansion was
released as commits `b546e6f`/`3e927ed`; exact-SHA Tier-1 run `31280579891`
passed all five jobs and both browser runs at
`3e927ed569d3db8d6f5c08b7843fb049fcc86412`. O37 bridge grouping was released
as commits `a0caa5a`/`4d83d15`; exact-SHA Tier-1 run `31291016394` passed all
five jobs and both browser executions at
`4d83d15832c7905d7ea9727d14c07c5a75eb7312`. O38 direct bridge-direction
selection was released as commits `04920e0`/`2d6154d`; exact-SHA Tier-1 run
`31303115603` passed all five jobs and both browser executions at
`2d6154d401c3c954bed69de6ba631a53af05f1a3`. O38 remains crate-private and
inactive. O39 was released as commits `2038e93`/`c84119e`; exact-SHA Tier-1
run `31317150231` passed all five jobs and both browser executions at
`c84119ee6871a176ec94117bc16f7e402c9caf96`. O39 is the bounded
`detect_bridge_directions` composition at `LayerRegion.cpp:262-308`, preserving
the supplied-order forward anchor cursor, source-width boundary casts,
contour/hole order, scaled-epsilon Miter-3 expansion, non-recombining open-path
difference, unchanged-scale O38 call, direct errors, and
`PI + atan2(y,x)` assignment. Its repaired fresh-cycle RED, 14/14 focused
GREEN, original-Orca multi-bridge/missing-boundary helper matrix, reviewed
literals/pointer ownership, M01-M28 campaign, exact restoration, and both
implementation rereviews pass. Complete exact-final-byte native/WASM/static/
rollback verification passes; both local Playwright attempts failed before test
execution on missing `libglib-2.0.so.0`, and neither was treated as a pass. The
later exact-SHA run supplies the required browser evidence. O39 remains
inactive. O40 locally implements the next `merge_bridges` boundary at
`LayerRegion.cpp:310-351`, including source-to-expansion association, root-
group collection, per-group Miter-3 flat closing, and bottom-bridge surface
materialization. Its focused tests and independent pinned-Orca closing oracle
pass; after repairing the initial review's rustfmt, coverage, and citation
findings, the same six-dimensional thread approved O40 with zero findings. It
remains inactive and unreleased. O35-O41 add no Option or lifecycle wiring.
The current local O42 activates external-surface processing after O26, O43
activates internal-bridge candidate discovery after O42, O71 consumes that
candidate state through the admitted first internal-bridge transaction, and
O72 applies the infill-combination identity gate. Public slicing consumes and
disposes O72 before returning `ProjectSlicingIncomplete`. O73 is implemented
only as a crate-private, lifecycle-inactive base fill-grouping module and does
not advance that sink. O74's grouping tail, later fill/toolpath stages,
complete G-code assembly, and normalized KSR parity remain deferred.

## Fixed baseline

The compatibility target is OrcaSlicer `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

| Artifact | Contract |
| --- | --- |
| `ksr_fdmtest_v4.project.3mf` | SHA-256 `698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`; 15 package entries |
| `ksr_fdmtest_v4.gcode` | SHA-256 `10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`; 269,330 lines; 460 layer markers |
| Normalized reference | SHA-256 `c61202df3fa26ffcb3064f2dbc02e06a89f95565b8325b31029ec4ed6cedcdc4` |

The fixtures remain test data. Production code must not read the reference,
recognize the fixture name or hash, or invoke OrcaSlicer.

## Golden comparison boundary

`Format/bbs_3mf.*` owns the project input identity, while
`GCodeProcessor.cpp` owns the generated header exception. The golden helper
requires exactly one complete UTF-8 generator line on each side:

- Orca: `; generated by OrcaSlicer 2.4.2 on YYYY-MM-DD at HH:MM:SS`
- Ares: `; generated by Ares 2.4.2 on YYYY-MM-DD at HH:MM:SS`

Each validated line is replaced by the same sentinel. Every other byte,
including line endings, whitespace, statistics, ordering, and trailing bytes,
is compared without normalization. A mismatch reports only the first byte,
line, and bounded three-line context; it never prints either complete G-code
document.

## Current progress boundary

The active fixture identity, generator validation, normalized hash, and bounded
difference tests pass. Public slicing now loads the project, resolves typed
configuration, plans and intersects 460 layers, repairs and closes the sliced
geometry, simplifies it, composes dense Internal region surfaces, and removes
only the maximal suffix of post-region layers whose every region surface
vector is empty. It then applies conical-overhang projection from resolved 3MF
object and region Options, builds ordered layer islands, applies single-region
elephant-foot compensation, and retains ordered uncompensated `lslices` before
deliberately returning `ProjectSlicingIncomplete`.

For the committed KSR project, layer 459 is nonempty, so Task 22K removes zero
layers. Its `make_overhang_printable` region switch is false, so Task 22L also
leaves the body unchanged: the post-L state remains one object with 460 planned
and retained layers and one complete 460-layer occurrence sidecar. Task 22M
resolves 0.15 mm and one compensation layer from that 3MF, changes only the
first retained layer, and preserves the pre-compensation layer islands in
`lslices`. The exact M checkpoint is 3,008,346 bytes / SHA-256
`91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`.
The complete CLI golden remains explicitly ignored. Production does not read
or identify the reference G-code, and normalized `ksr_fdmtest_v4.gcode` parity
is not claimed.

## Task 5 fixed-source inventory

The inventory is derived only from fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its source boundaries are
`Config.hpp/cpp::ConfigOption*` and `ConfigBase::save_to_json`,
`PrintConfig.hpp/cpp` option ownership, registration, defaults, nullable and
legacy handling, `Preset.cpp` raw-scope lists, `Preset.hpp` JSON-key macros,
`GCode.cpp::append_full_config`, and the project-settings save call in
`Format/bbs_3mf.cpp`.

The committed `options-v242.json` contract contains 653 bytewise-sorted unique
rows: 448 scalar strings, 205 arrays, and five empty arrays. Raw scopes are 132
printer, 352 process, 122 filament, and 47 residual keys. Static ownership and
effective projection are recorded separately; the verified projections are
126 object, 153 region, and 149 G-code keys. Thirty-one options are nullable.

The concrete type histogram is:

| Type | Count | Type | Count |
| --- | ---: | --- | ---: |
| `coBool` | 105 | `coBools` | 22 |
| `coEnum` | 44 | `coEnums` | 9 |
| `coFloat` | 160 | `coFloatOrPercent` | 36 |
| `coFloats` | 90 | `coInt` | 41 |
| `coInts` | 45 | `coPercent` | 25 |
| `coPercents` | 5 | `coPoint` | 4 |
| `coPoints` | 6 | `coPointsGroups` | 1 |
| `coString` | 30 | `coStrings` | 27 |
| metadata | 3 |  |  |

Config-export disposition is 615 canonical, 31 omit-when-nil, three metadata
exclusions, and four source-derived special rules: scaled flush matrix,
extruder-colour substitution, and duplicate plate-coordinate output for the X
and Y wipe-tower keys. The export parser proves the fixed banned-key and nil
guard, independent wipe/substitution branches, read/write data flow, and
canonical fallback; fixture keys may not intersect the banned set.

## Typed codec contract

The public codecs directly deserialize Orca's string/array wire forms without
an erased intermediate value. They cover embedded `0`/`1` booleans plus the
existing explicit-STL native forms, signed and unsigned integer bounds, finite
floats with Orca lexical output, millimeters, whitespace-tolerant percentages,
float-or-percent unions, nullable `nil`, typed vectors and enums, scalar and
`x`-separated points, point groups, and the fixture's opaque matrix, stride,
AMS, ramming, CSV, and space-tuple forms. Empty strings remain distinct from
empty arrays, and invalid or non-finite values are rejected.

The group dispatcher reads a value only in the matching concrete key arm,
stores presence as `Option<T>`, resolves missing values through typed defaults,
rejects duplicate known keys, and leaves a nonmatching value available for the
next group. Task 6 adds `ProjectSettings::printer`, a flat `PrinterOptions`
dispatcher, and its completed `MachineEnvelopeOptions` child. Strict top-level
deserialization remains deferred until all project groups exist.

## Provenance and behavior boundary

Active tests consume only the committed 653-row semantic artifact and fixture.
Task 19A removed the checkout-dependent source-line/symbol reconstruction test,
its ignored Orca provenance gate, and the generator/Rust mutation probes. No
active test requires an OrcaSlicer checkout or treats mutable source text as a
runtime oracle.

The deterministic fixed-commit artifact generator remains and regenerates the
committed inventory byte-identically. The artifact and current semantic tests
retain the metadata partition, qualified enum/default, nullable-sentinel, and
axis properties. In particular, `InputShaperType::Default` serializes as
`Default` independently of `PrintOrder::Default` serializing as `default`;
`NozzleType::ntUndefine` is the real token `undefine`, while only the nullable
integer `INT_MAX` sentinel is `nil`; and the 12 axis rows preserve their typed
declarations, aggregate member order, and registration-to-default properties.
These are committed-artifact semantic assertions, not source-pinning tests.

Every non-metadata row currently cites the generic
`GCode::append_full_config` banned/nil guard, and the three metadata rows cite
the project-settings `save_to_json` call. These are truthful generic
retention/export consumers, not claims that 653 option-specific slicing
algorithms are implemented. Later tasks must replace or supplement them with
the actual fixed upstream behavioral consumer as each option becomes consumed.

## Option implementation ledger

Each entry records the key boundary, raw scope, concrete type, fixed upstream
owner or consumer, state (`retained-only` or `consumed`), focused test,
observable serialization effect, and deferred adjacent behavior. Merely
retaining a key or citing the generic export loop is not option parity.

### Task 6: printer machine envelope

All 28 entries have raw scope `printer`, static owner
`PrintConfig.hpp::MachineEnvelopeConfig`, and state `retained-only` in the new
typed project path:

- `OrcaBool`: `emit_machine_limits_to_gcode`, `resonance_avoidance`,
  `input_shaping_emit`.
- `OrcaFloat`: `min_resonance_avoidance_speed`,
  `max_resonance_avoidance_speed`, `input_shaping_freq_x`,
  `input_shaping_freq_y`, `input_shaping_damp_x`, and
  `input_shaping_damp_y`.
- `OrcaFloats`: the 12 individual XYZE speed, acceleration, and jerk fields;
  `machine_max_acceleration_extruding`,
  `machine_max_acceleration_retracting`,
  `machine_max_acceleration_travel`, `machine_max_junction_deviation`,
  `machine_min_travel_rate`, and `machine_min_extruding_rate`.
- `InputShaperType`: `input_shaping_type`, with the exact 13 fixed-tag tokens.

The focused test proves the exact inventory intersection, defaults, 3MF wire
shape, concrete semantic values, fixture cardinalities, declaration order,
lexicographic serialization, duplicate/unknown rejection, every input-shaper
token, and changed typed state. Raw vectors retain all fixture values; active
variant selection is not performed here. Existing legacy `SliceOptions` tests
for machine-limit, resonance, and input-shaping G-code do not make the new typed
project path `consumed`. Top-level project dispatch, effective config
composition, normalization, and G-code consumption remain deferred.

### Task 7: printer G-code source

The next 62 entries have raw scope `printer`, static owner
`PrintConfig.hpp::GCodeConfig`, and state `retained-only` in the typed project
path. Their concrete key/type ledger is:

- `coBool`: `auxiliary_fan`, `disable_m73`, `enable_filament_ramming`,
  `fan_speedup_overhangs`, `high_current_on_filament_swap`,
  `manual_filament_change`, `purge_in_prime_tower`, `scan_first_layer`,
  `silent_mode`, `single_extruder_multi_material`, `support_air_filtration`,
  `support_chamber_temp_control`, `support_multi_bed_types`,
  `support_object_skip_flush`, `tool_change_on_wipe_tower`, `use_3mf`,
  `use_firmware_retraction`, and `use_relative_e_distances`; `coBools`:
  `long_retractions_when_cut`.
- `coEnum`: `bed_temperature_formula`, `enable_power_loss_recovery`,
  `gcode_flavor`, `printer_structure`, and `wipe_tower_type`; `coEnums`:
  `extruder_type`, nullable `nozzle_type`, `retract_lift_enforce`, and
  `z_hop_types`.
- `coFloat`: `cooling_tube_length`, `cooling_tube_retraction`,
  `extra_loading_move`, `fan_kickstart`, `fan_speedup_time`,
  `machine_load_filament_time`, `machine_tool_change_time`,
  `machine_unload_filament_time`, `parking_pos_retraction`, and `time_cost`;
  `coFloats`: `retraction_distances_when_cut` and `travel_slope`.
- `coInt`: `enable_long_retraction_when_cut`, `master_extruder_id`,
  `nozzle_hrc`, `part_cooling_fan_min_pwm`, and
  `wrapping_detection_layers`; `coInts`: nullable
  `nozzle_flush_dataset`, `physical_extruder_map`, and
  `printer_extruder_id`.
- `coPoints`: `wrapping_exclude_area`; `coStrings`:
  `printer_extruder_variant`.
- `coString`: `before_layer_change_gcode`, `change_extrusion_role_gcode`,
  `change_filament_gcode`, `file_start_gcode`, `layer_change_gcode`,
  `machine_end_gcode`, `machine_pause_gcode`, `machine_start_gcode`,
  `printing_by_object_gcode`, `template_custom_gcode`, `time_lapse_gcode`,
  and `wrapping_detection_gcode`.

`PrinterOptions` dispatches the 28 machine-envelope and 62 G-code-source keys
through their private concrete builders without an erased remainder. The
focused tests prove the exact disjoint inventory intersection, fixed defaults,
all nine enum domains, element-level nullable vectors, vector cardinalities,
non-empty point parsing, five multiline G-code values, declaration order,
lexicographic serialization, and mixed child dispatch. The typed project path
still does not feed these fields into slicing; effective projection,
normalization, placeholder/template consumers, G-code generation, and config
export remain deferred.

### Task 8: remaining printer raw options

The final printer child contains exactly the 42 inventory rows where
`raw_scope=printer` and `static_owner` is `print_config` (27) or `unowned`
(15). All remain `retained-only`. Together with Tasks 6 and 7,
`PrinterOptions` now owns the complete disjoint `28 + 62 + 42 = 132` printer
raw-key set without a dynamic remainder. The remaining key/type ledger is:

- `coBool`: `bbl_use_printhost`, `pellet_modded_printer`,
  `printhost_ssl_ignore_revoke`, and `support_parallel_printheads`.
- `coEnum`: `host_type`, `printer_technology`,
  `printhost_authorization_type`, and `thumbnails_format`; `coEnums`:
  `default_nozzle_volume_type`.
- `coFloat`: `adaptive_bed_mesh_margin`,
  `extruder_clearance_height_to_lid`, `extruder_clearance_height_to_rod`,
  `extruder_clearance_radius`, `nozzle_height`, `preferred_orientation`,
  `printable_height`, and `z_offset`; `coFloats`: nullable
  `extruder_printable_height`, `grab_length`, and nullable `nozzle_volume`.
- `coInt`: `parallel_printheads_count`.
- `coPoint`: `bed_mesh_max`, `bed_mesh_min`, `bed_mesh_probe_distance`, and
  `best_object_pos`; `coPoints`: `bed_exclude_area`,
  `head_wrap_detect_zone`, and `printable_area`; `coPointsGroups`:
  `extruder_printable_area`.
- `coString`: `bed_custom_model`, `bed_custom_texture`, `default_bed_type`,
  `default_print_profile`, `flashforge_serial_number`, `printer_agent`,
  `printer_model`, `printer_notes`, `printer_variant`, and `thumbnails`;
  `coStrings`: `extruder_variant_list`,
  `parallel_printheads_bed_exclude_areas`, and
  `upward_compatible_machine`.

The 27 `PrintConfig.hpp::PrintConfig` declarations and 15 fixed
`PrintConfig.cpp` registrations retain separate provenance orders, while the
remaining child serializes 42 keys and the parent serializes all 132 keys in
global lexicographic order. Explicit empty area arrays remain distinct from
missing fields and upstream defaults. `extruder_variant_list` and `thumbnails`
use semantic raw-preserving wrappers: Task 8 does not normalize variant tokens
or expose the existing stricter thumbnail parser as fixed-source behavior.
Exact variant expansion and thumbnail composite parsing remain deferred.
`extruder_ams_count` is a residual key, is rejected by `PrinterOptions`, and
remains owned by Task 14.

The focused tests prove exact ownership and type histograms, the disjoint
132-key union, fixed defaults and all 18 fixture/default differences, complete
enum domains, element-nullable float vectors, point/list/group wire shapes,
fixture cardinalities, raw structured strings, mixed flat dispatch, duplicate
and cross-scope rejection, exact 42-field byte round-trip, and exact flat
132-field byte round-trip with streaming key-order observation. The typed
project path still does not compose effective configs or consume these values
in slicing or G-code generation.

### Task 9: process object source

The first process child is the exact active intersection of process raw keys
with `PrintConfig.hpp::PrintObjectConfig` at fixed lines 917-1071. It contains
126 unique scalar-string fields and excludes the commented tuple-shaped
`independent_support_layer_height` and `adaptive_layer_height` lines. All 126
entries remain `retained-only`; `ProcessOptions` has one flat `object` child and
`ProjectSettings` exposes that typed process boundary without implementing a
partial whole-project deserializer. The concrete key/type ledger is:

- `coBool` (22): `bridge_no_support`, `brim_use_efc_outline`,
  `calib_flowrate_topinfill_special_order`,
  `detect_narrow_internal_solid_infill`, `enable_support`,
  `flush_into_infill`, `flush_into_objects`, `flush_into_support`,
  `interface_shells`, `interlocking_beam`, `precise_z_height`,
  `set_other_flow_ratios`, `staggered_inner_seams`,
  `support_critical_regions_only`, `support_interface_loop_pattern`,
  `support_interface_not_for_body`, `support_ironing`,
  `support_on_build_plate_only`, `support_remove_small_overhang`,
  `thick_bridges`, `thick_internal_bridges`, and
  `tree_support_auto_brim`.
- `coEnum` (12): `brim_type`, `dont_filter_internal_bridges`,
  `enable_extra_bridge_layer`, `gap_fill_target`, `seam_position`,
  `slicing_mode`, `support_base_pattern`, `support_interface_pattern`,
  `support_ironing_pattern`, `support_style`, `support_type`, and
  `wall_generator`.
- `coFloat` (63): `brim_ears_detection_length`, `brim_ears_max_angle`,
  `brim_flow_ratio`, `brim_object_gap`, `brim_width`,
  `default_acceleration`, `default_jerk`, `default_junction_deviation`,
  `elefant_foot_compensation`, `infill_jerk`,
  `initial_layer_acceleration`, `initial_layer_jerk`,
  `inner_wall_acceleration`, `inner_wall_jerk`, `interlocking_beam_width`,
  `interlocking_orientation`, `layer_height`,
  `make_overhang_printable_angle`, `make_overhang_printable_hole_size`,
  `max_bridge_length`, `min_length_factor`,
  `mmu_segmented_region_interlocking_depth`,
  `mmu_segmented_region_max_width`, `outer_wall_acceleration`,
  `outer_wall_jerk`, `raft_contact_distance`, `raft_expansion`,
  `raft_first_layer_expansion`, `skirt_start_angle`, `slice_closing_radius`,
  `support_angle`, `support_base_pattern_spacing`,
  `support_bottom_interface_spacing`, `support_bottom_z_distance`,
  `support_expansion`, `support_flow_ratio`,
  `support_interface_flow_ratio`, `support_interface_spacing`,
  `support_interface_speed`, `support_ironing_spacing`,
  `support_object_first_layer_gap`, `support_object_xy_distance`,
  `support_speed`, `support_top_z_distance`, `top_surface_acceleration`,
  `top_surface_jerk`, `travel_acceleration`, `travel_jerk`,
  `tree_support_angle_slow`, `tree_support_branch_angle`,
  `tree_support_branch_angle_organic`, `tree_support_branch_diameter`,
  `tree_support_branch_diameter_angle`,
  `tree_support_branch_diameter_organic`, `tree_support_branch_distance`,
  `tree_support_branch_distance_organic`, `tree_support_brim_width`,
  `tree_support_tip_diameter`, `wall_maximum_deviation`,
  `wall_maximum_resolution`, `wall_transition_angle`,
  `xy_contour_compensation`, and `xy_hole_compensation`.
- `coFloatOrPercent` (6): `bridge_acceleration`,
  `internal_solid_infill_acceleration`, `line_width`,
  `sparse_infill_acceleration`, `support_line_width`, and
  `support_threshold_overlap`.
- `coInt` (13): `elefant_foot_compensation_layers`,
  `enforce_support_layers`, `interlocking_beam_layer_count`,
  `interlocking_boundary_avoidance`, `interlocking_depth`, `raft_layers`,
  `support_filament`, `support_interface_bottom_layers`,
  `support_interface_filament`, `support_interface_top_layers`,
  `support_threshold_angle`, `tree_support_wall_count`, and
  `wall_distribution_count`.
- `coPercent` (10): `elefant_foot_layers_density`,
  `initial_layer_min_bead_width`, `internal_bridge_density`,
  `min_bead_width`, `min_feature_size`, `raft_first_layer_density`,
  `support_ironing_flow`, `tree_support_top_rate`,
  `wall_transition_filter_deviation`, and `wall_transition_length`.

The raw enum boundary uses all fixed canonical maps. In particular,
`support_ironing_pattern` accepts the complete 28-token global
`InfillPattern` map; its two UI choices are not the deserialization domain.
Legacy aliases remain deferred to their separate normalization boundary.
Production declaration order is asserted positionally against the 126 active
HPP declarations, while direct serialization uses the independent bytewise
lexicographic order. The shared typed group decoder now attaches the literal
Option key to value-decoding errors without changing duplicate or unknown-key
behavior.

A literal-consumer scan records 108 of these 126 names in the existing dynamic
`SliceOptions` behavior pipeline. The exact 18 without a current literal
consumer are `flush_into_infill`, `flush_into_objects`, `flush_into_support`,
`interface_shells`, `interlocking_beam`, `interlocking_beam_layer_count`,
`interlocking_beam_width`, `interlocking_boundary_avoidance`,
`interlocking_depth`, `interlocking_orientation`, `max_bridge_length`,
`mmu_segmented_region_interlocking_depth`,
`mmu_segmented_region_max_width`, `raft_contact_distance`,
`slice_closing_radius`, `slicing_mode`, `xy_contour_compensation`, and
`xy_hole_compensation`; the complementary 108 names are the recorded
collisions. Task 9 does not migrate any of them. Effective object projection
and ordered sparse override resolution are now implemented by Task 15. Typed
consumer migration and removal of the dynamic compatibility path remain owned
by Tasks 20A-20E.

Focused tests prove the exact 126-key/type/wire intersection, all defaults and
18 fixture overrides, concrete field types, declaration and export orders,
the complete enum domains, both float-or-percent branches, flat parent serde,
strict cross-owner rejection, and exact fixture byte round-trip. A
single-field non-default typed-state test covers every one of the 126 dispatch
arms so the 108 fixture values equal to defaults cannot hide a dropped field.

### Task 10: process region source

The second process child is the exact active intersection of process raw keys
with fixed OrcaSlicer v2.4.2 `PrintConfig.hpp::PrintRegionConfig` declarations
and `PrintConfig.cpp` defaults and enum maps. Of the 155 active HPP tuples, it
owns 149. The four filament-scope nullable overrides
`filament_ironing_flow`, `filament_ironing_spacing`,
`filament_ironing_inset`, and `filament_ironing_speed`, plus the two
legacy-only shells `ironing_direction` and `wall_infill_order`, are excluded.
Task 10 introduced these 149 fields as retained raw source state. Task 16 now
reuses the same compile-time inventory for concrete effective region
projection, while dynamic consumer migration remains in Tasks 20A-20D. The
concrete key/type ledger is:

- `coBool` (31): `align_infill_direction_to_model`, `alternate_extra_wall`,
  `detect_overhang_wall`, `detect_thin_wall`, `enable_overhang_speed`,
  `extra_perimeters_on_overhangs`, `fuzzy_skin_first_layer`,
  `gyroid_optimized`, `hole_to_polyhole`, `hole_to_polyhole_twisted`,
  `infill_combination`, `ironing_angle_fixed`, `is_infill_first`,
  `make_overhang_printable`, `only_one_wall_first_layer`,
  `only_one_wall_top`, `overhang_reverse`, `overhang_reverse_internal_only`,
  `precise_outer_wall`, `relative_bridge_angle`, `role_based_wipe_speed`,
  `seam_slope_conditional`, `seam_slope_entire_loop`,
  `seam_slope_inner_walls`, `slowdown_for_curled_perimeters`,
  `small_area_infill_flow_compensation`, `symmetric_infill_y_axis`,
  `wipe_before_external_loop`, `wipe_on_loops`,
  `zaa_dont_alternate_fill_direction`, and `zaa_enabled`.
- `coEnum` (14): `bottom_surface_pattern`, `counterbore_hole_bridging`,
  `ensure_vertical_shell_thickness`, `fuzzy_skin`, `fuzzy_skin_mode`,
  `fuzzy_skin_noise_type`, `internal_solid_infill_pattern`,
  `ironing_pattern`, `ironing_type`, `seam_slope_type`,
  `sparse_infill_pattern`, `top_surface_pattern`, `wall_direction`, and
  `wall_sequence`.
- `coFloat` (49): `bottom_shell_thickness`,
  `bottom_solid_infill_flow_ratio`, `bridge_angle`, `bridge_flow`,
  `bridge_speed`, `filter_out_gap_fill`, `first_layer_flow_ratio`,
  `fuzzy_skin_persistence`, `fuzzy_skin_point_distance`, `fuzzy_skin_scale`,
  `fuzzy_skin_thickness`, `gap_fill_flow_ratio`, `gap_infill_speed`,
  `infill_direction`, `infill_lock_depth`, `infill_overhang_angle`,
  `infill_shift_step`, `inner_wall_flow_ratio`, `inner_wall_speed`,
  `internal_bridge_angle`, `internal_bridge_flow`,
  `internal_solid_infill_flow_ratio`, `internal_solid_infill_speed`,
  `ironing_angle`, `ironing_inset`, `ironing_spacing`, `ironing_speed`,
  `lateral_lattice_angle_1`, `lateral_lattice_angle_2`,
  `lightning_overhang_angle`, `lightning_prune_angle`,
  `lightning_straightening_angle`, `minimum_sparse_infill_area`,
  `outer_wall_flow_ratio`, `outer_wall_speed`, `overhang_flow_ratio`,
  `print_flow_ratio`, `scarf_joint_flow_ratio`, `seam_slope_min_length`,
  `skin_infill_depth`, `small_perimeter_threshold`,
  `solid_infill_direction`, `sparse_infill_flow_ratio`,
  `sparse_infill_speed`, `top_shell_thickness`,
  `top_solid_infill_flow_ratio`, `top_surface_speed`, `zaa_min_z`, and
  `zaa_minimize_perimeter_height`.
- `coFloatOrPercent` (24): `bridge_line_width`,
  `hole_to_polyhole_threshold`, `infill_anchor`, `infill_anchor_max`,
  `infill_combination_max_layer_height`, `inner_wall_line_width`,
  `internal_bridge_speed`, `internal_solid_infill_line_width`,
  `min_width_top_surface`, `outer_wall_line_width`, `overhang_1_4_speed`,
  `overhang_2_4_speed`, `overhang_3_4_speed`, `overhang_4_4_speed`,
  `overhang_reverse_threshold`, `scarf_joint_speed`, `seam_gap`,
  `seam_slope_start_height`, `skeleton_infill_line_width`,
  `skin_infill_line_width`, `small_perimeter_speed`,
  `sparse_infill_line_width`, `top_surface_line_width`, and `wipe_speed`.
- `coInt` (15): `bottom_shell_layers`, `bottom_surface_filament_id`,
  `fill_multiline`, `fuzzy_skin_layers_between_ripple_offset`,
  `fuzzy_skin_octaves`, `fuzzy_skin_ripples_per_layer`,
  `inner_wall_filament_id`, `internal_solid_filament_id`,
  `outer_wall_filament_id`, `scarf_angle_threshold`, `seam_slope_steps`,
  `sparse_infill_filament_id`, `top_shell_layers`,
  `top_surface_filament_id`, and `wall_loops`.
- `coInts` (1): `print_extruder_id`.
- `coPercent` (11): `bottom_surface_density`, `bridge_density`,
  `fuzzy_skin_ripple_offset`, `infill_wall_overlap`, `ironing_flow`,
  `scarf_overhang_threshold`, `skeleton_infill_density`,
  `skin_infill_density`, `sparse_infill_density`,
  `top_bottom_infill_wall_overlap`, and `top_surface_density`.
- `coString` (3): `extra_solid_infills`,
  `solid_infill_rotate_template`, and `sparse_infill_rotate_template`.
- `coStrings` (1): `print_extruder_variant`.

The wire boundary is 147 scalar strings plus the non-nullable integer and
string vectors `print_extruder_id` and `print_extruder_variant`. Their fixture
length is four, but typed parsing and serialization preserve any valid length;
there is no active-extruder cardinality rule in this raw layer. The three raw
string fields are preserved without template interpretation.

All five pattern fields use the complete fixed 28-token
`ProcessInfillPattern` map. The other nine enum domains are dedicated raw
types for vertical-shell enforcement, fuzzy-skin type, fuzzy noise, fuzzy
mode, ironing type, counterbore bridging, wall sequence, wall direction, and
seam scarf type. Only Orca's machine-readable tokens are accepted. UI labels,
aliases, and `handle_legacy` conversions remain Task 19A.

`ProcessOptions` now directly dispatches both children from one flat input
map. Each child serializes in its independent lexical order, and the parent
streams the disjoint 126 + 149 union as one globally lexical 275-key map. It
does not delegate nested child maps, use serde flattening, or allocate a DOM.

A literal-consumer scan records 109 of these 149 names in the existing dynamic
`SliceOptions` behavior pipeline. The exact 40 without a current literal
consumer are `align_infill_direction_to_model`,
`bottom_surface_filament_id`, `bridge_line_width`, `fuzzy_skin_mode`,
`gyroid_optimized`, `hole_to_polyhole`, `hole_to_polyhole_threshold`,
`hole_to_polyhole_twisted`, `infill_lock_depth`, `infill_overhang_angle`,
`inner_wall_filament_id`, `internal_solid_filament_id`,
`lateral_lattice_angle_1`, `lateral_lattice_angle_2`,
`lightning_overhang_angle`, `lightning_prune_angle`,
`lightning_straightening_angle`, `outer_wall_filament_id`,
`print_extruder_id`, `relative_bridge_angle`, `scarf_angle_threshold`,
`scarf_joint_flow_ratio`, `scarf_joint_speed`, `scarf_overhang_threshold`,
`seam_slope_conditional`, `seam_slope_entire_loop`,
`seam_slope_inner_walls`, `seam_slope_min_length`,
`seam_slope_start_height`, `seam_slope_steps`, `seam_slope_type`,
`skeleton_infill_density`, `skin_infill_density`, `skin_infill_depth`,
`sparse_infill_filament_id`, `top_surface_filament_id`,
`zaa_dont_alternate_fill_direction`, `zaa_enabled`, `zaa_min_z`, and
`zaa_minimize_perimeter_height`; the complementary 109 names are the recorded
collisions and are not migrated by Task 10.

Focused tests prove exact ownership and the type histogram, fixed declaration
order, exact defaults and all 30 fixture overrides, concrete field types, all
enum domains, both float-or-percent categories, arbitrary valid vector
lengths, direct non-default dispatch for all 149 fields, strict owner and shape
rejection, child 149-key bytes, and flat parent 275-key bytes and order.
Reviewed verification passes 24 focused object/region tests, all 4,246
workspace tests with three configured skips, the 22-test dynamic-value audit
with one configured skip, warning-denying workspace all-target Clippy,
rustfmt, both `ares-core` and `ares-wasm` WASM checks, and the diff whitespace
gate. Two independent frozen-byte code reviews, an independent documentation
review, and the primary-agent review approve this boundary under the
user-approved temporary OpenCode bypass.

### Task 11: remaining process raw source

The final process raw slice is the exact fixed-source intersection of the
fixture's Process keys with direct `GCodeConfig` and FFF `PrintConfig`
ownership. `GCodeConfig` is bounded by fixed
`PrintConfig.hpp:1299-1476`; direct FFF `PrintConfig` ownership is bounded by
`PrintConfig.hpp:1479-1660`, excluding the unrelated SLA
`filename_format` declaration. The one definition without an active static
owner is `ironing_expansion` from fixed `PrintConfig.cpp:4368`.

The exact ownership and type ledger is:

- `GCodeConfig` (17):
  - `coBool` (7): `accel_to_decel_enable`, `enable_arc_fitting`,
    `enable_wrapping_detection`,
    `extrusion_rate_smoothing_external_perimeter_only`,
    `gcode_add_line_number`, `single_extruder_multi_material_priming`, and
    `wipe_tower_no_sparse_layers`.
  - `coFloat` (4): `max_volumetric_extrusion_rate_slope`,
    `max_volumetric_extrusion_rate_slope_segment_length`, `travel_speed`,
    and `travel_speed_z`.
  - `coFloatOrPercent` (3): `initial_layer_travel_acceleration`,
    `initial_layer_travel_jerk`, and `initial_layer_travel_speed`.
  - `coPercent` (1): `accel_to_decel_factor`.
  - `coString` (1): `process_change_extrusion_role_gcode`.
  - `coStrings` (1): `small_area_infill_flow_compensation_model`.
- direct FFF `PrintConfig` (59):
  - `coBool` (18): `combine_brims`, `enable_prime_tower`,
    `enable_tower_interface_cooldown_during_tower`,
    `enable_tower_interface_features`, `exclude_object`, `gcode_comments`,
    `gcode_label_objects`, `independent_support_layer_height`,
    `ooze_prevention`, `prime_tower_enable_framework`,
    `prime_tower_flat_ironing`, `prime_tower_skip_points`,
    `reduce_crossing_wall`, `reduce_infill_retraction`,
    `single_loop_draft_shield`, `spiral_mode`, `spiral_mode_smooth`, and
    `wipe_tower_fillet_wall`.
  - `coEnum` (6): `draft_shield`, `print_order`, `print_sequence`,
    `skirt_type`, `timelapse_type`, and `wipe_tower_wall_type`.
  - `coFloat` (19): `initial_layer_infill_speed`,
    `initial_layer_print_height`, `initial_layer_speed`, `min_skirt_length`,
    `preheat_time`, `prime_tower_brim_width`, `prime_tower_width`,
    `prime_volume`, `resolution`, `skirt_distance`, `skirt_speed`,
    `spiral_finishing_flow_ratio`, `spiral_starting_flow_ratio`,
    `wipe_tower_bridging`, `wipe_tower_cone_angle`,
    `wipe_tower_extra_rib_length`, `wipe_tower_max_purge_speed`,
    `wipe_tower_rib_width`, and `wipe_tower_rotation_angle`.
  - `coFloatOrPercent` (3): `initial_layer_line_width`,
    `max_travel_detour_distance`, and `spiral_mode_max_xy_smoothing`.
  - `coFloats` (1): `wiping_volumes_extruders`.
  - `coInt` (6): `preheat_steps`, `skirt_height`, `skirt_loops`,
    `slow_down_layers`, `standby_temperature_delta`, and
    `wipe_tower_filament`.
  - `coPercent` (3): `prime_tower_infill_gap`, `wipe_tower_extra_flow`, and
    `wipe_tower_extra_spacing`.
  - `coString` (2): `filename_format` and `notes`.
  - `coStrings` (1): `post_process`.
- unowned (1):
  - `coFloat` (1): `ironing_expansion`.

The resulting histogram is exactly 25 bool, six enum, 24 float, six
float-or-percent, one float-vector, six int, four percent, three string, and
two string-vector fields. All 77 fields are non-nullable. The wire boundary
contains 74 scalar strings and exactly three arrays: `post_process`,
`small_area_infill_flow_compensation_model`, and
`wiping_volumes_extruders`. Raw parsing preserves arbitrary valid vector
lengths, including empty arrays; it does not infer a matrix or encode the
fixture's ten-element defaults. The leading newlines in
`small_area_infill_flow_compensation_model` remain individual string content.
These definitions are fixed at `PrintConfig.cpp:4479-4491`,
`PrintConfig.cpp:5068-5079`, and `PrintConfig.cpp:6976-6981`.

The six strict raw enum domains are:

| Option | Canonical machine tokens |
|---|---|
| `draft_shield` | `disabled`, `enabled` |
| `print_order` | `default`, `as_obj_list` |
| `print_sequence` | `by layer`, `by object` |
| `skirt_type` | `combined`, `perobject` |
| `timelapse_type` | `0`, `1` |
| `wipe_tower_wall_type` | `rectangle`, `cone`, `rib` |

The maps are fixed by `PrintConfig.cpp:295-305`,
`PrintConfig.cpp:432-449`, and `PrintConfig.cpp:560-565`, with their option
definitions at `PrintConfig.cpp:1836-1856`,
`PrintConfig.cpp:5706-5731`, `PrintConfig.cpp:5879-5894`, and
`PrintConfig.cpp:6925-6939`. UI labels, enum sentinels, and legacy spellings
are not raw enum tokens. `prime_tower_brim_width` remains a float: its
open-enum UI at fixed `PrintConfig.cpp:6891-6900` gives `-1` the meaning Auto
but does not turn its machine type into a closed enum.

Exactly 15 fixture values differ from fixed defaults:
`enable_arc_fitting`, `enable_prime_tower`,
`enable_tower_interface_features`, `filename_format`,
`initial_layer_infill_speed`, `initial_layer_line_width`,
`initial_layer_speed`, `initial_layer_travel_acceleration`,
`prime_tower_brim_width`, `prime_tower_flat_ironing`, `prime_tower_width`,
`reduce_infill_retraction`, `resolution`, `skirt_loops`, and `travel_speed`.
The other 62 fixture values equal their fixed defaults, so every field also
has an independent valid non-default typed-state proof.

The inventory records 13 canonical targets with fixed
`PrintConfigDef::handle_legacy` inputs: `draft_shield`,
`enable_prime_tower`, `initial_layer_print_height`, `initial_layer_speed`,
`prime_tower_brim_width`, `prime_tower_width`, `prime_volume`,
`timelapse_type`, `wipe_tower_extra_rib_length`, `wipe_tower_filament`,
`wipe_tower_fillet_wall`, `wipe_tower_rib_width`, and
`wipe_tower_wall_type`. Canonical Task 11 dispatch does not accept those
aliases or value conversions; the source-cited conversion boundary remains
Task 19A.

`ProcessOptions` now owns public `gcode` and `print` children and the direct
unowned `ironing_expansion` scalar. Together with the 126 object-source and
149 region-source fields, this forms exactly 352 disjoint Process fields.
Each child preserves its fixed HPP declaration order and standalone lexical
serialization. The parent directly streams one globally lexicographic
352-entry map without nested child maps, serde flattening, or a DOM. Relative
to the previous 275-entry parent, the new lexical entries distribute 23/32/22
across `early`/`middle`/`late`, producing helper totals of 115/124/113.

A production literal scan finds 63 of the 77 names in the existing dynamic
compatibility implementation. The exact 14-key complement is
`enable_arc_fitting`, `enable_tower_interface_cooldown_during_tower`,
`enable_tower_interface_features`, `filename_format`, `ironing_expansion`,
`max_travel_detour_distance`, `post_process`,
`prime_tower_enable_framework`, `prime_tower_flat_ironing`,
`prime_tower_infill_gap`, `prime_tower_skip_points`, `print_order`,
`reduce_crossing_wall`, and `wiping_volumes_extruders`. `prime_volume`
appears only in the legacy compatibility parser, so the behavioral-consumer
union is 62 names. Task 11 records but does not migrate these users.

The 17 G-code-owned raw fields project into effective `GCodeOptions` in
Task 17. Legacy canonicalization remains Task 19A, full-print resolution and
FDM normalization remain Task 19B, existing behavioral consumers migrate
across Tasks 20A-20D, and the final legacy compatibility parser is removed
only in Task 20E.

### Task 12: filament G-code source

The first filament raw child combines the fixed-source intersection of 52
live filament preset names with `GCodeConfig` plus the separately
project-owned `filament_colour` at OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The live preset boundary is
`Preset.cpp:1309-1346`; project ownership is explicit at
`PresetBundle.cpp:43-58,2652-2658,2795-2802`; and `GCodeConfig` is bounded by
`PrintConfig.hpp:1299-1476`. `filament_colour` is commented out at
`Preset.cpp:1309` but remains declared at `PrintConfig.hpp:1333` and defined
at `PrintConfig.cpp:2455`. The resulting boundary contains the 53 declarations
at `PrintConfig.hpp:1308-1464`; their definitions and singleton defaults are
in `PrintConfig.cpp:2046-2401,2447-2925,5229-5425,5949,6700`. Generic vector,
nullable-element, JSON-array load, and 3MF JSON-array emission behavior comes
from fixed `Config.hpp:624-663,812-952,995-1085,1118-1163,1857-1967` and
`Config.cpp:830-870,950-1004,1464-1496`.

All 53 top-level wire values are arrays and there are no enums in this slice.
The exact type ledger is:

- `coBools` (8): `adaptive_pressure_advance`,
  `adaptive_pressure_advance_overhangs`, `enable_pressure_advance`,
  `filament_adaptive_volumetric_speed`, `filament_is_support`,
  `filament_multitool_ramming`, `filament_soluble`, and
  `long_retractions_when_ec`.
- `coFloats` (27): `adaptive_pressure_advance_bridges`,
  `filament_change_length`, `filament_cooling_before_tower`,
  `filament_cooling_final_speed`, `filament_cooling_initial_speed`,
  `filament_cost`, `filament_density`, `filament_diameter`,
  `filament_flow_ratio`, `filament_flush_volumetric_speed`,
  `filament_loading_speed`, `filament_loading_speed_start`,
  `filament_max_volumetric_speed`,
  `filament_minimal_purge_on_wipe_tower`, `filament_multitool_ramming_flow`,
  `filament_multitool_ramming_volume`, `filament_stamping_distance`,
  `filament_stamping_loading_speed`, `filament_toolchange_delay`,
  `filament_tower_interface_pre_extrusion_dist`,
  `filament_tower_interface_pre_extrusion_length`,
  `filament_tower_interface_purge_volume`, `filament_tower_ironing_area`,
  `filament_unloading_speed`, `filament_unloading_speed_start`,
  `pressure_advance`, and `retraction_distances_when_ec`.
- `coInts` (7): `filament_adhesiveness_category`,
  `filament_cooling_moves`, `filament_flush_temp`, `filament_printable`,
  `filament_tower_interface_print_temp`, `required_nozzle_HRC`, and
  `temperature_vitrification`.
- `coStrings` (11): `adaptive_pressure_advance_model`,
  `default_filament_colour`, `filament_change_extrusion_role_gcode`,
  `filament_colour`, `filament_end_gcode`, `filament_extruder_variant`,
  `filament_ramming_parameters`, `filament_start_gcode`, `filament_type`,
  `filament_vendor`, and `volumetric_speed_coefficients`.

The exact seven element-nullable arrays are
`filament_adaptive_volumetric_speed` and `long_retractions_when_ec` as direct
`Vec<Nullable<OrcaBool>>`, `filament_flow_ratio`,
`filament_flush_volumetric_speed`, `filament_cooling_before_tower`, and
`retraction_distances_when_ec` as direct `Vec<Nullable<OrcaFloat>>`, and
`filament_flush_temp` as direct `Vec<Nullable<OrcaInt>>`. They preserve an
exact `"nil"` element. The other numeric and boolean arrays reject `"nil"`,
while string arrays retain it as ordinary string content. No printer-owned
nullable-vector wrapper or filament-only nullable wrapper is introduced.

Every fixed default is a singleton vector, including the single-space
`filament_start_gcode` and `filament_end_gcode` defaults, the embedded-newline
`adaptive_pressure_advance_model` default, and the exact raw ramming and
space-tuple strings. The implementation uses the raw semantic wrappers
`CsvTable` for `adaptive_pressure_advance_model`, `VariantStride` for
`filament_extruder_variant`, `RammingParameters` for
`filament_ramming_parameters`, and `SpaceTuple` for
`volumetric_speed_coefficients`; these wrappers preserve strings without
interpreting their contents. `filament_type` remains an open suggestion
string, `filament_extruder_variant` remains a raw string vector, and
`filament_printable` remains an integer bitmask.

The fixture contains exactly 43 vectors of length two and ten vectors of
length eight. The ten are `filament_adaptive_volumetric_speed`,
`filament_cooling_before_tower`, `filament_extruder_variant`,
`filament_flow_ratio`, `filament_flush_temp`,
`filament_flush_volumetric_speed`, `filament_max_volumetric_speed`,
`long_retractions_when_ec`, `retraction_distances_when_ec`, and
`volumetric_speed_coefficients`, matching the fixed variant-stride set at
`PrintConfig.cpp:8375-8415`. All 53 fixture values differ from singleton
defaults by cardinality. After cardinality is ignored, exactly 17 still
differ: `filament_adhesiveness_category`, `filament_change_length`,
`filament_colour`, `filament_cost`, `filament_density`,
`filament_end_gcode`, `filament_extruder_variant`, `filament_flow_ratio`,
`filament_max_volumetric_speed`, `filament_start_gcode`,
`filament_tower_interface_print_temp`, `filament_vendor`,
`long_retractions_when_ec`, `required_nozzle_HRC`,
`retraction_distances_when_ec`, `temperature_vitrification`, and
`volumetric_speed_coefficients`. Raw parsing and serialization preserve any
valid vector length, including empty vectors; this layer neither imposes the
fixture cardinality nor collapses eight-value vectors to active values.

`FilamentGCodeSourceOptions` preserves the fixed filtered HPP declaration
order in production and serializes independently in bytewise lexical key
order. `FilamentOptions` owns public `gcode: FilamentGCodeSourceOptions` and
directly streams the same flat lexical 53-key map without a nested `gcode`
object, serde flattening, or DOM buffering. `ProjectSettings` adds public
`filament: FilamentOptions`, whose aggregate default is the filament default.

A production-literal audit, excluding registry/tests and these typed Task 12
declarations, records 51 of the 53 names as existing compatibility-consumer
collisions. The exact two-key complement is
`adaptive_pressure_advance_model` and
`adaptive_pressure_advance_overhangs`. This task records that boundary but
does not migrate a consumer. Fixed `PrintConfig.cpp:8153-8154,8219` legacy
conversions for `Normal`/`Big Traffic` and `ASA-Aero` remain Task 19A work.
The `[0,4]` active selection, variant resizing, cross-field cardinality, and
full FDM normalization remain Task 19B work at fixed
`PrintConfig.cpp:9004-9054,9805-10023`, `PrintApply.cpp:1164-1173`, and
`Print.cpp:3166-3175`. The seven nullable `omit_when_nil` export rules remain
Task 19C work. Existing option/profile and G-code consumer migrations remain
Tasks 20A and 20D respectively, and the compatibility parser is removed only
in Task 20E.

TDD first produced the expected missing-interface compiler failures for
`FilamentGCodeSourceOptions`, `FilamentOptions`, and
`ProjectSettings::filament`. The completed focused matrix has 14 tests proving
the exact inventory and histogram, fixed declaration order, singleton
defaults, fixture cardinality and bytes, the exact 17 payload overrides,
concrete field types, every-field child and flat-parent non-default dispatch,
all seven nullable vectors, arbitrary valid lengths, keyed invalid shapes,
duplicate and unknown keys, lexical wire order, raw structured strings, and
the aggregate boundary.

Reviewed local verification passes all 14 focused tests, 62 adjacent typed
printer/process tests, all 4,274 workspace tests with three configured skips,
and the 22-test dynamic-value audit with one configured skip. Warning-denying
`ares-core` all-target Clippy, rustfmt, native `ares-core`, both `ares-core`
and `ares-wasm` WASM checks, tracked and untracked whitespace checks, and the
physical-LOC gate are green; the largest changed Rust module is 280 lines.
Independent fixed-source, TDD-plan, wrapper, inventory, frozen-byte quality,
and final specification reviews approve the slice under the user-approved
temporary OpenCode bypass.

### Task 13: remaining filament raw sources

The remaining filament raw slice is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its exact 69-key partition is 48
FFF `PrintConfig` declarations selected from `PrintConfig.hpp:1484-1650`, four
filament ironing fields declared by `PrintRegionConfig` at
`PrintConfig.hpp:1153-1156` and defined nullable at
`PrintConfig.cpp:3492-3538`, the 16-entry
`filament_extruder_override_keys` list plus `add_nullable` construction loop at
`PrintConfig.cpp:63-84,7287-7318`, and direct `pellet_flow_coefficient` at
`PrintConfig.cpp:2639-2643`. The fixed live filament list is
`Preset.cpp:1309-1346`; nil retract overrides are preserved by preset
serialization at `Preset.cpp:1861-1878`. Generic raw array, nullable-element,
JSON-array load, and 3MF JSON-array emission behavior remains the fixed
`Config.hpp:624-663,812-952,995-1085,1118-1163,1857-1967` and
`Config.cpp:830-870,950-1004,1464-1496` boundary.

All 69 top-level values are arrays. Their exact histogram is 11 `coBools`,
three `coEnums`, 20 `coFloats`, 30 `coInts`, four `coPercents`, and one
`coStrings`. The subgroup histograms are Print `8/1/6/30/2/1`, Region
`0/0/3/0/1/0`, retract overrides `3/2/10/0/1/0`, and pellet one float vector.
The 48-, four-, and 16-field children preserve fixed HPP/list order internally
and serialize independently in bytewise lexical order.

Exactly 20 Task 13 fields have nullable elements. The four region fields are
`filament_ironing_flow`, `filament_ironing_spacing`,
`filament_ironing_inset`, and `filament_ironing_speed`. The 16 generated fields
are `filament_retraction_length`, `filament_z_hop`,
`filament_z_hop_types`, `filament_retract_lift_above`,
`filament_retract_lift_below`, `filament_retract_lift_enforce`,
`filament_retraction_speed`, `filament_deretraction_speed`,
`filament_retract_restart_extra`, `filament_retraction_minimum_travel`,
`filament_wipe_distance`, `filament_retract_when_changing_layer`,
`filament_wipe`, `filament_retract_before_wipe`,
`filament_long_retractions_when_cut`, and
`filament_retraction_distances_when_cut`. Region defaults are singleton nil;
the generated defaults clone their corresponding concrete extruder singleton
at `PrintConfig.cpp:7311-7315`. The fixture is all-nil for the four region
fields and 11 generated fields; its five nullable fields with concrete payloads
are `filament_retraction_distances_when_cut`,
`filament_retraction_length`, `filament_wipe`, `filament_wipe_distance`, and
`filament_z_hop_types`. Together with Task 12's seven, `FilamentOptions` owns
exactly 27 nullable fields. `ProjectSettings` reaches 31 only after adding the
four printer nullable fields; that project count is not a filament count.

Every Task 13 source default is a singleton vector. The direct pellet default
is exactly `[0.4157]` from `PrintConfig.cpp:2639-2643`. The three strict raw enum
domains come from `PrintConfig.cpp:1227-1248,5282-5295,5320-5333`:
`overhang_fan_threshold` accepts `0%`, `10%`, `25%`, `50%`, `75%`, and `95%`
with default `95%`; nullable `filament_retract_lift_enforce` accepts
`All Surfaces`, `Top Only`, `Bottom Only`, and `Top and Bottom` with concrete
source default `All Surfaces`; nullable `filament_z_hop_types` accepts
`Auto Lift`, `Normal Lift`, `Slope Lift`, and `Spiral Lift` with concrete
source default `Slope Lift`. The fixture carries `50%`, all nil, and
`Spiral Lift` respectively. The legacy `5%` threshold conversion at
`PrintConfig.cpp:8132-8133` is not part of raw parsing. `filament_notes` is the
only Task 13 string vector; raw empty, multiline, UTF-8, and literal `"nil"`
strings remain uninterpreted.

The fixture has exactly 42 two-entry vectors and 27 eight-entry vectors. The
27 are the exact Task 13 intersection with `filament_options_with_variant` at
`PrintConfig.cpp:8375-8415`: all 16 retract overrides,
`nozzle_temperature_initial_layer`, `nozzle_temperature`, the four filament
ironing fields, the three air-filtration toggles, and both exhaust-fan-speed
fields. Raw parsing and serialization preserve arbitrary valid cardinality and
do not perform active selection, resizing, or cross-field validation. All 69
fixture values differ from singleton defaults by cardinality; ignoring
cardinality, exactly 36 are semantic overrides and 33 repeat the source
default.

`FilamentOptions` exposes public `gcode`, `print`, `region`, and
`retract_overrides` children plus direct `pellet_flow_coefficient`. Its parent
serializer directly streams all 122 Task 12 plus Task 13 keys in one global
lexical map, split only into contiguous 41/41/40 helpers; it emits no nested
group, serde flattening, or DOM. A production-literal audit records collisions
for exactly 66 Task 13 names. The exact three-key complement is
`chamber_minimal_temperature`, `filament_long_retractions_when_cut`, and
`filament_retraction_distances_when_cut`; no consumer is migrated in this
slice.

Task 16 now selects the four nullable region ironing vectors into concrete
effective region values using the final top-surface filament ID. The four
legacy transformations
`bridge_fan_speed`, `cooling`, `overhang_fan_threshold=5%`, and
`chamber_temperatures` remain Task 19A work at
`PrintConfig.cpp:8048-8049,8105-8106,8132-8133,8184-8185`. Active sizing,
variant selection, full FDM normalization, and nullable retract inheritance
remain Task 19B work at `PrintConfig.cpp:9004-9054,9805-10023`,
`PrintApply.cpp:1164-1173`, and `Print.cpp:3166-3175`. The 20 all-nil effective
export omissions remain Task 19C at `GCode.cpp:5632-5640`. Existing
option/profile and G-code consumer migrations remain Tasks 20A and 20D, and
compatibility-parser removal remains Task 20E.

TDD first produced only the expected missing-interface failures for the three
new source groups, their enum, and the four new parent fields. The completed
focused matrix passes 22 tests and proves the exact inventory, partitions,
histograms, declaration orders, defaults and concrete types, nullable split,
fixture cardinalities and overrides, strict enums, every-field dispatch,
invalid shapes, arbitrary cardinalities, standalone lexical maps, and the flat
122-key parent bytes. The adjacent Task 12 matrix passes all 14 tests while
retaining its exact standalone 53-key child boundary.

Reviewed local verification passes all 4,296 workspace tests with three
configured skips and the 22-test dynamic-value audit with one configured skip.
Warning-denying workspace all-target Clippy, rustfmt, native `ares-core`, both
`ares-core` and `ares-wasm` WASM checks, tracked and untracked whitespace
checks, and the physical-LOC gate are green; the largest changed Rust module is
283 lines. Independent upstream, RED-test, final-specification, code-quality,
and frozen-byte reviews approve the slice under the user-approved temporary
OpenCode bypass. The exact pushed commit remains subject to the five-job Tier 1
gate before downstream implementation proceeds.

### Task 14: project/runtime residual raw options

Task 14 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its source boundary is the exact
difference between the 653-key fixture and the already typed Printer 132,
Process 352, and Filament 122 union:

```text
fixture 653 - printer 132 - process 352 - filament 122 = residual 47
ProjectGCodeSourceOptions 17
+ ProjectPrintSourceOptions 19
+ ProjectPresetSourceOptions 8
+ PresetMetadata 3
= Task 14 47
```

The literal complement of the three fixed preset lists is 48, not 47,
because `filament_colour` is commented out at fixed `Preset.cpp:1309`; Task 12
already owns it in `FilamentGCodeSourceOptions`. Task 14 therefore uses the
typed-union difference and does not duplicate that field. The corrected source
audit also fixes the raw enum domains and the empty-vector
`extruder_ams_count` default that were wrong in the earlier contract.

`ProjectRuntimeOptions { gcode, print, preset }` owns the 44 real raw options,
and sibling `PresetMetadata { from, name, version }` owns the three provenance
strings. `ProjectSettings` exposes both as concrete public fields. The three
metadata strings are not `PrintConfig` options and never enter the 44-key
runtime map.

The exact 44-real-option upstream type histogram is:

```text
coBool=2, coBools=2, coEnum=2, coEnums=1, coFloats=19,
coInt=1, coInts=4, coPercents=1, coPoints=2,
coString=2, coStrings=8
```

All 44 real values and all three metadata values are non-nullable on the wire;
an absent field resolves to its concrete typed default, while JSON null is
rejected. The fixed field types and defaults are:

| `ProjectGCodeSourceOptions` field | Upstream type | Fixed default wire value |
| --- | --- | --- |
| `deretraction_speed` | `coFloats` | `["0"]` |
| `filament_ids` | `coStrings` | `[]` |
| `filament_map_mode` | `coEnum` | `"Auto For Flush"` |
| `filament_map` | `coInts` | `["1"]` |
| `retract_before_wipe` | `coPercents` | `["100%"]` |
| `retraction_length` | `coFloats` | `["0.8"]` |
| `retract_length_toolchange` | `coFloats` | `["10"]` |
| `z_hop` | `coFloats` | `["0.4"]` |
| `retract_lift_above` | `coFloats` | `["0"]` |
| `retract_lift_below` | `coFloats` | `["0"]` |
| `retract_restart_extra` | `coFloats` | `["0"]` |
| `retract_restart_extra_toolchange` | `coFloats` | `["0"]` |
| `retraction_speed` | `coFloats` | `["30"]` |
| `nozzle_volume_type` | `coEnums` | `["Standard"]` |
| `extruder_ams_count` | `coStrings` / raw `AmsCounts` | `[]` |
| `bbl_calib_mark_logo` | `coBool` | `"1"` |
| `has_scarf_joint_seam` | `coBool` | `"0"` |

| `ProjectPrintSourceOptions` field | Upstream type | Fixed default wire value |
| --- | --- | --- |
| `curr_bed_type` | `coEnum` | `"Cool Plate"` |
| `first_layer_print_sequence` | `coInts` | `["0"]` |
| `other_layers_print_sequence` | `coInts` | `["0"]` |
| `other_layers_print_sequence_nums` | `coInt` | `"0"` |
| `extruder_colour` | `coStrings` | `[""]` |
| `extruder_offset` | `coPoints` | `["0x0"]` |
| `max_layer_height` | `coFloats` | `["0"]` |
| `min_layer_height` | `coFloats` | `["0.07"]` |
| `nozzle_diameter` | `coFloats` | `["0.4"]` |
| `retraction_minimum_travel` | `coFloats` | `["2"]` |
| `retract_when_changing_layer` | `coBools` | `["0"]` |
| `wipe` | `coBools` | `["0"]` |
| `wipe_distance` | `coFloats` | `["1"]` |
| `wipe_tower_x` | `coFloats` | `["15"]` |
| `wipe_tower_y` | `coFloats` | `["220"]` |
| `flush_volumes_matrix` | `coFloats` / raw `FlatMatrix` | 16 values, `0` on the 4x4 diagonal and `280` elsewhere |
| `flush_volumes_vector` | `coFloats` | eight `"140"` values |
| `flush_multiplier` | `coFloats` | `["0.3"]` |
| `start_end_points` | `coPoints` | `["30x-3","54x245"]` |

| `ProjectPresetSourceOptions` field | Upstream type | Fixed default wire value |
| --- | --- | --- |
| `print_compatible_printers` | `coStrings` | `[]` |
| `default_filament_profile` | `coStrings` | `[]` |
| `filament_multi_colour` | `coStrings` | `[""]` |
| `filament_colour_type` | `coStrings` | `["1"]` |
| `filament_settings_id` | `coStrings` | `[""]` |
| `print_settings_id` | `coString` | `""` |
| `printer_settings_id` | `coString` | `""` |
| `filament_self_index` | `coInts` | `["1"]` |

`PresetMetadata::default()` uses empty strings for `from`, `name`, and
`version`; the committed fixture carries `"project"`, `"project_settings"`,
and `"02.06.00.51"`. Its lexical wire order is exactly `from,name,version`.

The strict raw enum maps come from fixed `PrintConfig.cpp`, not UI suggestion
lists:

- `curr_bed_type`: `Default Plate`, `Supertack Plate`, `Cool Plate`,
  `Engineering Plate`, `High Temp Plate`, `Textured PEI Plate`, and
  `Textured Cool Plate`;
- `filament_map_mode`: `Auto For Flush`, `Auto For Match`, and `Manual`;
  UI-only `Default` is not a raw token; and
- each `nozzle_volume_type` element: `Standard` or `High Flow`.

Case variants, numeric forms, unknown tokens, UI-only tokens, and the legacy
spellings `SuperTack Plate`, `Auto`, `Normal`, and `Big Traffic` are rejected
at this raw boundary. Their conversions remain Task 19A.

The 44 real fixed declarations comprise 37 vectors and seven scalars.
Canonical save plus metadata is therefore exactly 37 JSON arrays and ten
scalar strings. The fixture's six singleton arrays are
`default_filament_profile`, `first_layer_print_sequence`,
`other_layers_print_sequence`, `print_compatible_printers`, `wipe_tower_x`,
and `wipe_tower_y`; the exact vector-length histogram is
`{1:6, 2:14, 4:15, 8:2}`. These lengths are fixture evidence only. Empty,
one-element, three-element, and other valid vector cardinalities remain valid;
the raw layer does not infer active extruders, AMS topology, or matrix
dimensions.

Exactly seven real fixture values equal their typed defaults:
`bbl_calib_mark_logo`, `filament_map_mode`,
`first_layer_print_sequence`, `has_scarf_joint_seam`,
`other_layers_print_sequence`, `other_layers_print_sequence_nums`, and
`start_end_points`. The other 37 differ. `AmsCounts`, `FlatMatrix`, point,
percent, bool, numeric, and string wrappers preserve their raw distinctions;
in particular, `[]` is not `[""]`.

Each child preserves fixed declaration or registration order in memory and
serializes its own direct lexical map. `ProjectRuntimeOptions` opens one
`SerializeMap(Some(44))` and streams one globally lexical flat 44-key map; it
does not emit nested child maps, serde flattening, a remainder map, or a DOM.
Tests merge Printer 132, Process 352, Filament 122, Project 44, and Metadata 3
into the exact pairwise-disjoint 653-key fixture union. That test-only merge
does not implement Task 18's production top-level `ProjectSettings`
visitor/serializer, duplicate/unknown dispatch, project loading, or
persistence. The complete 650-real-option histogram remains:

```text
coBool105/coBools22/coEnum44/coEnums9/coFloat160/
coFloatOrPercent36/coFloats90/coInt41/coInts45/coPercent25/
coPercents5/coPoint4/coPoints6/coPointsGroups1/coString30/coStrings27
```

The existing compatibility implementation contains literal collisions for 31
of the 44 real names. The exact 13-key complement is
`bbl_calib_mark_logo`, `extruder_offset`, `filament_self_index`,
`first_layer_print_sequence`, `flush_multiplier`, `flush_volumes_matrix`,
`flush_volumes_vector`, `has_scarf_joint_seam`,
`other_layers_print_sequence`, `other_layers_print_sequence_nums`,
`retract_length_toolchange`, `retract_restart_extra_toolchange`, and
`start_end_points`. This is a debt ledger only; Task 14 migrates no consumer
and changes no dynamic-value baseline.

The retained-only boundary explicitly defers all 17 effective residual G-code
projections to Task 17; strict full-fixture dispatch and persistence to Task
18; raw legacy key/value conversion to Task 19A; active sizing,
`filament_self_index`, AMS interpretation, vector/matrix normalization, and
cross-field validation to Task 19B; metadata exclusion, `extruder_colour`
substitution, scaled `flush_volumes_matrix`, duplicate plate-indexed
`wipe_tower_x/y`, and exact config-block export to Task 19C; and behavioral
consumer migration plus final dynamic compatibility-parser removal to Tasks
20A-20E.

TDD first ran the frozen focused RED twice and failed only on the planned
missing Task 14 interfaces. The implemented matrix passes 23 focused tests,
107 adjacent typed-option tests, all 4,319 workspace tests with three
configured skips, and the 22-test dynamic-value audit with one configured
skip. Rustfmt, warning-denying workspace all-target Clippy, native
`ares-core`, `ares-core` and `ares-wasm` WASM checks, release WASM generation,
the generated-binding real-3MF Playwright test, tracked and untracked
whitespace checks, the forbidden-dynamic scan, exact ownership audit, and the
under-400-physical-LOC gate are green; the largest changed Rust file is 290
lines. Independent final specification and code-quality reviews approve the
frozen implementation under the user-authorized temporary OpenCode bypass.
Commit `dc47e069ede1caa307411d63ba29f78784630494` and five-job Tier 1 run
`29253342315` are green and satisfy the Task 15 entry gate.

### Task 15: effective object options

Task 15 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is the
exact 126 active `PrintObjectConfig` fields at `PrintConfig.hpp:917-1071`
with concrete types, defaults, and enum domains from `PrintConfig.cpp`;
ordered model-object metadata and lexical decoding at
`Model.hpp:72-102,354-370`, `PrintConfig.hpp:2053-2128`,
`Format/bbs_3mf.cpp:2119-2132,4389-4399`, and `Config.cpp:573-654`; static
object projection at `PrintObject.cpp:3555-3579` and `Config.cpp:461-500`;
and the default-object recomputation and `num_extruders` input at
`PrintApply.cpp:1130-1133,1190-1194,1273-1283,1468-1482,1539-1548,
1646-1656`. The normalization write-set evidence is fixed to
`PrintConfig.cpp:8520-8741`.

One private compile-time inventory now expands three distinct concrete
126-field structs: raw `ProcessObjectSourceOptions`, all-absent-by-default
sparse `ObjectOptionOverrides`, and effective `ObjectOptions`. The effective
struct has no independent default; it copies the supplied typed process base.
Sparse presence is represented only by `Option<T>`, so an explicitly present
raw-default value still replaces a non-default base. The shared histogram is
22 bool, 12 enum, 63 float, six float-or-percent, 13 int, and ten percent
fields, with one production source for defaults and enum domains.

`ObjectSettings` processes object metadata in XML order. It retains the ID,
the last assigned `name` and `module`, typed sparse values for the 126 owned
keys, and every entry not consumed as `name`, `module`, an object option, or a
region option in ordered `retained_config`.
Repeated object fields and strings are last-write-wins, while a malformed
later assignment returns a keyed error instead of exposing an earlier valid
value. Part `matrix` metadata remains on the part path. Noncanonical aliases
remain ordered text for Task 19A; Task 16 now routes canonical region keys and
`extruder` into typed sparse region overrides. Neither stage invents a global
option registry or a legacy fallback.

Resolution copies the supplied base, applies only present sparse fields, then
runs the two post-overlay `PrintObject.cpp:3555-3560` clamps. For
`support_filament` and `support_interface_filament`, only a value strictly
greater than `num_extruders` becomes `1`; negative values, zero, one, and a
value equal to the extruder count remain unchanged. Resolution is recomputed
for each supplied extruder count. Test-owned fixed write sets separately
freeze monolithic `normalize_fdm` and split `normalize_fdm_1` /
`normalize_fdm_2`; all have zero intersection with the 126 object fields.
Normalization-driving keys such as `extruder`, `spiral_mode`, and
`enable_prime_tower` therefore remain outside this projection until the
reviewed normalization stage.

The real bounded 3MF path finds object ID 2 generically with
`name=ksr_fdmtest_v4.drc`, no module, typed region override `extruder=1`, and
zero typed object overrides or residual object config. Its two `0.4` nozzle
diameters supply
`num_extruders=2`; effective object state equals the complete typed process
base. Exactly 108 fields equal fixed defaults and 18 differ:
`brim_object_gap`, `brim_width`, `default_acceleration`,
`elefant_foot_compensation`, `initial_layer_acceleration`,
`inner_wall_acceleration`, `line_width`, `max_bridge_length`,
`outer_wall_acceleration`, `support_interface_bottom_layers`,
`support_interface_top_layers`, `support_line_width`, `support_speed`,
`support_type`, `top_surface_acceleration`, `tree_support_branch_angle`,
`tree_support_branch_diameter`, and `wall_generator`. These fixture facts are
test evidence, not production branches.

Included behavior is limited to the typed inventory, ordered object-metadata
handoff, sparse overlay, exact support-filament clamps, normalization
zero-intersection proof, and real-document verification. Region/extruder
propagation is now implemented by Task 16; G-code projection remains Task 17;
strict top-level
project storage Task 18; legacy rewrites Task 19A; general normalization,
active sizing, and object association Task 19B; config export Task 19C; and
consumer migration/removal Tasks 20A-20E. Geometry, slicing, G-code generation,
and final `ksr_fdmtest_v4` byte parity are not claimed by Task 15.

All six sequential slice filters are green for inventory/base identity,
ordered metadata, sparse projection, clamps, normalization, and the real
fixture. Each slice received independent specification and quality approval,
and the complete pre-documentation 28-file Task 15 diff has fresh literal
`SPEC VERDICT: APPROVE` and `QUALITY VERDICT: APPROVE`. Commit
`4fbb61282cdb73160414d2d9f67edacf61ba2e42` is pushed, and exact-SHA Tier 1
run `29273332261` is green across format, Ubuntu/Linux, WASM, macOS, and
Windows. This satisfies the Task 16 entry gate.

### Task 16: effective region options

Task 16 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is
`PrintConfig.hpp:1074-1249::PrintRegionConfig`, region construction and
override application at `PrintObject.cpp:3582-3709`, the model-part and
modifier call sites at `PrintApply.cpp:786-795,1021-1042`, final ironing reads
at `Fill/Fill.cpp:1591-1604`, ordered object/volume metadata at
`Format/bbs_3mf.cpp:2119-2132,4894-5117`, and the string/vector lexical codecs
at `Config.hpp:994-1067,1087-1158` and
`Config.cpp:123-144,146-215`. The Rust destination is the public concrete
`RegionOptions` projection plus crate-private sparse metadata handoff and
resolution under `ares-core::options`, with ordered object/part decoding under
`ares-core::project::model_settings`.

One shared compile-time inventory owns the 149 process-region fields and
expands both raw and effective concrete state. `RegionOptions` adds four
concrete selected ironing values, for 153 fields total. Presence-preserving
`RegionOptionOverrides` stores all 149 fields plus `extruder`; direct keyed
codecs implement comma-separated integer vectors, C-style scalar strings, and
the quoted/escaped semicolon string-vector grammar without a dynamic value
map. Object and part metadata are consumed in source order with last write
winning, consumed region entries omitted from residual storage, and part
structural metadata retained in exact order.

The crate-private `RegionBase` ADT makes the two upstream bases explicit. A
model part starts from process state and applies object, volume, material, then
layer-range overrides. A modifier starts from an already-resolved parent,
clears all six feature-explicit mask bits, and applies only volume then
material; object and layer-range inputs are unrepresentable in that branch.
For each feature filament ID, positive explicit values assign and set the mask,
nonpositive values clear it without assigning, and a positive same-scope
`extruder` fills only clear features. Finalization maps each ID at or below zero
or above `num_extruders` to one. Sparse density below the double value promoted
from Orca's `0.00011f` literal becomes zero and values above 100 become 100,
with equality at that promoted threshold retained. Every non-None
fuzzy variant becomes None when point distance is below `0.01` or thickness is
below `0.001`, again retaining equality. Only then does the final clamped
top-surface filament ID select all four nullable filament ironing vectors. A
nil selected value inherits its corresponding final ordinary ironing value.

The real project proves object `extruder=1`, all six effective feature IDs
equal to one, selected filament index zero, and nil inheritance to concrete
`10%` flow, `0.15` spacing, `0.21` inset, and `30` speed. These are typed
fixture observations, not production identity branches. Included behavior is
limited to the 149+4 projection, ordered metadata/codecs, both precedence
branches and feature mask, six ID clamps, density/fuzzy normalization, and
final ironing selection. G-code projection remains Task 17; active sizing,
association, and cardinality errors remain Task 19B; consumer migration remains
Tasks 20A-20E. Modifier graph construction, region deduplication, geometry,
slicing, G-code generation, and final byte parity remain deferred.

All seven sequential TDD slices received independent specification and quality
approval. The frozen 34-file whole diff also received literal
`SPEC VERDICT: APPROVE` and `QUALITY VERDICT: APPROVE`. Task 16 is released as
pushed commit `2651c6376d0cc8229876471d0a4d5c6f98f84314`; exact-SHA Tier 1
run `29286285164` is green across format, Ubuntu/Linux, WASM, macOS, and
Windows. Task 16 does not claim complete slicing or G-code parity.

### Task 17: registered pre-normalization GCodeConfig projection

Task 17 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is
`PrintConfig.hpp:759-776::StaticPrintConfig::StaticCache::finalize`, the
static class definition at `PrintConfig.hpp:838-865`,
`PrintConfig.hpp:1299-1476::GCodeConfig`, the `PrintConfig` and
`FullPrintConfig` inheritance boundaries at `PrintConfig.hpp:1479-1482` and
`:1662-1666`, and static cache initialization at
`PrintConfig.cpp:10571-10585`.

`GCodeConfig` declares 151 active C++ members, but only 149 enter the
registered runtime key set finalized from `PrintConfigDef`. The two excluded
members are unregistered `thumbnail_size`, a legacy input, and
`bbl_bed_temperature_gcode`, a temporary placeholder rather than an Option.
The registered fields are owned exactly once by the existing typed raw source
groups: 62 printer, 17 process, 53 filament, and 17 project/residual fields.
Their raw wire shapes are 69 scalars and 80 arrays, including nine nullable
arrays.

The Rust destination is `ares-core::options::gcode_fields`, which defines one
compile-time ledger in fixed HPP declaration order, and
`ares-core::options::gcode_options`, which exposes the public concrete
`GCodeOptions`. The effective type derives only `Clone`, `Debug`, and
`PartialEq`; it has no independent default or serde implementation. Its
crate-private infallible `from_sources` constructor directly clones each
identically named field from its unique typed source. It performs no
selection, resizing, inheritance, validation, normalization, fallback, or
runtime key lookup, so this is specifically a registered pre-normalization
projection rather than final effective configuration.

The exact registered type histogram is:

| Type | Count | Type | Count |
| --- | ---: | --- | ---: |
| `coBool` | 27 | `coBools` | 9 |
| `coEnum` | 6 | `coEnums` | 5 |
| `coFloat` | 14 | `coFloats` | 38 |
| `coFloatOrPercent` | 3 | `coInt` | 5 |
| `coInts` | 11 | `coPercent` | 1 |
| `coPercents` | 1 | `coPoints` | 1 |
| `coString` | 13 | `coStrings` | 15 |

Independent inventory, concrete-type, projection, template, shape, and real
3MF tests prove the compile-time ledger and direct typed projection without
using an upstream checkout as an active oracle. Fidelity coverage preserves
all 16 template fields byte-for-byte, including LF and CRLF line endings,
backslashes, placeholders, UTF-8, empty strings, and trailing newlines; it also
preserves four opaque typed strings and every one of the 80 array shapes,
including nullable elements. The real project is loaded through the bounded
in-memory reader and split into the four typed source groups only in test code.
Its array histogram is one empty array, 49 length-two, 19 length-four, ten
length-eight, and one length-ten array. The 19 printer-variant arrays remain
length four, the ten filament-variant arrays remain length eight, and the
other 43 filament G-code arrays remain length two.

Production code remains platform-neutral and WASM-safe: it adds no file I/O,
terminal/UI or FFI boundary, erased/dynamic option value, runtime registry,
JSON/serde round trip, fixture/reference branch, or source-line pinning test.
The real-project proof does not read the reference G-code.

Task 17 includes only the registered ledger, public concrete projection,
direct typed cloning, and the independent fidelity/fixture verification above.
Production flat project-settings parsing remains Task 18. Legacy conversion
remains Task 19A. Active sizing and printer/filament selection, nullable
retract overrides, model-driven recomputation, normalization, and final
reprojection remain Task 19B. Config export remains Task 19C; consumer
migration and compatibility-parser removal remain Tasks 20A-20E; template
evaluation remains Task 28; document assembly remains Task 29. Geometry,
slicing, generated G-code bytes, and complete `ksr_fdmtest_v4` parity remain
deferred.

All four Task 17 slices and the frozen whole implementation have independent
specification and quality approval. Task 17 is released as pushed commit
`18e7065856bee306cd643ffe359023758a60befe`; exact-SHA Tier 1 run
`29294487109` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.
That release gate completed before Task 18 implementation began.

### Task 18: strict typed ProjectSettings load

Task 18 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream load boundary is
`Config.cpp:573-685::set_deserialize_nothrow/set_deserialize/set_deserialize_raw`,
`Config.cpp:820-1100::ConfigBase::load_from_json`,
`Config.hpp:2763-2963::DynamicConfig`, and
`Format/bbs_3mf.cpp:210,1569-1573,1923-1926,2632-2653` for
`Metadata/project_settings.config`. The adjacent project-settings save path at
`Config.cpp:1464-1502::ConfigBase::save_to_json` and
`Format/bbs_3mf.cpp:6351-6355,7722-7728` is not part of this load task or the
current G-code parity program.

The Rust destination is a streaming `ProjectSettings` deserializer backed
directly by the existing concrete builders. It dispatches the complete 653-key
fixture into `PrinterOptions` (132), `ProcessOptions` (352),
`FilamentOptions` (122), `ProjectRuntimeOptions` (44), and `PresetMetadata`
(3), independent of input member order and with concrete group defaults for
omitted keys. The production path adds no flattened/dynamic value map,
`serde_json::Value`, `BTreeMap`, runtime registry, global key sort, or native
file I/O.

At the untrusted 3MF boundary, Ares intentionally rejects still-unknown
canonical keys and duplicate canonical assignments, although fixed Orca
ignores unknown keys after legacy handling and its materialized JSON object
collapses duplicates. Diagnostics remain compact and key-specific, including
`unknown Orca project option <key>` and
`duplicate Orca option <key>`. Archive loading wraps malformed typed
content as `invalid project settings JSON: ...`. The real fixture contains no
unknown or duplicate member. Its scalar values remain Orca-shaped strings and
its vectors remain string arrays. Existing typed Ares codecs continue to
accept native JSON booleans and numbers where already supported and
canonicalize them through their concrete group serializers; Task 18 neither
widens nor narrows that compatibility behavior.

`Project` now owns the concrete `ProjectSettings` and exposes
`Project::settings()`. The former production raw-byte field and accessor are
deleted. Raw fixture JSON is available only through a bounded test archive
oracle, which proves exact 653-member semantic equality against the five
standalone concrete group serializers. There is deliberately no production
`Serialize for ProjectSettings` and no project-settings JSON writer.

Task 18 remains isolated from the temporary dynamic `SliceOptions` shell and
does not change project slicing, geometry, or G-code generation. Legacy
key/value and complete-document composite conversion remain Task 19A; active
filament sizing, selection, inheritance, normalization, and recomputation
remain Task 19B; Task 19C owns only the effective `FullPrintConfig` G-code
`CONFIG_BLOCK`, not project JSON. Dynamic consumer migration remains Tasks
20A-20E, and complete `ksr_fdmtest_v4` slicing/G-code byte parity remains
deferred.

Slices 18.1-18.3 received their applicable independent per-slice reviews;
Slice 18.4 passed its verification-only isolation gate. The frozen whole
implementation then received independent whole-specification and whole-quality
approval. The real-3MF native and browser paths reach the existing
`ProjectSlicingIncomplete` boundary only after typed load succeeds. Task 18 is
released as pushed commit `a2714d4a6a197c5e10aec1b686e80e9b66794fd6`;
exact-SHA Tier 1 run `29298974173` is green across format, Ubuntu/Linux, WASM,
macOS, and Windows. That release gate completed before Task 19A implementation
began.

### Task 19A: typed legacy conversion across project inputs

Task 19A is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is
`PrintConfig.cpp:8033-8338` for per-entry legacy handling and the reachable
post-load composite, `Config.cpp:573-685,885-1017` for typed lexical decode,
JSON string/array iteration, alias lookup, and post-iteration slicing-state
writes, `PrintConfig.cpp:8099-8104,8121-8131` for the four deferred profile/UI
input rules, `Config.cpp:1018-1088` for deferred downstream profile-difference
bookkeeping, `Format/bbs_3mf.cpp:2119-2132,5088-5117` for ordered object and
part XML semantics, and `GCode/Thumbnails.cpp:530-577` for thumbnail
normalization. The Rust destination is the private
`ares-core::options::typed_legacy` action, conversion, project, model, and
thumbnail modules plus concrete builder entry points; it is not a new Ares
pipeline or a call into the temporary dynamic `SliceOptions` compatibility
path.

The compile-time source ledger records all 76 named fixed rules and the exact
44 obsolete keys. Seventy-two rules are executable typed project/model inputs;
the remaining four are explicitly deferred profile/UI bookkeeping inputs and
are rejected rather than stored in invented dynamic state. Obsolete inputs are
consumed without decoding. The ledger preserves direct and feature-filament
renames, conditional consumes, exact and global value rewrites, wall-order and
pattern conversions, filament-token rebuilding, JSON-only derived writes, and
each rule's string/array wire contract. The twelve registered vector targets
follow Orca's two-pass array behavior: an empty-value first pass, homogeneous
typed flattening, then one complete-string legacy pass. String-only targets
reject arrays unless the first pass consumes them.

Top-level project JSON remains a strict streaming concrete deserializer. A
canonical and legacy spelling share one target-presence bit, so collisions are
the same compact duplicate error as canonical/canonical assignments. Unknown
and deferred names report the exact input name. After the complete map is read,
`support_type=hybrid(auto)` applies `support_style=tree_hybrid`, and the two
infill-first wall spellings apply `is_infill_first=true`; these derived writes
overwrite an explicit target in either input order without becoming alias
duplicates. No generic JSON value, runtime option registry, or
`different_settings_to_system` state is retained.

Object and part XML metadata use only the per-entry conversion and preserve
document order. Canonicalized object/region owners are decoded directly into
their sparse typed owners, later canonical or legacy assignments win, and
canonicalized non-owner entries remain at the same ordered position for Task
19B. Obsolete and conditional-consume entries disappear. Structural metadata
and `mesh_stat` bypass option dispatch, while unclassified metadata remains
ordered and unchanged. XML receives neither the JSON-only derived writes nor
the top-level thumbnail composite.

The thumbnail composite runs on `ProjectSettingsBuilder` while input presence
is still observable. It acts only when canonical `thumbnails` or legacy
`thumbnail_size` was present, takes a missing per-item format from a present
`thumbnails_format` or otherwise PNG, preserves explicit per-item formats, and
normalizes valid items to `WIDTHxHEIGHT/FORMAT` joined by comma-space. Its
fixed stream-prefix dimension grammar and six-significant-digit default-float
formatting reject invalid dimensions, ranges, and formats through the typed
project-option error. An absent thumbnails assignment does nothing despite
resolved printer defaults.

The fixed-source exclusions are also behavioral contracts:
`perimeter_feed_rate` is not accepted as a Task 19A input, neither
`wiping_volumes_matrix` nor `wiping_volumes_use_custom_matrix` is created, and
canonical `flush_volumes_matrix` remains unchanged. The real project is
canonically idempotent through the public byte-oriented load path, and the
generated WASM web package still reaches `ProjectSlicingIncomplete` in
headless Chromium only after the same typed load and conversion complete.
These proofs do not read the reference G-code or claim generated G-code bytes.

As part of the whole-task boundary, obsolete checkout-dependent Orca
source-text pinning tests, exact source-line assertions, and generator mutation
probes were removed. The committed 653-row semantic option inventory and its
fixed-commit deterministic generator remain; regeneration is byte-identical to
the committed artifact. Fixed-source checks for the excluded aliases remain
review evidence rather than committed source-pinning tests.

All five Task 19A slices received independent specification and quality
approval after their RED/GREEN cycles. The frozen 49-path pre-documentation
implementation received literal `WHOLE SPEC VERDICT: APPROVE` and
`WHOLE QUALITY VERDICT: APPROVE`. Fresh local evidence passes 61 typed-legacy
tests, 160 adjacent tests, all 4,484 workspace tests with two configured skips,
the dynamic-value audit, rustfmt, warning-denying Clippy, native and WASM
checks, browser proof, and fixed-source exclusion scans. Task 19A is released
as pushed commit `0e85302416904d0de604b969afd7f546fb8b3c1a`;
exact-SHA Tier 1 run `29313932330` is green across format, Ubuntu/Linux, WASM,
macOS, and Windows. That release gate completed before Task 19B.1A began.

### Task 19B.1A: typed active variant materialization

Task 19B.1A is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream rewrite boundary is
`PrintConfig.cpp:8344-8473,8981-9054,9634-10023` for the four family ledgers,
selection guard, index lookup, and printer/process/filament materialization;
`PrintConfig.cpp:588-606` for canonical typed extruder/nozzle-volume spelling;
`PrintApply.cpp:1164-1173` for runtime family order; and
`Print.cpp:3166-3175` for restoring the saved pre-filament state before
rematerializing a changed map. `Config.hpp:624-630` owns the adjacent C++
vector recovery semantics. The Rust destination is the crate-private
`ares-core::options::project_variants` transform over existing typed option
owners, not a new orchestration pipeline.

`materialize_project_variants` clones an unmaterialized `ProjectSettings`,
installs the supplied typed `filament_map` in `ProjectRuntimeOptions`, and
mutates only the clone. `ProcessOptions` owns the two process selector fields;
`PrinterOptions` owns the printer selectors and machine fields;
`FilamentOptions` owns the logical-filament selectors and payloads; and the
24-member printer variant-1 family also crosses into existing
`ProjectRuntimeOptions` retract fields. The result is still a complete typed
`ProjectSettings`, not `FullPrintConfig`, a runtime G-code view, or serialized
configuration.

The source-ordered transform materializes exactly two process fields, 24
printer variant-1 fields at stride one, 15 printer variant-2 fields at stride
two, and 37 filament fields, plus the supplied map. Printer variant 1 and
process resolve the fixture's raw source indices `[0, 2]`. Because variant 1
also shortens the shared printer selectors, variant 2 resolves them again from
the current clone and obtains `[0, 1]`, selecting stride positions
`[0, 1, 2, 3]`; stale reuse of `[0, 2]` is forbidden. Filament selection uses
the installed map with raw fixture logical indices `[0, 4]`. The fixture's
selected printer/process IDs are values `[1, 2]`, not zero-based source
indices.

The activation guard and generated-ID lookup preserve Orca's distinct token
rules, typed Direct Drive/Bowden plus Standard/High Flow spelling, first exact
match, and one-based filament-map interpretation. At the untrusted external
project boundary, Ares is deliberately stricter than the adjacent C++
recovery. Fixed C++ falls back to index zero or ID/zero recovery when an exact
selector match is missing at `PrintConfig.cpp:9677-9682,9840-9854`; Ares
instead returns `SliceError::InvalidInput` naming the selector key. An
out-of-range selected printer/process payload likewise returns an error naming
that option instead of repeating its first value, and an invalid selected
filament payload errors instead of leaving a default-constructed element. The
inactive one-extruder/one-variant branch replaces only the map and does not
validate unused selectors or payloads.

Rematerialization is raw-source-only relative to variant selection. Callers
must rerun from the same unmaterialized typed source, which may already contain
earlier source-ordered normalization writes; a previously materialized result
is not a valid source. The transform never reloads the 3MF, preserves
`filament_self_index`, `extruder_variant_list`, and every non-family field, and
selects any already-normalized family payload supplied by its caller.

The implementation remains platform-neutral, filesystem-free, and isolated
inside `ares-core`; it adds no dynamic option value, JSON rematerialization,
adapter behavior, or fixture/reference branch and compiles through the existing
native/WASM boundaries. It is not yet called by project slicing. The current
Ares option/slicing scaffold remains a temporary compatibility shell until the
later source-cited orchestration replaces it, and the real project still
returns `ProjectSlicingIncomplete` through core and browser WASM.

All four TDD slices recorded genuine RED/GREEN evidence and culminated in
19/19 focused tests. The frozen thirteen-path implementation manifest
`96aa793696240f6d1a33d795e5e1ea308ee61a648fd2469d20263f98494d066b`
received independent specification-compliance, code-quality, and OpenCode
`VERDICT: APPROVE`; 235/235 adjacent typed tests, the 22/22 dynamic audit,
rustfmt, Clippy, fixture hashes, forbidden scans, and sub-400-LOC checks also
passed. Task 19B.1A was released as commit
`da896a98719a621ad87a2317c23f1d27f0a3c6e5`; exact-SHA Tier 1 run
`29330209222` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

Task 19B.1B's export/runtime split and nullable retract overlay, Task 19B.2's
model-option classification plus optional layer-config import and association,
and Task 19B.3's normalization plus source-ordered effective-project
orchestration are released. Task 19C's config export was released as commit
`656b32f987827b29d08010802ba03ef6ba822980`; exact-SHA Tier 1 run
`29457461048` is green. Tasks 20A-20E retain consumer migration and
compatibility-parser removal. Geometry, slicing, complete G-code generation,
metadata, post-processing, and normalized `ksr_fdmtest_v4` byte parity remain
deferred.

### Task 19B.1B: typed export/runtime retract views

Task 19B.1B is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream rewrite boundary is
`PrintApply.cpp:222-263,1164-1191,1261-1283` for retract-key diffing,
materialization order, and the preserved full/runtime split;
`PrintConfig.cpp:7374-7392,10300-10332` for the sixteen-key inventory and
nullable retract computation; `Config.hpp:713-751` for vector application;
`Print.cpp:3166-3195` for raw-source rematerialization;
`PrintConfig.hpp:1300-1478,1481-1610` for the twelve G-code and four print-only
owners; and `GCode.cpp:2532-2534,5552-5557,5591-5594` for the distinct runtime
and full-config consumers. The Rust destination is the crate-private typed
`ares-core::options::project_config_views` transform and its `retract` sibling,
not a public slicing pipeline.

The transform preserves the complete variant-materialized input as the
full/export view, clones it once for the runtime view, applies the nullable
retract overlays only to the runtime ordinary fields, and derives
`runtime_gcode` through the existing typed `GCodeOptions::from_sources`. The
twelve G-code-owned keys are `deretraction_speed`,
`long_retractions_when_cut`, `retract_before_wipe`, `retract_lift_above`,
`retract_lift_below`, `retract_lift_enforce`, `retract_restart_extra`,
`retraction_distances_when_cut`, `retraction_length`, `retraction_speed`,
`z_hop`, and `z_hop_types`. The four print-only keys are
`retract_when_changing_layer`, `retraction_minimum_travel`, `wipe`, and
`wipe_distance`. `travel_slope` is outside this override set and remains
unchanged.

For each typed vector, an empty machine or filament vector is a no-op. A
nonempty override must match `filament_map` cardinality or return
`SliceError::InvalidInput` naming the concrete `filament_*` key and
`filament_map`. `Value` replaces the logical entry directly;
`Nil` selects the one-based mapped machine default, with zero, negative, and
out-of-range indices falling back to machine element zero. A nonempty result
therefore preserves logical filament cardinality. Gate value `2` applies the
normal bool and distance overlays. Gates `0` and `1` first replace the bool
override vector with equal-cardinality all-`Nil` entries, while leaving the
long-distance machine vector and its physical cardinality unchanged. The
latter preserves the fixed upstream empty-float-temporary typo rather than
correcting it.

Changed maps are resolved only by rerunning Task 19B.1A's original source
materializer and then deriving fresh views; neither a previous full nor runtime
view is a rematerialization source. The obsolete dynamic
`filament_override` scaffold and its tests are deleted, and exactly its 31
baseline fingerprints are removed. The replacement remains byte/in-memory
only and portable across browser WASM, Windows, macOS, and Linux, with no
dynamic JSON or native I/O boundary.

The focused typed matrix passes 13/13 tests, the adjacent project/G-code
matrix passes 79/79, and the dynamic-value audit passes 22/22. The real 3MF
fixture and two-map test prove full/runtime differences and original-source
rematerialization. Frozen implementation manifest
`eb06ab4a08293acf2b89b4e026fc52ac02887118eb1845dae50048456cc5eedd`
received independent whole `SPEC_COMPLIANCE`, `CODE_QUALITY`, and OpenCode
`VERDICT: APPROVE` decisions. Public project slicing is deliberately not wired
and still returns `ProjectSlicingIncomplete`; the core/browser boundary and
full G-code parity remain incomplete by design.

Task 19B.1B was released as commit
`8e09be79881c6365100fac06ed064f487c75fb85`; exact-SHA Tier 1 run
`29345005311` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.
Task 19B.2 retains model/layer configuration association; Task 19B.3 retains
normalization and source-ordered orchestration; Task 19C retains config export;
and Task 20E retains the remaining dynamic compatibility removal.

### Task 19B.2: typed model/layer configuration association

Task 19B.2 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its canonical option boundary is
`PrintConfig.cpp:63-84,663-7328,7395-8031`,
`Config.cpp:258-318,573-685`, and the concrete scalar, nullable, vector,
string, enum, point, and point-group deserializers in `Config.hpp`. The model
association boundary is
`Format/bbs_3mf.cpp:744-764,2043-2168,3440-3513,3575-3735` and
`Format/bbs_3mf.cpp:3893-3908,4136-4400,4894-4954,5081-5126`,
`Model.hpp:354-370,865-918`, `Model.cpp:2717-2747`, and
`PrintConfig.hpp:2034-2128`. Optional layer ranges are bounded by
`Format/bbs_3mf.cpp:209-216,1896-1904,2087-2095,2886-2940,7517-7545` and
`Slicing.hpp:150-151`; later gap/overlap normalization in
`PrintApply.cpp:342-383` remains Task 19B.3.

The Rust destination is the private
`ares-core::options::model_config_deserialize` classifier plus typed ownership
on `ProjectObject`, `ProjectVolume`, and `LayerConfigRange`. The registry now
contains exactly 751 sorted unique fixed definitions: 18 missing canonical
rows were added and the legacy-only `solid_infill_filament`,
`sparse_infill_filament`, and `wall_filament` rows were removed while their
Task 19A lowering rules remain. The classifier covers all 21 concrete
`OptionValueKind` wire forms and the exact 650 typed-project-owner / 101
registry-only partition without an erased value, dynamic map, JSON round trip,
or public registry API expansion.

Object metadata classifies canonical object owners before region owners; part
and layer metadata accept only region owners. A canonical key with another
typed project owner is decoded through its existing concrete builder field and
discarded, while the remaining registry-only values are concretely validated
and discarded. The five registry-only scalar enum domains use one private
fixed lexical ledger from `PrintConfig.cpp:402-419,481-485`. Unknown keys
remain strict bounded errors. Model-path
legacy handling runs first, completes the three cumulative profile aliases and
`different_settings_to_system` validation without storing profile state, and
preserves XML source-order last-write-wins assignment.

Model-settings association keeps path-qualified geometry identity and the
existing ambiguous bare-ID rejection while attaching settings by bare object
ID and leaf mesh object ID. Final objects remain build-first-occurrence
ordered; nested leaf volumes remain breadth-first ordered. Objects own their
name, module, object overrides, region overrides, and sorted layer ranges;
volumes own their name, one of the five typed volume kinds, and region
overrides. Structural scope is exact: only object `name`/`module` and the fixed
part provenance fields bypass option classification.

Part selection preserves Orca's same-index match followed by first
source-ordered matching ID, including repeated IDs. Missing settings or
unmatched parts create default `ModelPart` metadata without changing the mesh
or accumulated component transform. Object and volume fallback names are
derived during assembly with the fixed unnamed counter. When no object
settings record exists, typed model XML `pid` and ordered material color groups
derive the one-based object `extruder` fallback with per-group last color,
submodel insert-only merge, root replacement, numeric group ordering, and
exact color deduplication; any matching settings record suppresses that
fallback.

Optional `Metadata/layer_config_ranges.xml` is read only through the bounded
in-memory `ProjectArchive`. ASCII case-insensitive lookup accepts one validated
case variant, rejects multiple variants as ambiguous, and treats absence as an
empty set. One-based ordinals target final object order. Source-ordered option
duplicates and exact-range duplicates use the later assignment; results sort
lexicographically while finite negative, reversed, gapped, and overlapping
ranges remain raw. Invalid XML, attributes, ordinals, bounds, keys, and values
return bounded contextual errors.

The frozen 73-entry implementation manifest
`2b80a68423b3476a7f83676393d72bc6129c6f1ce9f15654cea50a2dd7496eb7`
received independent whole `SPEC COMPLIANCE`, whole `CODE QUALITY`, and
OpenCode default-model `VERDICT: APPROVE` decisions. Independent verification
passed the 125/125 focused matrix, 4545/4545 workspace tests with two configured
skips, the 22/22 dynamic audit with one configured skip, rustfmt, warning-denying
Clippy, native/WASM checks, release WASM and wasm-bindgen, the real-project
browser test, fixture hashes, forbidden scans, diff validation, and the
sub-400-LOC audit.

The real KSR project now reaches these typed domain owners through public
loading, but public slicing intentionally still returns
`ProjectSlicingIncomplete`; the complete CLI golden remains configured skipped.
Task 19B.3 retains normalization, raw layer-range normalization, effective
object/volume/material/layer precedence, and the first production calls to the
typed configuration transforms. Task 19C retains config serialization; Tasks
20A-20E retain consumer migration and compatibility removal. Geometry,
toolpaths, G-code generation, and complete normalized KSR byte parity remain
open. Task 19B.2 was released as commit
`d5a50bd64b7ebe048c80919edc6028b57f83fefa`; exact-SHA Tier 1 run
`29391775108` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### Task 19B.3: effective project configuration resolution

Task 19B.3 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its normalization boundary is
`PrintConfig.hpp:628-631` and `PrintConfig.cpp:8520-8740`. The cold apply and
cardinality lifecycle comes from
`PrintApply.cpp:1113-1194,1256-1283,1525-1768` and
`src/slic3r/GUI/PartPlate.cpp:3503-3510`. Transform grouping, layer intervals,
and occupancy come from
`PrintApply.cpp:104-168,342-395,548-553,595-660,886-945`; candidate ownership
and precedence come from `PrintApply.cpp:1662-1747` and
`PrintObject.cpp:3555-3709`. Bounded filament participation is sourced from
`PrintRegion.cpp:71-110`, `Model.cpp:2512-2564`,
`Print.cpp:451-546,588-591,3290-3301,3385-3388`, and
`Print.hpp:362-365,429-431`.

The Rust destination is the typed `ares-core::options` project-normalization
boundary, private `ares-core::project::effective_config` resolution, and the
existing public project-slicing caller. Typed `normalize_fdm_1` preserves the
fixed ordered propagation and validation writes. Typed `normalize_fdm_2`
preserves its predicates, two-field write set, and changed-key result so the
same side effect can be applied to full, default-object, default-region, and
runtime owners. Project settings are validated before indexed resolution; the
existing dynamic `SliceOptions::normalize_fdm` path remains outside this
project caller.

The resolver performs the exact cold double-apply collapse: it normalizes the
unmaterialized source with one `_1` call, executes four source-ordered `_2`
calls across the two applies, rematerializes all four variant families from a
fresh normalized source for the second apply, discards preliminary candidates,
and builds only the final candidates and exported views from final materialized
state. This preserves the upstream dependency on first-apply object/region
state without introducing incremental cache or GUI state into the core API.

Cardinality remains explicitly split. Physical extruder count comes from
`nozzle_diameter`; logical filament count comes from materialized
`filament_diameter`. Filament maps and directly indexed vectors are validated
against their owning count. Object, region, volume, and layer selectors use the
logical count, including the fixed support-selector clamp-to-one for values
strictly greater than that count. A non-zero wipe selector must satisfy both
the strict physical bound and the logical output bound. Unequal physical and
logical test cases prevent either count from substituting for the other.

Raw layer ranges are normalized and queried with Orca's sorted interval,
gap/overlap, `EPSILON`, source-index, and unconfigured-tail behavior. Printable
instances are grouped by the exact ordered transform key. Minimal geometry is
limited to f32 Z-slab occupancy: composed print-object and source-volume
transforms determine whether a ModelPart occupies an interval without slicing
polygons or constructing toolpaths.

Each source `ProjectObject` owns one shared candidate vector. Its lexicographic
first transform-group representative supplies occupancy, and every group for
that object shares the result; different source objects never share it.
Candidates preserve source-volume and normalized-range identity. Effective
object options apply process then object precedence. Effective region options
apply process, object, volume, `None` for project material, then layer-range
precedence. Modifier-parent intersection, painted/fuzzy regions, painted
facets, and project material documents remain explicit unsupported or deferred
sources rather than inferred empty configuration.

`BoundedProjectUsage` composes only the supported filament sources. Region-role
participation, raw model/volume/layer selectors, support, brim, raft, and the
explicit wipe selector retain their fixed gates and composition points.
Object and support vectors deduplicate independently; their concatenation is
allowed to retain cross-vector duplicates until the wipe-participation check,
and only the final vector is sorted and deduplicated afterward. Support is
active for enabled support, enforced support layers, or `raft_layers > 0`.
Positive support selectors contribute their selected filament. After all
qualifying objects are scanned, a zero/current support selector appends the
deduplicated aggregate object-extruder set, matching `Print.cpp:514-519` and
`crates/ares-core/src/project/effective_config/usage.rs:79-83`. Print-wide brim
participation requires no raft and either `AutoBrim` at any width or another
non-`NoBrim` type at positive width; zero-width `Painted` remains explicitly
unsupported. Negative and zero raft counts both mean no raft for this
condition, and only a strictly positive count is a raft. Selector validation
and clamps use strict logical-count boundaries, while the wipe selector
additionally observes its separate strict physical bound.

Production project slicing now loads the 3MF, resolves the effective project
configuration, and only then returns `ProjectSlicingIncomplete`. This proves
the new path is called while preserving the public incomplete boundary until a
real slicing consumer exists. No reference G-code bytes or facts are used as
direct Task 19B.3 expectations; the unchanged complete CLI golden remains only
a configured-skipped regression contract.

The frozen 51-entry implementation manifest
`23CCB91EC4BE509E43EDECEFD864B83B9D7CB2B5C4DA2F0FF08020F52A8D5DEB`
received independent whole `SPEC COMPLIANCE`, whole `CODE QUALITY`, and fresh
OpenCode `VERDICT: APPROVE` decisions with no findings. Verification passed the
180/180 focused matrix, 4625/4625 workspace tests with two configured skips,
the 22/22 dynamic audit with one configured skip, the 5/5 CLI contract with one
configured golden skip, the 5/5 WASM contract, and the real-project browser
test. Rustfmt, warning-denying Clippy, native/WASM checks, release WASM,
wasm-bindgen, fixture hashes, forbidden scans, diff validation, and the
sub-400-LOC audit also passed. The independent spec reviewer additionally ran
its broader 195/195 focused selection.

Task 19B.3 was released as commit
`99fb0beba0a48603cb7875591cf77d02c26fb525`; exact-SHA Tier 1 run
`29444150217` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.
Task 19C retains effective config-block serialization. Tasks 20A-20E retain
consumer migration and dynamic compatibility removal. Project
material documents, modifier-parent and painted-region geometry, and per-plate
custom-G-code `ToolChange` filament participation from `Print.cpp:528-536`
remain deferred. The current loader does not retain those custom ToolChange
items, so the bounded resolver neither fabricates that source nor claims it can
reject it. Non-cold shrinkage-driven print-object/region regrouping is also
deferred; active non-100% `filament_shrink` or
`filament_shrinkage_compensation_z` values are instead rejected as
`UnsupportedProjectFeature` at this bounded resolver. Preset/UI sizing behavior
owned by `set_num_extruders`, `set_num_filaments`, `get_parameter_size`, and
`extend_extruder_variant` remains deferred, as do complete `FullPrintConfig`
conversion outside this resolver, wipe sequencing, geometry slicing, toolpaths,
G-code generation, metadata, post-processing, and final normalized KSR parity.

### Task 19C: exact effective config-block serialization

Task 19C is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream export boundary is
`Print.cpp:2618-2638`, `GCode.cpp:2030-2095,2461-2534,2637-2658,5591-5644`,
`Config.cpp:48-120,543-548,1715-1721`, the concrete serializers and nullable
rules in `Config.hpp`, the bed-temperature mapping in
`PrintConfig.hpp:489-509`, the external plate index in
`PrintBase.hpp:517-518,558`, and the CLI Bambu classification in
`src/OrcaSlicer.cpp:6045-6060`. The Rust destination is the crate-private typed
`ares-core::options::config_export` boundary plus the existing public project
caller; it does not create another flat config struct or a public partial-output
API.

The canonical body collects only the four typed owners in
`ProjectConfigViews::full`: 132 printer, 352 process, 122 filament, and 44
project-runtime entries. The resulting 650 unique entries are sorted by key;
the three preset metadata fields are excluded. A custom serde collector
consumes explicit semantic tags for string vectors, point groups, nullable
vectors, and nil values after each concrete type has produced its token. Those
tags remain transparent to existing JSON serialization, and the new path does
not round-trip through `serde_json::Value`, a registry, or a dynamic option
map.

Nullable state is explicit: an empty nullable vector and an all-nil nullable
vector are omitted, while mixed nullable vectors retain their `nil` positions.
Empty non-nullable vectors remain present. The writer applies the fixed nine
banned-key exclusions, scales each flush-matrix head by its matching
multiplier without mutating the source, substitutes typed filament colours,
writes both selected wipe-tower coordinates and their ordinary vector forms,
and appends the first-layer nozzle and bed temperatures from
`ProjectConfigViews::runtime`. Canonical option lines never read runtime-only
state.

The available printer classification is the exact case-sensitive
`printer_model.starts_with("Bambu Lab")` predicate. The project caller supplies
source plate index `0`; the writer accepts an explicit index and uses Orca's
first-element fallback for short non-empty vectors. It builds the complete
start/body/temperature/end block in a private scratch buffer and appends only
after success. Archive and materialization errors therefore retain precedence,
config-export errors precede the public incomplete boundary, non-Bambu projects
skip this writer, and every otherwise valid project still returns
`ProjectSlicingIncomplete`.

For the committed KSR project, the exported block is exactly 49,004 bytes with
SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`,
639 assignment lines, and 637 unique assignment keys. Fifteen all-nil options
are omitted, five empty non-nullable options remain, and the two wipe-tower
coordinates account for the duplicate keys. The generic thumbnail parser now
canonicalizes multi-thumbnail values with a comma and no added space, matching
the typed project value and the generic scalar writer without a thumbnail key
special case.

Task 19C also removes the remaining executable source-path/line/symbol pinning
assertions from the project inventory test while retaining its behavioral
ownership, type, default, projection, wire-shape, legacy-conversion, and fixture
agreement checks. The 39-path implementation received independent whole spec,
whole code-quality, and default-model OpenCode `VERDICT: APPROVE` decisions.
Fresh pre-documentation verification passed 29/29 config-export tests, 389/389
project tests, 4654/4654 workspace tests with two configured skips, 15/15 CLI
tests with the complete KSR golden as the sole CLI skip, the native/WASM build
matrix, and the real-project browser test; warning-denying Clippy and the
sub-400-LOC audit also passed.

This task does not slice geometry, generate toolpaths, assemble the complete
G-code document, emit generated-by metadata, estimate time, post-process
output, migrate Tasks 20A-20E consumers, or remove their dynamic compatibility
shells. Project material documents, unsupported painted/modifier sources,
selected-plate public plumbing beyond the source default, metadata and adapter
assembly, and final normalized `ksr_fdmtest_v4` byte parity remain deferred.
Task 19C was released as commit
`656b32f987827b29d08010802ba03ef6ba822980`; exact-SHA Tier 1 run
`29457461048` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### Task 20A.1: typed profile fragment, inheritance, and composition

Task 20A.1 is fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its profile-kind and identity
boundary is `src/libslic3r/Preset.hpp:22-24,43-65`; its ownership and
load/inheritance boundary is
`src/libslic3r/Preset.cpp:491-504,1476-1494,1622-1703,3112-3140`; and its
composition boundary is the `full_fff_config(false, std::nullopt)` subset of
`src/libslic3r/PresetBundle.cpp:3884-4165`. The concrete option owners come
from `src/libslic3r/PrintConfig.hpp:695-914,916-1666`; the dynamic upstream
load shell at `src/libslic3r/PrintConfig.hpp:610-682` is replaced at the Rust
API boundary. The separate calibration path
`src/libslic3r/PresetBundle.cpp:68-242::construct_full_config` is not an owner
for this slice.

`ProfileFragment::from_json_bytes` performs two order-independent streaming
serde passes over the supplied bytes: one reads concrete local/config metadata
while skipping option payloads, and the other dispatches option fields into
the sparse builder for the discovered profile kind. Both passes require a
complete input document. Unsupported kinds, wrong-kind or unknown option keys,
duplicate local or option fields, malformed typed values, and trailing tokens
return `InvalidInput`; the implementation retains neither a generic JSON tree
nor a dynamic unknown-value side map.

Same-kind inheritance uses a deterministic unique index and overlays the
oldest parent through the selected child. The compile-time sparse builders
consume child fields and replace only present values, then resolve concrete
defaults once after the full chain, so absence is preserved without runtime
field lookup, serde conversion, or a presence bitmap. Compatibility fields use
the same whole-field presence semantics: omission inherits and explicit empty
clears. A child filament uses the resolved root filament identity, while a
root retains its own `filament_id`. `thumbnails` and `thumbnails_format` are
also inherited as whole fields and normalized only once after the final
machine overlay. Per-element nullable inheritance and variant-indexed diff
mapping are explicitly deferred; present vectors replace whole vectors while
preserving their concrete nil/value elements.

The public merge result is the exhaustive by-value `MergedProfile` enum, whose
machine, process, and filament variants carry the corresponding concrete
options and `MergedProfileMetadata`. `ComposedProfile` exposes selected names,
typed `ProfileGroupMetadata`, and `ProjectSettings` through `settings()` and
`into_settings()`; it does not expose a `SliceOptions` map. Composition starts
from typed defaults and resolved profiles. The sparse overlay and append paths
are generated beside the concrete option declarations as zero-cost typed
operations with no dynamic lookup.

Multi-filament composition opts exactly four declaration groups into
compile-time append: 53 G-code fields, 48 print fields, four region fields,
and 16 retract-override fields, plus the direct
`pellet_flow_coefficient`, for the fixed 122-field inventory. Values append in
selection order using their concrete numeric, bool, string, enum, nullable, or
newtype representations. The typed project result records selected profile
IDs, positional `filament_ids`, `filament_map`, and `filament_self_index`.
`inherits_group`, compatible-machine conditions, and compatible-process
conditions remain positional and are omitted only when every slot in the
respective group is empty.

The migration removes exactly the 29 dynamic fingerprints previously owned by
the profile fragment/composition pair. The syntax-aware baseline retains 683
unchanged findings and the allowlist is unchanged. It also removes exactly the
two obsolete retained-STL tests that passed merged/composed profile maps into
the legacy slicer, plus the inventory test that pinned Orca source-citation
layout; behavioral typed profile and option-inventory coverage remains.

Task 20A.1 does not connect profiles to `slice_project`; a valid project still
reaches `ProjectSlicingIncomplete` after the released Task 19C config writer.
Profile discovery and management, alias/Semver/compatibility evaluation,
remaining Task 20A consumers, Tasks 20B-20E, geometry, toolpaths, G-code,
generated-by metadata, post-processing, adapters, and complete normalized KSR
parity remain deferred. Task 20A.1 was released as commit
`e0c50564283744b3dd3388eeaa10f624a492ff1f`; exact-SHA Tier 1 run
`29488449752` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### Task 20A.2: typed filament variant-aware inheritance

Task 20A.2 remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is
`PrintConfig.cpp:63-84,8375-8415,10209-10297`,
`Preset.cpp:231-278,922-945,1679-1697`,
`Config.hpp:558-580,624-665,812-837,921-931,1008-1016,1203-1218,1872-1879`,
and `libslic3r.h:52,306-310`. The Rust destination remains the concrete
filament option owners and typed profile resolver; this task does not add a
parallel dynamic pipeline.

The bounded family is exactly the stride-one, no-extruder-ID set of one
`filament_extruder_variant` mapping identity plus 36 data vectors. The
concrete root resolves against typed defaults, derives its cardinality from
the identity, applies each field's typed all-nil, empty, or no-reset rule, and
then clears, truncates, or grows vectors by their first value. Descendants stay
sparse: only present family fields are normalized, their identity is mapped
against the retained normalized root identity using the first exact match,
and the identity itself is never assigned as data.

Local comparison uses approximate nullable float/percent equality for exactly
19 vectors and exact equality for the other 17. Nullable child `Nil` preserves
the accumulated source slot and equals only `Nil`. An `N == 0` root keeps an
empty identity; its first implicit one-slot descendant reaches the whole-field
source-length fallback, while later equal-length descendants use normal
nil/value slot behavior. Equality short-circuits before fallback, and a source
length differing from the retained-root mapping length copies the normalized
child before any slot read.

The migration deletes exactly the eight obsolete dynamic findings owned by
the old filament diff scaffold and retains 675 baseline findings without an
allowlist addition. Profile-to-project wiring is unchanged, so a valid project
still reaches `ProjectSlicingIncomplete` after the released Task 19C config
writer.

Printer and process variants, stride-two behavior, profile-to-project wiring,
the remaining Task 20A work and Tasks 20B-20E, geometry, toolpaths, G-code,
generated-by metadata, post-processing, metadata byte parity, and complete
normalized KSR parity remain deferred. Task 20A.2 was released as commit
`4281e913b8eeaaeb6111cbefdf06f896f5c611aa`; exact-SHA Tier 1 run
`29520118127` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### Task 22A: typed slicing parameters and fixed layer planning

Task 22A remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is
`Slicing.hpp:25-38,44-52,66-85,98-114`,
`Slicing.cpp:24-43,62-70,106-146,228-304,713-866`,
`Model.cpp:1460-1499`, `PrintRegion.cpp:71-109`,
`PrintObject.cpp:3683-3686,3732-3833`,
`PrintObjectSlice.cpp:24-73,817-830`,
`PrintApply.cpp:104-167,1015-1054,1525-1621`, `Config.hpp:624-628`,
`libslic3r.h:46,48-60,300-310`, and the painted-profile archive handling in
`Format/bbs_3mf.cpp:209-216,1896-1903,2087-2095,2824-2881`. Private
`project_slice` modules own `SlicingParameters`, `PlannedPrintObject`, and
`PlannedLayer`; the prepared state owns the loaded project, its single resolved
configuration, optional config block, and materialized planned objects before
the existing public incomplete boundary.

The supported subset rejects a case-insensitive painted layer-height-profile
entry, an object-owned range `layer_height`, raft/support/precise-Z requests,
resolved region ZAA, and any typed true parameter-modifier `zaa_enabled`.
Painted-profile and range-height checks are typed presence gates; modifier ZAA
is conservatively rejected until modifier geometry and region assignment exist.
No deferred input is silently converted into a fixed-height plan.

Planning preserves the resolved object's stable source identity. Object height
uses the source object's first instance composed with each model-part volume
transform, requires every transformed vertex to be finite, and takes the
maximum Z across every mesh vertex, including unreferenced vertices.
Object-extruder collection covers the six gated region feature
selectors, print-wide brim contribution, and object/volume fallback for model
parts and parameter modifiers, then sorts and deduplicates zero-based IDs. The
nozzle helpers deliberately reproduce Orca's subtract-one/first-value indexing
without consulting `filament_map`; a bare range selector participates only
through an occupied resolved feature fallback.

The fixed profile installs the first layer, appends the uncovered regular-height
interval, and compresses adjacent approximately equal points. Pair generation
uses Orca's midpoint termination and produces ordered records with pair-index
IDs, `height = hi - lo`, `print_z = hi`, and midpoint `slice_z`. A single
project-wide budget permits exactly 100,000 materialized records and rejects
the next one; it is a generic input resource limit, not fixture data.

Using only `ksr_fdmtest_v4.project.3mf`, the approved implementation prepares
one planned print object with 460 complete records. Its first record is
`(id=0,height=0.2,print_z=0.2,slice_z=0.1)` and its final print-Z bits are
`0x4057000000000036`. The released Task 19C config block remains exactly 49,004
bytes with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.
The public API still returns `ProjectSlicingIncomplete` only after private
planning; it emits no placeholder or successful G-code.

Variable/adaptive layers, modifier geometry, Clipper behavior, paths and G-code,
generated metadata, and successful full KSR parity remain explicitly deferred.
Task 22A was released as commit
`91fc19f1dbfc85d21431791d2d5acb78af818671`; exact-SHA Tier 1 run
`29543841835` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### Task 22B: scaled raw mesh intersections

Task 22B remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is the
coordinate domain in `libslic3r.h` and `Point.hpp`, Bambu mesh import and fresh
mesh preparation in `Format/bbs_3mf.cpp`, `TriangleMesh.cpp`, `Model.cpp`, and
`Model.hpp`, object/volume identity and slicing transforms in `ObjectID.hpp`,
`Print.hpp`, `PrintObject.cpp`, `PrintApply.cpp`, and `PrintObjectSlice.cpp`,
and shared-edge/facet/multi-plane raw intersection behavior in
`TriangleMesh.cpp` and `TriangleMeshSlicer.cpp`. Private `ares-core` modules
`geometry`, `mesh_slicer`, `project::load`, and `project_slice` own the Rust
rewrite; no legacy STL pipeline is called as a fallback.

The loader materializes Bambu coordinates through f32, normalizes winding once,
omits empty meshes, compensates fresh-mesh centering, and bounds iterative
build-reachable component expansion. Project slicing selects a request-local
scale from resolved 3MF `printable_area`, checks the half-open i64 coordinate
boundary, constructs the raw center and centered slice transform, assigns
one-based per-source-object volume ordinals, builds shared-edge topology before
intersection, and dispatches faces across ordered `slice_z` planes. Facet
intersection retains Orca's strict f32 plane comparisons and top-edge/on-plane
ownership, directed endpoint provenance, vertex-coordinate truncation, and
interior `floor(value + 0.5)` conversion. Three independent request-wide
limits each accept exactly 1,000,000: expanded-model
occurrence/vertex/triangle units are claimed before scheduling or
materialization, dense layer slots are checked before allocation, and retained
raw lines are claimed before append. Nonempty layer ranges, distinct
print-object centering groups, explicit/shared mesh reuse, nonidentity shrink,
and normalized edge groups with more than two uses are gated before unsupported
work can be approximated.

Using only the committed 3MF, the approved implementation retains one
model-part volume with 6,109 vertices, 12,234 triangles, 18,351 normalized
shared edges, 460 layer slots, and 116,472 directed raw lines. The
source-semantic and deterministic Ares-order fixed-width encodings have SHA-256
`a82b2d193c23c8ba499c7abd56e21cb9956f5444e9b51b1b261a7e9b67d26d21`
and `1a6e83f2d5f53b73fa7ba9cb6444909816276496361f7fb9f9305412d2045e79`.
The Task 19C config block remains exactly 49,004 bytes with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.

Distinct transform-group center rotation/decomposition, nonidentity XY/Z
shrink, full typed layer-range membership and slab filtering, importer-global
shared-mesh cache/reuse and compensation, absolute process-global `ObjectID`
values, and undefined pairing for normalized edge groups with more than two
uses remain deferred. The same is true for remaining `Line`/`Polyline`/
`Polygon`/`ExPolygon` bounds, area, containment, orientation, and other
non-clipping path-domain operations; edge/vertex chaining, seed flags,
open-chain joining and repair, loops, and path ordering; and Clipper booleans,
PolyTree/fill rules, union, offset, simplification, closing, contour/hole
construction, and polygon ordering.

Geometry consumption of `slicing_mode`, `slice_closing_radius`, `resolution`,
and XY compensation is deferred, as are negative/modifier booleans,
range/region assignment, painted segmentation, fuzzy skin, interlocking,
conical overhang, slicing-error repair, final cleanup, and reproduction of an
Orca TBB raw-append schedule. Surfaces, elephant-foot compensation, perimeters,
fill, brim, supports, toolpaths, motion, G-code assembly, generated metadata,
time estimation, and post-processing remain later slices. Embedded/external
presets, CLI overrides, UI behavior, any Ares-owned alternative pipeline, and
final normalized KSR parity are also explicitly deferred. Supported requests
still return `ProjectSlicingIncomplete`, but only after the private raw state is
built. Task 22B was released as commit
`455a0d12a9c6ac48f6e2796669b4300a6a6190a2`; exact-SHA Tier 1 run
`29610017653` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### Task 22C: triangle-connectivity slice chaining

Task 22C remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is the
integer point and ordered-point storage in `libslic3r.h`, `Point.hpp`,
`MultiPoint.hpp`, and `Polygon.hpp`, the intersection reference/line and local
open-polyline records in `TriangleMeshSlicer.cpp:58-145,1043-1056`, and only
`chain_lines_by_triangle_connectivity` plus its first `make_loops` call in
`TriangleMeshSlicer.cpp:1058-1161,1383-1415`. Private `ares-core`
`geometry::polygon`, `mesh_slicer::chaining`, and
`project_slice::chained_intersections` modules own the Rust rewrite. The
project path does not call the legacy f64 STL segment/contour pipeline.

Each raw layer is consumed once. Separate flat Edge and Vertex start-reference
indexes preserve tagged identity, and records are ordered by identity followed
by original raw index. Raw index remains the component seed order and provides
a deterministic FIFO tie-break within an equal-identity range. Chaining follows
only the directed last-B to candidate-A identity, never connects by coordinate,
inspects candidate B, or reverses a line. Closed polygons preserve ordered
integer points without a duplicated terminal point, start rotation, winding
normalization, or cleanup. Unclosed chains preserve their tagged start/end,
all ordered points, f64 Euclidean length, and initial unconsumed state.

The project wrapper preserves print-object order and plan, one-based volume
ordinal and type, and every planned layer slot including empty layers. The
production `slice_project` path moves Task 22B raw state directly through this
wrapper, traverses the resulting polygons and open polylines, and continues to
return `ProjectSlicingIncomplete`; Task 22C consumes no new Option and emits no
placeholder or successful G-code.

Using only the committed 3MF, the implementation produces 460 chained layer
slots, 3,288 closed polygons, zero open polylines, and 116,472 closed polygon
points. The exact face/seed-order encoding has SHA-256
`6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`.
The independently normalized numeric encoding is 2,190,993 bytes with SHA-256
`7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`.
The Task 19C config block remains exactly 49,004 bytes with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.

Task 22D subsequently took the adjacent source-cited open-chain boundary:
`TriangleMeshSlicer.cpp:1163-1381,1428-1462`, including length ordering,
identity-exact joining with the source's allowed reversal passes, nearest-end
search, 2 mm gap repair, and remaining loop-closing passes; its implemented
outcome is recorded below. `slicing_mode`, hole ownership, Clipper processing,
negative/modifier volume booleans, regions, surfaces, perimeters, fill,
supports, toolpaths, motion, G-code assembly, metadata, post-processing, and
complete normalized `ksr_fdmtest_v4` parity remained beyond Task 22C.

### Task 22D: open-polyline loop repair

Task 22D remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its upstream boundary is open
length and signed area in `MultiPoint.hpp:172-187`, unconsumed length ordering,
identity-exact joining, and 2 mm gap repair in
`TriangleMeshSlicer.cpp:1163-1381`, plus the four-pass call order and final
loop return in `TriangleMeshSlicer.cpp:1428-1480`. Private `ares-core`
`mesh_slicer::chaining::{exact,gaps}`, the request-local spatial index, and
`project_slice::looped_intersections` own the Rust rewrite. No public geometry
API or legacy STL contour fallback is introduced.

The implementation runs exact same-direction, exact reversal-enabled, gap
same-direction, and gap reversal-enabled passes in that fixed order, then
intentionally drops residual opens exactly when the source returns only
polygons. Exact passes seed from descending cached length. Gap passes recompute
length before sorting. Source-unspecified equal-length, equal-identity, and
equal-distance choices use original open index followed by Start before End.
Identity lookup preserves the source's signed mapping `Vertex(n) -> +n` and
`Edge(n) -> -n`, including the observable `Vertex(0)`/`Edge(0)` collision and
the non-reversed pass's stale-end behavior.

Gap lookup uses exact widened squared distances, a strict radius comparison,
and the source's closure-before-attachment and conditional 30% branch order.
Coordinate differences, cell arithmetic, and area intermediates widen before
subtraction, addition, or squaring. The upstream 2 mm repair threshold is not
a 3MF Option: production scales it through the request-local
`CoordinateScale` already selected from the resolved 3MF `printable_area`,
yielding 2,000,000 normal units or 199,999 large-bed units. Junction omission,
nonzero bridge retention, changed-end reinsertion, and reversal/orientation
gates follow the fixed source.

The looped project wrapper consumes object, plan, volume ordinal/type/order,
layer slots, and polygon order exactly once. Production traverses the looped
state and still returns `ProjectSlicingIncomplete`; Task 22D consumes no new
Option and emits no placeholder G-code. A mutation-sensitive synthetic oracle
locks all four passes and their order, while a two-face project mesh proves a
real three-point open is repaired through the project wrapper.

The committed KSR 3MF enters this boundary with zero opens, so repair is an
exact no-op: 460 layers, 3,288 polygons, and 116,472 points remain unchanged.
The face-order and independently normalized encodings remain 2,190,993 bytes
with SHA-256
`6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`
and `7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`.
The Task 19C config block remains 49,004 bytes with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.

Task 22E ports the adjacent source set `TriangleMeshSlicer.hpp:11-33`,
`PrintConfig.hpp:162-170,947`, `PrintConfig.cpp:307-312,6030-6042`,
`PrintObjectSlice.cpp:138-225`, and
`TriangleMeshSlicer.cpp:1483-1532,2003-2049`. The private
`ares-core::mesh_slicer::slicing_mode` module owns the direct raw-mesh polygon
policy: `Regular` and `EvenOdd` preserve polygon order and orientation,
`Positive` makes every polygon counter-clockwise, and
`PositiveLargestContour` keeps the first polygon with the strictly greatest
absolute area and makes it counter-clockwise. Empty input remains empty,
`Positive` leaves zero-area polygons unchanged, and a nonempty all-zero-area
`PositiveLargestContour` input is an internal invariant failure. This direct
policy is separate from project slicing because upstream `slice_mesh_ex` delays
largest-contour selection until after polygon combination.

The private `ares-core::project_slice::slicing_mode_intersections` module owns
the 3MF-derived project adapter. It resolves the object base mode from the
object Option overlay, maps external `Regular`, `EvenOdd`, and `CloseHoles` to
the corresponding internal `Regular`, `EvenOdd`, and `Positive` modes, and
applies those modes in object-plan, layer, and source-volume order. The original
`PositiveLargestContour` choice is retained as project state while raw
intersections use `Positive`, so this slice does not discard contours before
the later `ExPolygon` boundary. Raw, chained, and looped intersections carry
the actual source-volume index instead of treating a filtered ordinal as the
volume identity.

When spiral mode is enabled, only model-part volumes receive
`PositiveLargestContour` above the bottom region; negative and modifier volumes
continue to use the object base mode. The bottom region first counts
`bottom_shell_layers`, then includes additional layers only while
`f64::from(slice_z as f32) < bottom_shell_thickness - 1e-4`. Negative layer
counts and negative or non-finite thicknesses are rejected only when spiral
mode consumes those Options. No native TBB execution model is copied into the
Tier-1/WASM core.

The committed KSR project resolves to external `Regular`, spiral mode disabled,
three bottom layers, and zero bottom thickness, so Task 22E is an exact no-op on
its Task 22D contour result: 460 layers, 3,288 polygons, 116,472 points, and the
same face-order and normalized hashes remain. Archive-level process, object
override, and spiral-threshold mutations prove that the behavior is assembled
from 3MF Options rather than fixture identity. Production still returns
`ProjectSlicingIncomplete`; no placeholder G-code is emitted. The implementation
passed the focused, workspace, native and WASM checks, browser-real-3MF,
code-quality, default-model, and independent six-dimensional review gates.

### Task 22F: safe Clipper 6 pre-closing union

Task 22F is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its source boundary is the closed
Boolean and PolyTree dependency closure in
`deps_src/clipper/clipper.hpp:75-81,88-100,121-123,137,141-223,225-535` and
`deps_src/clipper/clipper.cpp:67-72,78-161,167-426,429-1614,1630-3340`, exact
full-range slope products from `Int128.hpp:234-277`, and the direct union and
tree ownership wrappers in
`ClipperUtils.cpp:169-204,303-350,634-668,737-740,812-814`.
`TriangleMeshSlicer.cpp:1738-1823,2003-2034` is consumed only
through its initial `union_ex` result. Portable volume creation order comes from
`Model.hpp:1227-1230`, `ObjectID.hpp:20-87`, and the released import ordinal.

The private `ares-core::geometry::clipper` module owns a safe typed-index
closed-path engine with all four operations and fill rules, exact winding,
intersection, horizontal, join, ordered Paths, and PolyTree behavior. Minima
and intersection ordering use one platform-neutral Rust rewrite of the
separately audited MSVC STL 14.44 sort control flow. `union_ex` preserves the
fixed two-pass Paths-then-fresh-PolyTree overlap workaround, and
`ares-core::geometry::expolygon` owns each contour and its ordered holes. No
native Clipper library, host sort, platform branch, unsafe graph, or output
canonicalization is used. ARD-0024 is accepted, and the component-scoped
BSL-1.0 and Apache-2.0 WITH LLVM-exception provenance is carried with the
implementation.

The private `ares-core::project_slice::pre_closing_unions` stage sorts each
object's volumes by `VolumeOrdinal`, rejects duplicate ordinals as an internal
invariant, projects `Regular` and `Positive` to NonZero, `EvenOdd` to EvenOdd,
and `PositiveLargestContour` to Positive, and applies `union_ex` to every
retained volume/layer slot. Empty slots and the original mode, source-volume
index, ordinal, volume type, and owned result remain present. External Clipper
coordinate overflow maps once to `SliceError::InvalidInput`; there is no raw
polygon or alternate-engine fallback.

The complete KSR pre-closing encoding is 1,645,481 bytes with SHA-256
`209c6149c93994cc3ae6fa8e2f8f43dc9875b1b07b2320da9e67d8a2c43ab6e2`:
2,891 contours, 397 holes, and 99,260 points. Exact representative layers
0, 46, and 459 and the first hole-bearing layer match the corrected fixed
source oracle, and repeat runs are byte-identical. Task 22F has 50 focused
tests, passes the workspace, native, WASM, browser, code-quality,
default-model, and independent six-dimensional gates, and leaves both committed
fixtures unchanged. Production deliberately still returns
`ProjectSlicingIncomplete`; no placeholder or reference-derived G-code is
emitted, so complete normalized KSR parity is not claimed.

### Task 22G: safe closed ClipperOffset and project closing

Task 22G is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports only closed
`ClipperOffset` from `clipper.hpp:138-139,144-167,538-575` and
`clipper.cpp:63-65,73-106,128-134,150-161,1000-1036,3345-3777`, the directly
used defaults and `offset_ex`/`offset2_ex` wrappers from
`ClipperUtils.hpp:17-34,326-355,389-393` and
`ClipperUtils.cpp:264-293,303-315,333-351,360-410,437-558,560-585`, and the project
consumer in `TriangleMeshSlicer.hpp:20-46`,
`TriangleMeshSlicer.cpp:1738-1824,2003-2034`, and
`PrintObjectSlice.cpp:145-221`.

The private `geometry::clipper::offset` modules reuse the Task 22F Boolean and
PolyTree kernel for closed input normalization, orientation, normals, Miter,
Square, and Round joins, positive and negative execution cleanup, and the
directly used ExPolygon ownership wrappers. The second `offset2_ex` stage uses
one PolyTree cleanup and does not call Task 22F's two-pass `union_ex` overlap
workaround. Neighboring generic `closing*` helpers remain deferred; the project
consumer calls the source-owned `offset2_ex` sequence directly. Their
`ClipperUtils.hpp:400-410` and `ClipperUtils.cpp:592-610` ranges are
context-only.

The private `project_slice::closing` stage associates each print-object plan
with its resolved 3MF object by `source_object_index` and consumes only that
object's effective `slice_closing_radius`. It preserves the exact `f64` Option
to `f32`, widened scale division, then `f32` delta chain. The KSR fixture
resolves `slice_closing_radius=0.049` and normal scale, yielding
`+49000/-49000` and `offset2_ex(..., Miter, 3.0)`. Zero or f32-underflow radii
move the owned records unchanged; invalid external values and scaled overflow
are rejected at the Option boundary. Archive mutations cover process-base and
object-override precedence, large-bed scale, and non-integer float values.
Synthetic owned-stage vectors separately cover reversed object association,
empty layers, and metadata retention without a fixture identity branch.

The complete KSR post-closing encoding is 1,644,681 bytes with SHA-256
`29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919`:
one object, one volume, 460 layers, 2,890 contours, 395 holes, and 99,212
points. Native and browser executions are byte-identical and repeatable, both
committed fixtures remain unchanged, and all focused, full native, WASM,
browser, code-quality, default-model, and independent six-dimensional review
gates pass. Production deliberately still returns `ProjectSlicingIncomplete`;
no placeholder or reference-derived G-code is emitted, so complete normalized
KSR parity is not claimed.

### Task 22H: post-closing largest-contour selection

Task 22H is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports the direct consumer in
`TriangleMeshSlicer.cpp:2025-2037`, the selector in
`ExPolygon.cpp:532-549` / `ExPolygon.hpp:493-497`, and signed polygon area in
`Polygon.cpp:52-69`.

The private `geometry::polygon` implementation preserves the source's serial
signed `f64` shoelace order and positive zero for fewer than three points. The
private `geometry::expolygon` selector ranks contour area only, starts at zero,
uses strict `>` so the first positive tie wins, and moves the complete selected
ExPolygon with its ordered holes. Multiple nonpositive candidates remain an
internal invariant failure. The private `project_slice::largest_contours`
stage mutates each post-closing object, volume, and layer independently and
only when the retained mode is `PositiveLargestContour`. It parses no Option:
the mode and spiral bottom boundary remain assembled from the resolved 3MF by
Task 22E, and Task 22I simplification has not yet run.

The committed KSR project remains all Regular at this stage. Its exact H
checkpoint is 1,644,681 bytes with SHA-256
`e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163`:
2,890 contours, 395 holes, and 99,212 points; only the checkpoint magic differs
from Task 22G. A complete 3MF mutation of `spiral_mode`,
`bottom_shell_layers`, and `bottom_shell_thickness` enters with mode histogram
`2/0/0/458` and 337 multi-ExPolygon PLC layers, then produces 427,465 bytes,
SHA-256 `a0df3397e498306bfcade84b03721fe345d2f4b501e578a5b54df39faff44353`,
470 contours, 13 holes, and 25,747 points. An independent threshold-21 3MF
mutation selects 336 layers from slot 21 onward while preserving Regular slot
20 and produces 674,201 bytes, SHA-256
`4b64a4e70bfceabf414572f6dbe13903245612908cbaf2d12985b6c1ed440214`,
569 contours, 127 holes, and 41,012 points. These archives change Options only
inside the complete 3MF and prove there is no fixture or fixed-layer branch.

Task 22H passes focused and full native, WASM/browser, structural, code-quality,
default-model, and independent six-dimensional review gates. Both committed
fixtures remain unchanged. Production executes the selector and still returns
`ProjectSlicingIncomplete`; no placeholder or reference-derived G-code is
emitted, so complete normalized KSR parity is not claimed.

### Task 22I: resolution-driven per-ExPolygon simplification

Task 22I is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its direct stage boundary is
`PrintConfig.hpp:1554-1562`, `PrintConfig.cpp:5172-5179`,
`PrintObjectSlice.cpp:166-177`, `TriangleMeshSlicer.hpp:37-48`, and
`TriangleMeshSlicer.cpp:2025-2044`. Closed-loop Douglas-Peucker and Boolean
repair come from `ExPolygon.cpp:223-259`, `MultiPoint.cpp:164-230`,
`MultiPoint.hpp:94-99`, `Line.hpp:41-76,155-188`,
`ClipperUtils.cpp:1019-1030`, and the cited Clipper 6.4.2 strict-state closure
in `clipper.hpp` / `clipper.cpp`.

The only new runtime input is the already resolved
`views.full.process.print.resolution` value from the 3MF. Values at or below
`0.001` return before geometry traversal. Larger values select fixed
`0.0025 mm`, evaluated as `f64` division by the existing coordinate-scale
factor, narrowed to `f32`, then promoted to `f64`. The exact tolerance is
therefore `2500.0` at Normal scale and `250.0` at LargeBed scale. The private
project stage runs after Task 22H for every retained slicing mode and walks
object, volume, layer, then each source ExPolygon independently. It changes
only each layer's ExPolygon vector; plans, volume identity, layer mode, source
order, and empty records remain owned and ordered.

`geometry::simplification` owns finite-segment distance, iterative closed-loop
Douglas-Peucker, and contour-before-source-order-holes orchestration.
`geometry::clipper::simplify`, `strictly_simple`, and `output::simple` extend
the released Clipper rewrite with the required strict pass, top-edge/maxima
state, duplicate-point splitting, and ownership repair. One source ExPolygon
enters exactly three ordered NonZero unions: a mandatory StrictlySimple Paths
pass, a mandatory non-strict Paths pass, then a non-strict PolyTree pass only
when the second pass is nonempty. Outputs from one source ExPolygon are
appended contiguously before the next source ExPolygon; siblings are never
merged, sorted, canonicalized, or recovered through a fallback.

`ClipperOptions::strictly_simple` defaults to false, preserving released Task
22F/G/H behavior. The strict Paths pass performs the upstream maxima/touch
state machine and `DoSimplePolygons`; dependent `FirstLeft` repair is performed
only when PolyTree output needs ownership. The implementation introduces no
second geometry engine, unsafe/FFI, filesystem, process, thread, native-only
dependency, or platform branch. Tests use real Rust modules, every Rust source
and test file remains below 400 LOC, and the non-default WASM feature exposes
only post-H input and post-I output checkpoint hooks.

The committed `resolution=0.012` project produces a 999,721-byte I checkpoint,
SHA-256 `0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef`,
with 2,890 contours, 395 holes, and 58,902 points. Exactly layer slots 0 through
259 change. A complete `resolution=0.001` 3MF mutation is a marker-only
1,644,681-byte H-to-I identity; `0.0011` is byte-identical to the committed
enabled output. The three-Option Task 22H archive becomes 275,433 bytes,
SHA-256 `022cc958a38d5654e0a5fc4e2ca44d5e5ef068b7e57b271cb14151b11005343e`,
with 470 contours, 13 holes, and 16,245 points. Native and real Chromium runs
reach exact EOF and agree on hashes, counts, ownership, and repeatability.

Production still returns `ProjectSlicingIncomplete`; Task 22I emits no
placeholder or reference-derived G-code and does not claim normalized KSR
parity. Raw `resolution` consumers in brim, fill, perimeter, arc fitting, and
G-code remain deferred, as do cross-ExPolygon/cross-volume negative and
modifier composition, regions, surfaces, perimeters, fill, supports,
toolpaths, G-code assembly, metadata, and post-processing. Task 22J must begin
with a separately approved source-cited slice of the adjacent
`PrintObjectSlice.cpp` volume-to-region composition boundary.

### Task 22J: single-range volume region composition

Task 22J is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The owning upstream boundary is
the volume-region data and graph construction in
`Print.hpp:44-48,102-120,216-305,423-427` and
`Print.hpp:516-519,553-555,585-590`,
`PrintApply.cpp:342-405,542-553,582-592,699-724` and
`PrintApply.cpp:887-910,958-1057,1727-1739`, and
`PrintObject.cpp:3555-3710`; composition and its direct caller are
`PrintObjectSlice.cpp:21,231-241,269-480,1149-1192`. Boolean ownership and
closing follow `ClipperUtils.hpp:400-410` and
`ClipperUtils.cpp:550-584,640-667,737-803`, while the retained Internal
surfaces follow `Surface.hpp:9-47`, `SurfaceCollection.hpp:65-81`, and
`Layer.hpp:33-48,335-341`.

The current Rust boundary deliberately accepts only the one implicit
`[0, DBL_MAX)` layer range assembled from the loaded 3MF and resolved typed
Options; a nonempty explicit range chain is rejected. No new Option, default,
or external input is introduced. Physical carriers use stable nonzero volume
occurrence IDs, while graph traversal remains in source-volume order. Bounds
come from each transformed source mesh, the region registry preserves
first-created equality order, support volumes are excluded, negative volumes
remain regionless, and modifier regions are resolved only from their selected
parent region plus that volume's 3MF overrides.

Every accepted physical carrier keeps a complete occurrence-keyed slice
sidecar. The output is dense over every planned layer and every registered
region, including explicit empty slots. The single-model-part path transfers
geometry directly. The complex path partitions modifiers with NonZero
Intersection and Difference, subtracts later model parts and negative volumes,
uses the source validity order plus stable `(region_id, occurrence_id)` order,
and applies one source-compatible `offset2_ex(+delta, -delta, Miter, 3.0)`
closing when multiple records append to the same region. Each resulting
ExPolygon becomes one Internal surface with tag 4 and defaults
`thickness=-1`, `thickness_layers=1`, `bridge_angle=-1`, and
`extra_perimeters=0`; top-empty-layer removal has not yet run.

The Boolean wrappers execute Difference or Intersection into Paths, then
rebuild ownership through a fresh NonZero Union PolyTree. Coordinate-range
failures cross the public slicing boundary as the single exact region
composition `InvalidInput` error. The implementation remains crate-private,
safe Rust with real modules and no filesystem, process, thread, native-only,
fixture-reading, or platform-specific path. Default WASM exports no Task 22
hook; the non-default browser test feature exposes only the Task 22J input and
output checkpoints.

The committed KSR archive produces a repeatable 2,008,706-byte J checkpoint,
SHA-256 `2b474697f4afae95c9a55d709d8740d382a80b2969fc5118dc89e13c1906162d`:
one object, 460 planned layers, occurrence `[1]`, 460 sidecar layers, 460 dense
retained layers, one region per layer, 2,890 ExPolygons, 395 holes, and 58,902
points in both the sidecar and retained geometry. A complete 3MF modifier
archive and its no-override control share the exact 478-byte Task 22I input
but produce distinct repeatable J checkpoints of 1,054 and 698 bytes, proving
that region composition consumes the loaded volume Option instead of fixture
identity. Native Rust and fresh Chromium reach exact EOF and agree on the full
registered bytes and parsed ownership.

Task 22J stops before `PrintObjectSlice.cpp:1194-1203` top-empty-layer removal.
Multi-range chains, material and painted segmentation, the ten neighboring
usage gates, support-region graph construction, top trimming, conical
overhang, XY and elephant-foot compensation, `make_slices`, surface
classification, perimeters, fill, support generation, toolpaths, G-code,
metadata, and post-processing remain deferred to separately approved upstream
slices. Production still returns `ProjectSlicingIncomplete`; no placeholder or
reference-derived G-code is emitted, and normalized KSR G-code parity is not
claimed.

### Task 22K: post-region top-empty-layer removal

Task 22K is implemented from fixed OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The owning upstream boundary is
`PrintObjectSlice.cpp:1194-1201`, which repeatedly deletes only the final layer
while it is empty, and `PrintObjectSlice.cpp:1202-1203`, which clears the new
final layer's upper pointer. `Layer.cpp:21-29` defines a layer as empty only
when every present region has an empty slice collection, while
`SurfaceCollection.hpp:49-51` defines collection emptiness solely by
`surfaces.empty()`, independent of polygon area.

The Rust stage reverse-searches each `PostRegionPrintObject` for the final
layer whose any region owns at least one surface, then truncates the planned
layers and every dense region-layer vector to that identical prefix. Leading
and interior empty layers remain, surviving IDs are not renumbered, a surface
containing an empty `ExPolygon` keeps its layer, zero-region and all-empty
objects retain zero layers, and occurrence-keyed volume sidecars remain
complete. Ares has no layer adjacency pointers; retaining only the dense prefix
is the Rust equivalent of deleting the suffix and clearing the surviving final
`upper_layer`.

Native tests fix the ten-object synthetic K checkpoint at 5,848 bytes, SHA-256
`037b5e1b5aa9eb2f5c9c38f00a8d7a23768217fd7cc7ec13bb71f21d9edb3b07`:
only object 9 loses its empty final retained layer, while both of its two-layer
sidecars remain complete. The committed KSR checkpoint is 2,008,706 bytes,
SHA-256
`c101e0f9ff863c7abe72cd1cb792fcd8e0074d8d6d2e77d3bb56c32eedba13be`;
all bytes after the eight-byte `ARES22K` magic equal the released Task 22J
stream, and all 460 layers remain. Real-loader top- and bottom-negative-slab
3MF vectors independently prove `[nonempty, empty] -> 1` and
`[empty, nonempty] -> 2`, with ordered occurrence sidecars `[(1, 2), (2, 2)]`
unchanged.

The browser J/K known-answer checkpoints are respectively 433 bytes /
`940f01934309cf1a23afe67e7d8365ced3e9f8296f8ee4db73261aac74e71a6a`
and 385 bytes /
`a49fcd311d79d216d874c585ae107f33a178fd47e99d3f862475295d0e237751`.
The top and bottom archive semantic-entry digests are respectively
`36f49fc5ad0788dc63ce9e25111d5d758c67711137d368dc63eb76c5aee1e538`
and
`2001de693fbcc3781d733beebc8ace871cc42a2abe47865c51159192b9a94817`.
Two fresh Chromium passes reach exact EOF, reproduce the opposite trim
decisions and complete sidecars, and agree with the exact KSR J/K identities.
Default WASM exposes no Task 22 hook; the non-default feature exposes exactly
`task22kBrowserInputOracle` and `task22kBrowserOracle`.

Task 22K introduces and consumes no Option. Public `slice_project` executes the
stage and still returns `ProjectSlicingIncomplete`; the `ARES22K` stream is a
test checkpoint, not public G-code. Cancellation at
`PrintObjectSlice.cpp:1204`, the adjacent `apply_conical_overhang` call at
`PrintObjectSlice.cpp:1206` and implementation at
`PrintObjectSlice.cpp:1394-1509`, material and painted segmentation,
compensation, surface classification, perimeters, fill, supports, toolpaths,
G-code assembly, metadata, and post-processing remain deferred. The next
source audit must start at the `1204-1206` caller sequence, classify
cancellation separately, and bound `apply_conical_overhang` together with its
`make_overhang_printable*` Option ownership before another implementation
slice is approved.

### Task 22L: conical-overhang region projection

Task 22L ports the uncancelled success path from fixed OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The caller is
`PrintObjectSlice.cpp:1204-1206`, the implementation is
`PrintObjectSlice.cpp:1394-1509`, and `Layer.cpp:117-136` defines the four-field
merged layer footprint. The caller cancellation check at line 1204 and the
per-layer-pair check at line 1421 are classified but deferred because Ares has
no public cancellation contract; the slice adds no no-op callback or test-only
production control plane.

The stage consumes only resolved values loaded from the supplied 3MF. Object
Options provide `make_overhang_printable_angle`,
`make_overhang_printable_hole_size`, and nominal `layer_height`; each ordered
region provides `make_overhang_printable`, `bottom_shell_layers`,
`top_shell_layers`, `sparse_infill_density`, and `wall_loops`. Every object is
validated in vector order, angle before hole size, before any object is
mutated. No fixture name, digest, reference G-code, rectangle fallback, or new
production default participates in the result.

The Rust arithmetic preserves the source conversion points:
`epsilon_scaled = f32(0.0001 / scale_factor)`, angle and tangent are evaluated
in `f64`, `distance_scaled` is
`-f32(tan(angle_radians) * layer_height / scale_factor)`, and the protected-hole
threshold is `f32(hole_size / scale_factor / scale_factor)`. Adjacent layer
pairs run in reverse. Each upper and current layer uses the complete merged
footprint; eligible small holes are protected, the upper footprint is offset
with Miter join and limit 3, and enabled regions take ownership in existing
vector order. Cross-region removal uses the fixed per-path 10-coordinate
safety offset. Rebuilt affected collections contain Internal surfaces with the
source default metadata tuple, while the layer plan, sidecars, skipped layers,
and unaffected surface metadata remain unchanged.

The fixed Orca oracle's 40 ordered synthetic cases, together with its stepped
and KSR binary and text outputs, remain byte-identical across two runs. Ares has
53 focused Task 22L tests plus the released Task 22K suite.
The native stepped disabled/enabled archives are respectively 181,446 bytes /
`ee928a255109b491b0640da279b86d9282c573ec49a400e3cc4529eac915030e`
and 181,447 bytes /
`be286d7abb2bef8ab5e8b650657b114ea35c4dcff3a1463eba1a0dd278a89faa`;
their semantic streams are 1,020,460 bytes /
`ade484830a6492b50c3233e51debf5eab1db7d3e3bbf81fa8cd72f10226ea9ef`
and 1,020,460 bytes /
`f61089d040d1edf002f1dedca66b433e4982e18b9ce69a6385aa42dbf4c780b9`.
Both share the 490-byte K checkpoint
`c6668cfbc56b20abe71606d59d2e28abf08ebb8b22f3ecebb3058d63ba05b44f`;
their L checkpoints are 490 bytes /
`0834c61cc48aece1afd52d060c5c2a58f7243124664ad0a7dd3f500d6735b790`
and 554 bytes /
`33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505`.
The independent ten-object L transition is 5,848 bytes /
`fe46d60251dcf95590c71a3e55cafdf81e0fc6af5b3cb95d58d6c39ea693b264`.

The committed KSR project's disabled L checkpoint is 2,008,706 bytes /
`7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`.
Fresh fflate browser archives are 190,380 bytes /
`c4c0ea05709a6fadd8b2d0d6d34dab1cad5420865c5993b58b9d8e91a8f73313`
and 190,381 bytes /
`130260c5c63846759aa66d25e68ff9bb07cf5aeec86ef7da9476c12761f3836d`.
Two fresh Chromium passes reproduce the archive, semantic, K, L, lower-only
geometry change, unchanged upper/plan/sidecars, exact EOF, and repeatability
contracts. Default WASM exposes no Task 22 hook; the non-default feature
exposes exactly `task22lBrowserInputOracle` and `task22lBrowserOracle`.

Public `slice_project` executes Task 22L and still returns
`ProjectSlicingIncomplete`; `ARES22L` is a test checkpoint, not G-code, and
normalized KSR G-code parity is not claimed. Cancellation remains deferred
until a public control plane exists. The next source audit starts at
`PrintObjectSlice.cpp:1208-1225`: filament count and
`mmu_segmentation_facets`, the XY-compensation warning, and
`apply_mm_segmentation`. Fuzzy segmentation at lines 1227-1241, interlocking at
line 1243, `make_slices` at line 1246, compensation, surface classification,
perimeters, fill, supports, toolpaths, G-code assembly, metadata, and
post-processing remain separate source-cited slices.

### Task 22M: single-region make_slices and elephant-foot compensation

Task 22M ports the uncancelled single-region success path from fixed
OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The owning caller boundary is `PrintObjectSlice.cpp:1246-1276,1287-1292` and
`1364-1387`; `Layer.cpp:38-66` / `Layer.hpp:123-178` own island extraction and
ordering, while `ElephantFootCompensation.cpp:20-28,233-447,465-532,544-644`
and `EdgeGrid.cpp:28-334` / `EdgeGrid.hpp:15-356` own the geometry kernel and
spatial index. The parallel scheduler and cancellation checks are classified
but deferred; deterministic sequential execution preserves uncancelled output.

The stage consumes only effective Options resolved from the supplied 3MF:
`elefant_foot_compensation`, `elefant_foot_compensation_layers`, `raft_layers`,
the zero-only XY compensation pair, initial/regular/external line widths,
external-perimeter filament selector, nozzle diameters, and each planned layer
height. Percent widths use the selected nozzle directly, without applying
`filament_map`; invalid raw values, nonpositive Flow spacing, nonzero XY, and
valid nonempty multi-region input fail before any mutation. The source f32
compensation ramp, conversion points, coordinate-scale round trips, signed-zero
selection, width fallback order, and strict comparisons are preserved.

Production builds the complete `EdgeGrid` with two-pass raster/count/fill and
box traversal. The fixed oracle intentionally uses a full segment scan, so it
can compare identical geometry without sharing the grid's failure modes. The
kernel compensates each ExPolygon independently, then performs the exact
two-pass NonZero union. A direct one-pass mutant produces
`[right, left, nested]` instead of the required `[left, nested, right]` and is
killed by the synthetic matrix. No rectangle shortcut, fixture branch, or
broad identity fallback is present.

The orchestration wrapper owns both the post-region object and one ordered raw
`lslices` vector per planned layer. Disabled and raft paths still run
`make_slices`; enabled layers replace surfaces with default Internal metadata
while retaining the uncompensated backup. Plans, volume sidecars, region ids,
and unaffected layers remain unchanged. The `ARES22M` checkpoint appends each
layer's raw `lslices` immediately after its retained regions and is not a public
format.

The 19-case synthetic aggregate is 10,351 bytes / SHA-256
`c112246ff48b280eb803082749d74315e771d073b0407e45afde536e37fcf46d`.
The committed KSR L input remains 2,008,706 bytes /
`7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`;
its M output is 3,008,346 bytes /
`91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`.
Rust 1.91 passes 81 Task 22M tests, 53 Task 22L tests, and all 509 Task 22 tests.
Fresh default WASM exposes no Task 22 hook; the feature build exposes exactly
`task22mBrowserInputOracle` and `task22mBrowserOracle`. Two fresh Chromium runs
each pass all five parser, Option-only archive, public lifecycle, and complete
KSR contracts.

Public `slice_project` executes Task 22M and still returns
`ProjectSlicingIncomplete`; `ARES22M` is a test checkpoint, not G-code. Painted
MMU and fuzzy segmentation, interlocking, nonzero XY algorithms, multi-region
compensation and safety union, classification, perimeters, fill, supports,
toolpaths, G-code, metadata, and post-processing remain deferred. For the
active KSR path, the next source audit begins at `PrintObject.cpp:452-560`,
`PrintObject::make_perimeters`, and the called `Layer::make_perimeters`
boundary. That audit must first prove from the 3MF that the skipped
`PrintObjectSlice.cpp:1208-1243` segmentation/interlocking gates remain
inactive; activated variants require their own source-cited slices.

### Task 22N: single-region perimeter inputs and Flow dispatch

Task 22N ports the input-preparation seam reached by the KSR project from fixed
OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The owning call graph is `PrintObject.cpp:453-558`,
`Layer.cpp:185-225`, `LayerRegion.cpp:21-58,82-142`,
`PrintRegion.cpp:7-54`, `PrintObject.cpp:3562-3565,3602-3661,3694-3700`,
`Flow.cpp:20-35,129-143,146-229`, `Flow.hpp:16-25,52-139`, and the input
contract in `PerimeterGenerator.hpp:73-141`. The Rust destination is the
crate-private `project_slice::perimeters` module. It stops at the exhaustive
Classic/Arachne dispatch and does not call either perimeter process body.

Preparation is transactional across all objects. It preserves each complete
post-M object and creates one optional record per planned layer. A present
record owns object/occurrence/layer/region identity and indices into the
unchanged current, lower, upper, and upper-same-region geometry; accessors
resolve complete ordered surface collections rather than selecting one
surface. Empty surface collections retain their M state but have no record.
The record also carries exact layer height and slice Z, four Flow values,
spiral state, model rotation, and dispatch.

External, internal, and solid-infill Flow resolution uses the effective
one-based feature-filament selector loaded from the 3MF, subtracts one only at
the nozzle lookup, retains element-zero vector fallback, and never applies
`filament_map`. Initial-layer width wins only when positive on layer zero;
role width then object `line_width`, then the fixed `1.125f * nozzle` automatic
width are the reached fallback order. Percent values use the selected f32
nozzle. Spacing, bridge circular area, rounded-rectangle area, and
`mm3_per_mm` preserve the fixed f32 narrowing and float-to-double promotion
sites.

Overhang Flow uses the internal-perimeter selector. Thick mode constructs the
fixed circular bridge after multiplying diameter by `sqrt(bridge_flow)`;
nonthick mode starts from ordinary internal Flow and follows the exact
`with_cross_section` grow-height, canonical increase-else, shrink-width, round,
and epsilon branches. The increase-else branch reuses the old f32 area divided
by height and rebuilds width and spacing instead of asserting that the proposed
full spacing grew. The shared Task 22M ordinary-Flow constructor remains
spacing-only, so a metadata-valid `1e-30` height/width predecessor still reaches
its released minimum-width behavior even when stored volume underflows to zero.
Task 22N validates every ordinary and overhang record's final positive volume
before state consumption. All raw widths, nozzles, planned heights, and
`bridge_flow` values are preflighted; nonpositive/nonfinite ratios and positive
ratios whose thick or nonthick result underflows are attributed to the
`bridge_flow` Option rather than accepted as a zero-area Flow.
The fixed-release decrease path matches Orca with assertions disabled: the
debug-only intermediate-width assertion is not a runtime contract, so a
metadata-valid nozzle `100`, width `500%`, height `2e-7`, and
`bridge_flow=f64::MIN_POSITIVE` reducer reaches the same zero Flow and is
reported by the existing Task 22N boundary as `invalid Orca option
bridge_flow`. Pure Flow resolution, a real in-memory 3MF through public Rust,
and a generated real-archive Chromium case freeze the exact error without a
Rust panic or WASM trap. This expected-error self-check is separate from the
unchanged 25-object success aggregate.

Spiral state requires `spiral_mode`, `layer_id >= bottom_shell_layers`, and
`print_z >= bottom_shell_thickness - EPSILON`. Model rotation is zero unless
`align_infill_direction_to_model` is enabled, then uses the matching
occurrence's stored `atan2(m10, m00)` inputs, including signed zero. Arachne is
selected only for a nonspiral Arachne request; Arachne plus spiral dispatches
Classic by the fixed branch, not by a compatibility fallback.

Task 22O.1 removes the opaque tracked 25-object binary aggregate. Readable
behavioral builders retain Task 22N parser framing, semantic validation,
corruption, truncation, and trailing-byte coverage without embedding a binary
or pinning Orca source text. For the committed KSR archive, the complete
predecessor M wire remains
3,008,346 bytes /
`91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`;
the complete 460-record N wire is 7,083,888 bytes /
`42e0053bffb3093a44597abd0a2b4e8b8c8c11d6f07003cb894399ad7dce3c6e`.
The original Task 22N real-3MF matrices covered 19 Flow Option pairs and six
context pairs, including raw/effective selector normalization, scoped fallback,
two nozzles, every reached bridge branch, an anti-`filament_map` swap, spiral
gates, alignment/signed-zero transforms, and generator dispatch. A dedicated
single-delta archive reducer changes only `bridge_flow` from `1` to
`1.0000001`, preserves M, produces two populated N slots, and freezes the
canonical increase-else bits; native and browser reducers also freeze the
Task 22M volume-underflow predecessor and both tiny-positive bridge modes.
Default WASM exposes no Task 22 hook; the non-default build exposes exactly
`task22nBrowserInputOracle` and `task22nBrowserOracle`. Strict composite N/M
parser KATs run before fixture fetch, and optimized Chromium verifies the exact
KSR wire, repeatability, all Option families, and the public lifecycle.
Final local gates pass 45 Task 22N tests, 82 Task 22M tests, all 555 Task 22
tests, 5,191 complete `ares-core` tests with one configured skip, and 5,227
workspace tests with two configured skips. Two fresh Chromium runs each pass
all nine contracts.

Public `slice_project` now executes this preparation and still returns
`ProjectSlicingIncomplete`; `ARES22N` is a test checkpoint, not G-code. The
Classic and Arachne process bodies, perimeter loops and extrusion entities,
precise-spacing behavior, dynamic top-one-wall behavior, overhang splitting,
smaller-width external loops, perimeter gap generation, multi-region merging,
fill, supports, toolpaths, G-code, metadata, and post-processing remain
deferred. Perimeter gaps must not be suppressed by
`gap_fill_target=nowhere`: fixed `PerimeterGenerator.cpp:1192,1325-1332,
1573-1624` enables them from `gap_infill_speed > 0`; KSR sets that speed to
250 and its reference contains 470 Gap infill feature blocks.

Task 22O is implemented as serial source-cited slices beginning at
`PerimeterGenerator::process_classic()`. Task 22O.1 ports the pre-onion prefix
through transactional capability validation, exact Flow-derived prelude
arithmetic, smaller external Flow reconstruction, lower-support growth and
sample series, counterbore-none behavior, arc-aware simplification,
bounding-box-center surface ordering, and loop-count preparation. Its Rust
boundary is `project_slice::perimeters::classic`, with reusable integer bounds
in `geometry::bounding_box`; the public project lifecycle consumes this state
and remains intentionally incomplete.

The earlier Package-A0 qualification/recovery chain is historical,
non-blocking audit evidence and is not a prerequisite for Rust production
behavior. It is not retried and does not establish Task 22O completion. Task
22O.1 stops before `split_top_surfaces()` and the onion loop. Dynamic top
splitting, onion offsets, hierarchy/traversal, overhang path splitting, medial
axis gaps, variable-width extrusion, fill remainder, seams, infill, motion,
G-code, metadata, post-processing, and exact KSR G-code parity remain deferred
source slices.

Task 22O.2 advances the fixed upstream boundary to
`PerimeterGenerator.cpp:574-660,1235-1306,1343-1385`. It ports the complete
dynamic top split and only the non-thin-wall first external offset required by
the exact caller. `project_slice::perimeters::classic::top_split` owns the
unchanged Task 22O.1 predecessor and resolves `wall_loops`,
`only_one_wall_top`, `interface_shells`, `min_width_top_surface`,
`sparse_infill_line_width`, outer-nozzle selection, and gap enablement from
effective typed 3MF configuration transactionally before Clipper geometry.
Percent bases, zero infill-width auto resolution, mixed fixed/float casts, and
the upstream bbox vertex prefilter are part of this boundary.

Later onion loops, materialized perimeter entities, hierarchy and traversal,
thin walls, active multi-region behavior, later bridge surface kinds, gap
masks, overhang splitting, fill remainder, seams, infill, motion, writer and
post-processing remain source-cited future slices. Task 22O.2 is not complete
Task 22O and is not an independently designed Ares pipeline stage.

Task 22O.3 advances only the raw onion loop-back boundary at fixed
`PerimeterGenerator.cpp:1304-1387`. The Rust destination
`project_slice::perimeters::classic::onion` nests the immutable Task 22O.2
object and transactionally resolves typed effective `sparse_infill_density`,
including the source conversion to its local `int`, while reusing predecessor
gap enablement and integer spacing. It preserves the source f64-to-f32 delta
casts, fixed-coordinate `-1`, `+1`, and `10` terms, ordered gap append before
collapse/extra-pass termination, effective loop reduction, raw depth geometry,
final `last`, and the positive converted-density gap-only iteration. Task 22O.2 remains the sole owner of depth zero and dynamic top
splitting.

Task 22O.4 advances the fixed boundary through `PerimeterGenerator.cpp:34-55,
1353-1369,1388-1443`, with containment from `Polygon.hpp:66`,
`Polygon.cpp:722-729`, and Clipper v6 `PointInPolygon`. The Rust destination
`project_slice::perimeters::classic::hierarchy` nests O3, materializes only its
ordered raw shells, and preserves exact boundary-inclusive, first-point,
depth/index first-parent and destructive erase/retry behavior. Roots and
unaltered diagnostic leftovers retain source order.

Task 22O.5 advances the source prefix at `PerimeterGenerator.cpp:100-151` and
`PerimeterGeneratorLoop::is_internal_contour` at `2537-2547` into
`project_slice::perimeters::classic::traversal`. It transactionally nests O4
and builds ordered seed trees from roots only, retaining exact extrusion and
loop roles, smaller/external/internal predecessor flow and lower-series routes,
`f32` width, source `f64` layer height and `f64 mm3_per_mm`, inactive overhang
reversal provenance, and the read-only pending predicate from line 158. O4
diagnostics remain nested and are not traversal input.

No fuzzy mutation or pending path branch executes. Support clipping,
`intersection_pl`/`diff_pl`, extrusion entities, actual recursive entity
traversal, thin walls, active overhang reversal, wall ordering, gaps/fill,
seams, infill, motion, writer/post-processing, complete Task 22O, and exact KSR
parity remain deferred. This is a bounded upstream rewrite slice, not an
Ares-owned perimeter algorithm.

Task 22O.6 ports the exact open-path dependency boundary from fixed Clipper v6
`clipper.cpp:756-949` and all output-affecting `IsOpen` branches through
PolyTree construction/extraction, together with OrcaSlicer
`ClipperUtils.cpp:835-934`. The Rust destination `geometry::clipper` now uses a
single `Clipper` engine for closed and open subjects, rejects open clip paths
and flat output for open input, preserves zero-winding scanline, horizontal,
maxima, fixup and root-record behavior, and exposes source-order polygon
`intersection_pl`/`diff_pl` with exact closure and destructive recombination.
Closed topology/order and the active `f64` determinant remain unchanged.

This dependency slice does not fabricate a traversal result. Its open clipping
APIs are consumed by O7 below.

Task 22O.7 ports the reached raw-path construction from fixed
`PerimeterGenerator.cpp:153-207,218-224`, reached
`ExtrusionEntity.hpp:153-188,551-580`, and `Polyline.hpp:291-302` into the
crate-private `project_slice::perimeters::classic::materialize` boundary. Its
aligned successor nests O5, dispatches only on the O5 pending branch, derives
source `SCALED_EPSILON` from the prepared coordinate scale, borrows the
route-selected final lower series, uses O2 bbox filtering and O6 intersection
then difference ordering, and preserves exact fixed-coordinate fragment order,
role, flow, width, and height provenance. The narrow local extrusion types are
intentional because the unrelated public legacy scaffold is floating 2D and
does not represent this reached source seam.

O1 proves fuzzy skin inactive and rejects active `overhang_reverse`; therefore
O7 consumes unchanged O5 polygons and does not model unreachable fuzzy,
steep-overhang, or reverse branches at lines 153-177. Public slicing now
executes O7 before remaining `ProjectSlicingIncomplete`.

Task 22O.8 ports `PerimeterGenerator.cpp:208-210,227`, the reached
all-paths-reversible `ShortestPath.cpp` greedy multi-fragment specialization
with its exact `KDTreeIndirect.hpp` and `MutablePriorityQueue.hpp` behavior,
and the reached `ExtrusionLoopRole` / `ExtrusionLoop` ownership boundary from
`ExtrusionEntity.hpp`. O8 consumes O7 path vectors and point buffers zero-copy,
retains the unchanged boxed O5 predecessor, applies the empty `continue` and
start-near chaining only to overhang-clipping records, and exhaustively maps
loop roles into crate-private loop compatibility shells. Transformation and
terminal ownership are iterative. Public slicing now executes O8 before the
same `ProjectSlicingIncomplete` boundary.

O8 intentionally does not pre-orient loops: upstream orientation occurs only
after entity selection and recursive traversal.

Task 22O.9 ports fixed `PerimeterGenerator.cpp:230-280`, caller setup/call
`1443-1450`, reached `ShortestPath.cpp:1026-1040`, and exact
`ExtrusionEntity.cpp:141-170` loop orientation into crate-private
`project_slice::perimeters::classic::entity_collections`. It moves O8 loops
into flat ordered collections, chains each local loop group from zero, applies
typed per-region wall direction, preserves the source lone-hole rule and
contour/hole emission order, and sets source `inset_idx`. Traversal and cleanup
are iterative and retain the same boxed O5 predecessor.

The line-208 `continue` compacts source entities while lines 240-250 index the
original loop array. O9 deliberately reproduces that indexing rather than
adding a survivor map. Active thin walls remain unreachable because O1 rejects
`detect_thin_wall=true`; no heterogeneous entity abstraction is fabricated.
Active overhang reversal, gaps/fill, seams, infill, motion, G-code,
writer/post-processing, complete Task 22O, and exact final KSR G-code parity
remain deferred.

Task 22O.10 ports fixed `PerimeterGenerator.cpp:1451-1569` into
crate-private `project_slice::perimeters::classic::perimeter_append`. O1's
transactional preflight makes overhang reorientation, non-`InnerOuter` wall
ordering, and active layer-zero outer-brim reversal unrepresentable at this
stage. O10 retains typed inactive operands and false-reason provenance without
implementing a fallback. It executes the remaining source behavior by moving
each nonempty O9 flat collection into one nested perimeter collection and
omitting empty collections, while preserving order, allocations, and the boxed
O5 predecessor. Gap fill beginning at line 1573 and all downstream behavior
remain deferred.

Task 22O.11 ports fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PerimeterGenerator.cpp:1573-1581,1583-1585`, into crate-private
`project_slice::perimeters::classic::gap_domain`. It borrows O3 gaps and O1
prelude widths/resolution through O10's boxed O5 predecessor, stages all
fallible opening/offset/difference geometry transactionally, then moves every
O10 collection and inactive provenance into aligned O11 surfaces. Empty gaps
are typed `None`; nonempty results retain exact bounds and source-ordered,
in-place simplified ExPolygons. The boxed O5 allocation and nested O10
allocations remain unchanged. Both success and error cleanup consume retained tree structures iteratively. Public slicing reaches O11 and remains
`ProjectSlicingIncomplete`.

O11 is only the upstream pre-medial prefix, not an Ares-designed pipeline. The
next boundary starts at `PerimeterGenerator.cpp:1586` with
`ExPolygon::medial_axis` and its actual ThickPolyline prerequisites. Medial
axis, gap extrusion, downstream G-code, final KSR parity, and Orca end-to-end
comparison remain deferred.

Task 22O.12 ports fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, `Line.hpp:15-19,202-212`,
`Polyline.hpp:14-17,256-287`, and `Polyline.cpp:637-679` into crate-private
`geometry::{line,polyline}`. It fixes the source ThickLine/ThickPolyline data,
two-widths-per-segment invariant, reversal, clear, closed rotation, and fixed
width conversion required by the medial-axis output contract. Rust flattens
C++ Polyline inheritance rather than adding a compatibility shell.

O12 is a prerequisite, not a lifecycle stage: O11 remains the public terminal
prefix and `PerimeterGenerator.cpp:1586` remains unexecuted. A partial
MedialAxis shell is rejected because source edge validation and neighbor
chaining depend directly on the Boost.Polygon Voronoi cell/edge/twin/rotation
topology. The next milestone must cite and port that actual source boundary;
it may not substitute a simplistic skeleton algorithm, dependency, or runtime
Orca oracle.

Task 22O.13 advances that source-cited boundary through fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` `ExPolygon.cpp:261-369`,
`Geometry/MedialAxis.cpp:458-707`, and the reached
`Geometry/VoronoiOffset.cpp:646-971` annotation path. A crate-private
`boostvoronoi` 0.12.1 adapter preserves source segment order and directed
half-edge topology; annotation, edge validation, chaining, endpoint extension,
short removal, and reconnect produce ordered ThickPolylines. The aligned O13
Classic stage transactionally moves O11 domains and preserves O10/O5 ownership.
Public slicing reaches O13 and intentionally remains `ProjectSlicingIncomplete`.

Voronoi `Point(double, double)` conversion follows upstream `std::round`
(half away from zero); only endpoint-extension Eigen `cast<coord_t>` sites
truncate. Reached vector norms retain the source multiply/add/`sqrt` operation
order, Voronoi vertex/site equality uses the Boost 64-ULP rule, and Boundary
cell annotation is overriding and sticky. The crate validates source bounds,
secondary twin/site relations, endpoint reversal, and bounded face/rotation
cycles without claiming the deferred completed-diagram detector. For Tier-1
browser builds, `ares-core` enables `getrandom` 0.3.4's `wasm_js` feature on
`wasm32` to qualify the transitive `boostvoronoi` dependency chain.

The upstream invalid-diagram detector/rotation/closing repair remains deferred.

Task 22O.14 advances the fixed source boundary through
`PerimeterGenerator.cpp:1604-1624`, `VariableWidth.cpp:99-234`, reached
`Flow`, `ExtrusionEntity`, `ExtrusionEntityCollection`, `ClipperUtils`, and
Clipper 6 open-butt offset behavior. The aligned typed `RegionOptions` record
supplies `filter_out_gap_fill`; a whole-project validation pass rejects
negative or non-finite values before variable-width or Clipper geometry. The
unrounded threshold is divided by the active coordinate-scale factor and the
strict source filter removes only shorter polylines without changing order.
The aligned perimeter input record supplies `solid_infill_flow`; no legacy
`SliceOptions`, fixture substitution, or `gap_fill_flow_ratio` is used.

The variable-width conversion preserves the mutable ThickLine loop, strict
scaled epsilon and tolerance comparisons, source midpoint versus asymmetric
final width averaging, normalized-vector splitting with truncating coordinate
casts, and reached `Flow::with_width` cast order. It emits fixed-coordinate
GapFill paths or loops in a separate ordered collection. Covered-width geometry
uses `float(scale_(width / 2)) + 10.f`. The Clipper rewrite now carries a
crate-private OpenButt end type: each input is prepared and Positive-unioned in
a cleared offset engine, then the ordered aggregate receives the wrapper-level
NonZero union. Consecutive short-edge removal remains strict, so equality
survives; closed-offset behavior is unchanged.

O14 first stages option validation, keep masks, entities, coverage, and every
ordinary difference for the whole project. Only after all fallible work
succeeds does it move O13/O11/O10/O5 ownership, retaining surviving medial
allocations, attaching gap entities, and cloning or subtracting the onion
`last` fill remainder. Error cleanup iteratively consumes untouched O13 state.
Public slicing reaches O14 once and intentionally remains
`ProjectSlicingIncomplete`. Tier-1 run `30900710846`, Windows job
`91964102127`, first recorded post-O20 integration aborts in the O11
closed-boolean-tree constrained-stack test (`86.033s`) and this O14
open-offset/variable-width constrained-stack test (`47.523s`): both aborted at
64 KiB with `0xc00000fd` / OS error 1001 after 4,175 preceding passes. After
raising only those two tests, exact-SHA rerun `30904949178`, Windows job
`91977766653`, advanced to O15 and exposed the same 64-KiB floor in aggregate-
union and final-top-union cleanup. Project-slice constrained-stack tests now
share one test-only baseline: 64 KiB on Unix and 256 KiB on Windows. Every
10,000-node predecessor witness and iterative cleanup assertion remains
unchanged.

Task 22O.15 advances the fixed source boundary through
`PerimeterGenerator.cpp:1628-1691`. A crate-private
`classic::infill_boundary` successor recovers aligned O3 loop counts, O2
`top_fills`/`fill_clip`, O1 spacings, original layer adjacency, and effective
typed `RegionOptions.infill_wall_overlap` and
`top_bottom_infill_wall_overlap`. Whole-project numeric preflight preserves
the source signed half operations and the full unscale/percent/scale
conversion sequence at Normal and LargeBed scales. O15 uses raw
`m_scaled_resolution`, not O1's arc-fitting-adjusted tolerance.

The sibling geometry helper implements reached `ExPolygon::simplify_p` flat
polygon output without changing the older per-expolygon grouping helper.
O15 then performs one aggregate NonZero union, ordinary collapsing, mandatory
top offset/intersection, conditional top expansion, ordered internal-surface
append, the inactive six-operand extra-perimeter guard, and the selected
no-overlap branch with exact `i64 -> f64 -> f32` casts. All fallible numeric
and geometry work is staged before moving O14/O13/O11/O10/O5 ownership, and
the public lifecycle intentionally remains `ProjectSlicingIncomplete`.

The O15 KSR structure is pinned at
`136197013209006370081121271251125478104`. The 49 focused O15 tests and geometry
regressions, 5,540 workspace Nextest tests with 2 skipped, strict Clippy,
workspace/native and both WASM checks, formatting, diff, LOC,
forbidden-pattern, dependency, and staging audits pass. The final independent
six-dimensional implementation rereview and OpenCode rereview both returned
`VERDICT: APPROVE`.

The activated `apply_extra_perimeters` body and Arachne-only helper beginning
at `PerimeterGenerator.cpp:1695` remain deferred.

Task 22O.16 advances the KSR Classic path through
`LayerRegion::make_perimeters` at `LayerRegion.cpp:82-142` and the
one-compatible-region branch of `Layer::make_perimeters` at
`Layer.cpp:185-226`. The crate-private `perimeters::layer_region` successor
moves the ordered perimeter collections, gap-fill entities, internal fill
surfaces, and no-overlap polygons out of O15, and copies ordered
`fill_expolygons` exactly like `to_expolygons(const Surfaces&)` at
`Surface.hpp:159-166`.

The source-required many-to-one append consumes Ares' artificial per-surface
wrapper-vector storage while preserving every nested collection entity,
loop/path/point, gap-loop/path/point, record-level fill/no-overlap geometry,
and boxed traversal allocation that becomes LayerRegion state. The copied
fill expolygons are value-equal but allocation-distinct from fill-surface
geometry. Existing one-region validation and Classic preflight keep the
multi-compatible-region branch and active `process_no_bridge` body deferred.

The O16 KSR checkpoint is
`-169716507603417685621692788651154411580`, with totals
`[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 1112]`. Fourteen focused O16
tests, 192 O1/O10-O16 regressions, and 5,554 workspace Nextest tests with 2
skipped pass together with strict Clippy, workspace/native and both WASM
checks, formatting, diff, LOC, forbidden-pattern, source-pinning, dependency,
and staging audits. The final independent six-dimensional implementation
review and OpenCode review both returned `VERDICT: APPROVE`.

Public slicing reaches O16 once and intentionally remains
`ProjectSlicingIncomplete`.

Task 22O.17 advances the first complete `PrintObject::prepare_infill` mutation
through `detect_surfaces_type` at `PrintObject.cpp:1520-1923` and
`LayerRegion::slices_to_fill_surfaces_clipped` at `LayerRegion.cpp:63-80`.
The crate-private successor classifies original region slices as ordered
`Internal`, `Top`, `Bottom`, and `BottomBridge`, then clears and rebuilds fill
surfaces by numeric surface-kind order against the unchanged O16
`fill_expolygons` boundaries.

The rewrite preserves integer-to-float opening arithmetic, two separately
fallible miter/3.0 opening offsets, clip-only 10-unit safety, stable contour-
then-hole path order, terminal metadata clone/reconstruction behavior, and the
pinned `ExPolygon, ExPolygon` overload quirk that discards the apparent crack-
containment safety flag. O16 perimeter, thin-fill, fill-boundary, no-overlap,
and boxed traversal allocations move unchanged; old fill surfaces are consumed
and typed slices/fills are fresh staged geometry.

Only the temporary early capability gates for `enable_support` and
`enforce_support_layers` move forward. Their typed values feed Orca's literal
automatic-support predicate, but public slicing still ends honestly at O17.
`interface_shells = true` and active external/all extra-bridge modes fail in
whole-project O17 preflight; spiral and non-`none` counterbore modes retain
earlier precedence.

The O17 KSR checkpoint is
`-126362407653399901571400348049652748978`, with totals
`[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294,
113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011]`. Forty-three focused
O17 tests, 178 O1-O17 regressions, and 5,597 workspace Nextest tests with 2
skipped pass with strict Clippy, workspace/native and both WASM checks,
formatting, diff, LOC, forbidden-pattern, source-pinning, dependency, and
staging audits. ZIP repack/non-slicing rename invariance and the exact component
X-scale/elephant-foot span relation distinguish fixture hardcoding. The final
independent six-dimensional implementation rereview and OpenCode rereview both
returned `VERDICT: APPROVE`.

Public slicing reaches O17 once and intentionally remains
`ProjectSlicingIncomplete`.

Task 22O.18 ports the slicing-state mutation in
`LayerRegion::prepare_fill_surfaces` at `LayerRegion.cpp:935-973`, called from
`PrintObject.cpp:587-592`. Each aligned record reads its typed resolved
`RegionOptions` and retags only existing fill surfaces in three source-ordered
passes: zero top shells map `Top` to `Internal`, zero bottom shells map `Bottom`
and `BottomBridge` to `Internal`, then sparse density within strict `1e-4` of
100% maps `Internal` to `InternalSolid`. The pinned static
`PrintObject::infill_only_where_needed = false` keeps `InternalVoid` deferred.

The stage validates all object/record/slot/identity alignment before writing,
then mutates allocation-free in place. Fill vector, geometry, metadata, order,
typed slices, perimeter/thin-fill outputs, boundaries, and boxed predecessor
retain identity. `RegionSurfaceKind::InternalSolid = 5` is non-bridge. The typed
global print `spiral_mode` is now rejected in early capabilities before O17,
closing the threshold-masked record-local bypass while direct tests preserve
the upstream pass guards. Six obsolete unsupported-spiral checkpoint-pinning
tests were deleted rather than retained as legacy expectations. Tier-1 run
`30900710846`, WASM job `91964102068`, confirmed the stale six-pair browser
matrix failed at its first spiral-activated pair with
`unsupported project feature: spiral_mode`. The browser N matrix therefore
excludes the three spiral-activated historical pairs and keeps the supported
alignment, signed-zero, and generator contexts.

The inactive KSR 5/3/15% options preserve checksum
`-126362407653399901571400348049652748978`; O18 totals are
`[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294,
113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011, 0, 0]`. Seventeen focused
O18 tests, 209 O10-O18 regressions, and 5,607 workspace tests with 2 skipped
pass with native, strict Clippy, both WASM, formatting, diff, LOC,
forbidden-pattern, dependency, pinning-removal, and staging gates. Active
project/global and model-part overrides freeze literal 113 top, 6 bottom, 48
bottom-bridge, and 1,127 internal transitions. The final independent
six-dimensional implementation rereview and OpenCode rereview both returned
`VERDICT: APPROVE`.

Task 22O.19 ports caller `PrintObject.cpp:595-596` and the single-region cache
population at `PrintObject.cpp:2008-2027,2111-2149`. Each aligned O18 record
reads `ensure_vertical_shell_thickness` from its resolved region and the
already-scaled `ClassicPreludeRecord.solid_infill_spacing`; `EnsureAll` expands
typed top and bottom/bottom-bridge slices by exactly `(spacing as f32) *
0.05_f32` with miter `3.0`, while other modes produce empty caches. Fill
expolygons flatten contour then holes without union. A borrowed-expolygon
Clipper adapter preserves raw Paths order and the source conditional positive
NonZero union.

All caches stage before ownership moves. The successor retains the exact O18
boxed predecessor and object/record allocations and adds a separate aligned
`Vec<Option<VerticalShellCache>>` sidecar; cache geometry is allocation-distinct.
The existing one-region preflight keeps `PrintObject.cpp:2028-2109` deferred.
Public slicing reaches O19 once and intentionally remains
`ProjectSlicingIncomplete`. KSR freezes cache checksum
`-114359197324258778780701398534712718623`, parent-bound successor checksum
`148296943860974241781127169756103364063`, totals
`[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, and first/later scaled
spacings `[457079, 377079]`. The active cache contains exactly 572 top, 713
bottom, and 1,227 hole paths. Twenty-one focused O19 tests, 310 O10-O19
regressions, and 5,630 workspace tests with 2 skipped pass with strict Clippy,
native all-target, both WASM, formatting, diff, LOC, forbidden-pattern,
dependency, source-pinning, and staging gates. The final independent
six-dimensional implementation rereview and OpenCode rereview both returned
`VERDICT: APPROVE`.

Task 22O.20 ports the single-region projection gather at
`PrintObject.cpp:2153-2278`, stopping before internal-surface trimming at line
2334. For each populated active layer, the crate-private temporary projection
sidecar starts with current cache holes, scans top neighbors before bottom
neighbors using the source's strict count-or-thickness windows and f64
`bottom_z = print_z - height`, intersects holes incrementally, and appends then
NonZero-unions shell Paths incrementally. A neighboring aligned `None` is a
visited empty cache: it clears holes, contributes no shell, suppresses the
anchor, and does not terminate the planned-index window. A current `None`
remains sidecar `None`; its otherwise-transient projection is dead at the next
upstream trim because its internal fill set is empty.

When a positive layer count visits no neighbor but the stopped index exists,
the anchor expands the current cache with the current aligned
external-perimeter spacing after the exact `coord_t -> f32` cast, miter `3.0`,
then intersects with the stopped index's object `lslices` flattened contour
then holes. The existing offset engine preserves per-path CCW Positive cleanup,
CW Negative cleanup/outer removal/result reversal, and final NonZero union. New
Paths-only union/intersection adapters call `Clipper::execute_paths` with
NonZero subject and clip rules; no PolyTree, sorting, or canonicalization is
introduced.

O20 validates every aligned object, record, slot, identity, plan, flow, cache,
and object-slice relation before geometry, stages the whole project while
borrowing O19, then moves every O19 allocation unchanged beside fresh,
non-aliasing projections. Any projection geometry failure has stable text
`vertical-shell projection geometry is outside the supported Clipper range`
and iteratively disposes the predecessor. Public slicing reaches O20 once and
intentionally remains `ProjectSlicingIncomplete`.

KSR freezes parent-bound O20 checksum
`-106767561006193260948265111057697183253`, totals
`[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, and ordered event totals
`[1830, 917, 1539, 749, 0, 0, 0, 0]` for top visits, bottom visits, hole
intersections, shell unions, and four anchor sites. Forty-five focused O20
tests cover exact Paths composition, count/thickness boundaries, current and
neighbor `None`, both anchors, an exact acute miter-3 witness,
current-versus-stopped spacing, every geometry failure site, exhaustive
alignment/identity rejection, recursive ownership of both predecessor tree
families, an active later-object transactional failure, constrained-stack
cleanup, typed 3MF/model-part options, ZIP/name invariance, component scaling,
and independent KSR parses. All 355 O10-O20 regressions and 5,678 workspace
tests with 2 skipped pass with strict Clippy, native all-target, both WASM,
formatting, diff, LOC, forbidden-pattern, dependency, source-pinning, and
staging gates;
the pushed commit additionally requires
the Tier-1 native matrix and complete browser-WASM job.

The next source boundary begins at internal-surface trimming in
`PrintObject.cpp:2334`; regularization, horizontal shells, external-surface
processing, fill generation, seams, ordering, motion, and G-code remain
deferred. O19/O20 sidecars remain temporary compatibility representations of
`PrintObject::discover_vertical_shells`, not an Ares-owned slicing pipeline.

Task 22O.21 ports the bounded internal-surface trim at
`PrintObject.cpp:2334-2342`, stopping before regularization at line 2344. Each
active populated record scans its retained O18 `fill_surfaces` once in
collection order for the reachable `Internal | InternalSolid` envelope and
flattens each ExPolygon contour immediately followed by stored holes. The
source-listed `InternalVoid` role remains unreachable because the pinned
`infill_only_where_needed` producer is statically false; O21 does not synthesize
an enum variant or producer.

The projected O20 shell is flat-Paths intersected against a fresh internal path
list using NonZero rules. The existing shared safety constants and raw offset
engine expand only the clip, independently path by path, by `10.0_f32` with
miter limit `3.0`, preserving CCW Positive and CW Negative cleanup semantics;
expanded paths are not pre-unioned. A separate ordinary flat NonZero difference
`polygons_internal - projection.holes` always executes and appends after the
intersection without sorting, deduplication, union, or PolyTree conversion. The
combined result then reaches the source empty gate. Only a nonempty result
re-scans the collection for `InternalSolid` and appends fresh contour-then-hole
paths verbatim, intentionally retaining solid duplication and performing no
following union.

O21 validates the complete O20 object/cache/projection/fill/input/prelude/plan/
layer alignment before geometry, stages all trim objects while borrowing O20,
and moves the exact predecessor allocations only after whole-project success.
Inactive populated records retain `Some(empty trim)` with no O21 events and
aligned `None` slots stay `None`. Geometry failures use stable text
`vertical-shell internal trimming geometry is outside the supported Clipper
range`, expose no successor, and iteratively delegate O20 disposal. Public
slicing invokes O21 once, iteratively disposes the successor, and remains
`ProjectSlicingIncomplete`.

Independently parsed KSR runs guard the O19 checksum
`148296943860974241781127169756103364063` and totals
`[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, then the O20 checksum
`-106767561006193260948265111057697183253`, totals
`[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, and events
`[1830, 917, 1539, 749, 0, 0, 0, 0]`. O21 freezes parent-bound checksum
`-86220837291247746226319093859583939318`, totals
`[1, 460, 0, 460, 7704, 104680]`, and ordered event totals
`[460, 460, 460, 460, 259]` for safety offset, safety intersection, ordinary
difference, empty gate, and reached solid-append site. Forty-two focused O21
tests, 386 explicit O10-O21 regressions, and 5,717 workspace tests with 2
skipped pass; native all-target check and strict all-feature Clippy are clean.
Post-review compiling mutation REDs cover the complete final 11 adapter, 10
record, and 21 integration filters before byte-exact production restoration.

The next rewrite boundary is regularization beginning at
`PrintObject.cpp:2344`. O19-O21 sidecars remain temporary source-compatibility
representations. O21 adds no public API, persisted format, dependency,
migration, compatibility layer, or fallback. Mechanical rollback restores O20
terminal consumption, removes only O21 state/wiring/tests/docs and the two
flat-Paths adapters, and returns the pre-existing safety constants to private
visibility while retaining all O20 behavior.

Task 22O.22 ports only the initial morphology regularization in
`PrintObject::discover_vertical_shells` at `PrintObject.cpp:2344-2367`, with
its aligned flow provenance at lines 2174-2182. Each nonempty O21 flat shell
uses the retained `ClassicPreludeRecord::solid_infill_spacing`: one `i64` to
`f32` cast, multiplication by `1.05_f32`, then the source-ordered radii
`0.5_f32 * 0.65_f32 * minimum`, `0.5_f32 * 1.2_f32 * minimum`, and
`0.2_f32 * minimum`. The nested geometry remains NonZero `union_ex`, Square
`offset2_ex(-ensure, ensure + sparse, 3.0)`, then Square shrink through the
existing ExPolygon offset with `-(sparse - overlap)` and miter limit `3.0`.
Only the O21 empty gate skips regularization; a nonempty shell whose union
becomes empty still reaches both offset stages and shrink.

The existing two-stage offset implementation exposes one inter-stage observer
entry so O22 can record and fail the second stage after the first succeeds;
the ordinary `offset2_ex` delegates through the identical body with a no-op
observer. This adds no alternate geometry algorithm. Exact Square-join contour
and hole vectors freeze Clipper ExPolygon/path/point ordering, including narrow
material removal and near-gap closure. Actual coordinate failures are exercised
at union, both offset2 stages, and shrink, while the project boundary maps every
site to `vertical-shell regularization geometry is outside the supported
Clipper range`.

O22 validates all O21 object/cache/projection/trim/predecessor, record, slot,
source/transform, region/compatibility, plan/layer/current/input, prelude, and
lslice relations before geometry. It stages the complete project while
borrowing O21, then moves the exact predecessor graph beside fresh aligned
ExPolygon sidecars only after success. Failure and success cleanup iteratively
delegate to O21 on both 10,000-node tree families and the shared constrained
stack. Public slicing invokes O22 once and remains `ProjectSlicingIncomplete`.
Typed `internal_solid_infill_line_width` mutation proves the complete 3MF to
retained spacing to exact radius bits to ordered regularization-output chain;
ZIP repacking, non-slicing renames, model-part precedence, inactive modes, and
component scaling do not select fixture-specific behavior.

Two independent KSR captures guard the frozen O19, O20, and O21 parent values
before freezing O22 checksum
`134936948052282121922360252649864225707`, totals
`[1, 460, 0, 460, 632, 632, 128, 34557]` for objects, slots, `None`, `Some`,
ExPolygons, contours, holes, and points, ordered events
`[259, 259, 259, 259]`, and exact-radii digest
`-119839535044106185061007902266478724784`. Eleven direct and 22 integration
O22 tests pass; 346 explicit O10-O22 regressions and 5,750 workspace tests with
2 skipped pass. Strict all-target/all-feature Clippy is clean. Compiling
post-implementation mutation REDs remove the `1.05_f32` factor and fail 4 of 11
direct tests plus 2 of 22 integration tests. Supplemental mutations fail all 5
alignment tests, the public lifecycle witness, and the genuine later-slot
transaction witness before current tuple-signature production artifacts are
restored byte-exactly and affected/full GREEN gates rerun.

The exact next rewrite boundary starts with `object_volume` and neighbor/tiny
area filtering at `PrintObject.cpp:2369`. Horizontal shells, external surfaces,
fill generation, seams, ordering, motion, G-code, and post-processing remain
deferred. O19-O22 are temporary source-compatibility sidecars, not an Ares-owned
pipeline. O22 adds no public API, persisted format, dependency, migration, or
fallback. Mechanical rollback restores O21 terminal consumption and removes
only O22 state, wiring, tests, docs, and the inter-stage observer entry while
retaining the unchanged ordinary `offset2_ex` behavior.

Task 22O.23 ports the next single-region block of
`PrintObject::discover_vertical_shells` at `PrintObject.cpp:2369-2400` and
stops before `intersection_ex(polygonsInternal, regularized_shell)` at line
2402. For every nonempty O21 trim, previous retained `lslices` are flattened as
the subject and next retained `lslices` as the clip of a flat NonZero
intersection. Current internal Paths are closed by Miter-3 grow then shrink
using `(1e-4_f64 / scale.factor()) as f32`; this floating epsilon never passes
through the truncating coordinate conversion used by the area constants.

Candidate area remains signed `f64`. The selected scale truncates `scaled(1.5)`
and `scaled(8.0)` to `i64`, each is cast to `f32` and multiplied by the shared
O22 minimum in `f32`, and only the resulting products are promoted for strict
`<` comparisons. The source predicate preserves lazy visibility difference,
Miter-3 expansion, and the literal
`diff(internal_volume, expanded_candidate).len() >= internal_volume.len()`
protection heuristic. Survivors are deep-cloned in stable O22 order into a
fresh aligned sidecar without grouping, sorting, union, canonicalization, or
deduplication. A nonempty O21 trim with empty O22 morphology still constructs
both volumes and reaches the empty gate; an empty O21 trim invokes no O23
geometry.

O23 validates the complete O22 object, record, sidecar, input, prelude, plan,
layer, region, identity, and retained-scale alignment before the first event.
It stages the whole project while borrowing O22 and moves the exact predecessor
graph only after success. Every geometry site maps to
`vertical-shell tiny-island filtering geometry is outside the supported
Clipper range`; failure and success cleanup delegate iteratively through both
10,000-node predecessor tree families. Public slicing invokes O23 once and
continues to return `ProjectSlicingIncomplete`.

Two independent KSR captures first reassert O19-O22 and then freeze O23
checksum `-41564956609250807593946297629749369320`, totals
`[1, 460, 0, 460, 632, 554, 78, 554, 128, 33815]`, threshold digest
`-167664109034474951983490568976349754300`, and ordered event totals
`[259, 259, 259, 632, 66, 80, 80, 259]` for neighbor intersection, closing
grow, closing shrink, candidate scan, visibility difference, candidate
expansion, protection difference, and empty gate. The LargeBed scale witness is
`[1221399551, 150000, 1209170944, 799999, 1229148144, 1365946746,
1385985605, 4621819117588971520]`, retaining truncating `scaled(8.0) = 799999`.

Eighteen direct and 29 integration O23 tests pass, as do 393 explicit O10-O23
regressions and 5,797 workspace tests with 2 skipped. Native all-target check,
strict all-target/all-feature Clippy, four WASM checks, optimized default and
feature browser-WASM builds/export audit, and two 9-test Playwright runs are
green. Ten required compiling behavioral mutations are killed by their
intended witnesses, followed by byte-exact production restoration and GREEN
reruns. Formatting, diff, dependency, forbidden-pattern, staging, and LOC
audits pass; every Rust file is below 400 LOC and every O23 shard is at most 270
LOC.

The exact next rewrite boundary is
`intersection_ex(polygonsInternal, regularized_shell)` at
`PrintObject.cpp:2402`. Fill-surface mutation, `InternalVoid`, horizontal
shells, external surfaces, fill generation, seams, ordering, motion, G-code,
and post-processing remain deferred. O19-O23 stay temporary
source-compatibility sidecars rather than an Ares-owned pipeline. Rollback
restores O22 terminal consumption and removes only O23 state, wiring, tests,
docs, the restricted O22 minimum accessor, and the O21 internal-flattening
visibility change.

Task 22O.24 ports the coherent final state transition in
`PrintObject::discover_vertical_shells` at `PrintObject.cpp:2402-2432`. It
adds the source-cited `InternalVoid = 8` vocabulary with exhaustive non-bridge
classification and extends the shared internal flattening order to Internal,
InternalVoid, and InternalSolid. The one new mixed adapter consumes flat
Polygon subject Paths directly, consumes ExPolygon clip contours then holes,
and reuses the existing two-pass NonZero PolyTree output. It introduces no
safety offset, pre-union, sorting, canonicalization, deduplication, or alternate
geometry algorithm.

For every nonempty O23 filter, O24 computes from the original collection, in
source order, the mixed internal-solid intersection, Internal difference, and
InternalVoid difference. Only after whole-project staging succeeds does it
stably retain Top, Bottom, and BottomBridge, then append fresh Internal,
InternalVoid, and InternalSolid surfaces with default metadata
`(-1.0, 1, -1.0, 0)`. Empty filters execute no geometry and preserve the exact
record allocation. Complete inherited alignment, including the retained scale
against typed printable area, is checked before the first event. Failure,
success, and public-incomplete cleanup iteratively dispose both 10,000-node
predecessor families while the exact O23 graph and all unrelated allocations
remain owned by the successor.

Two independent KSR captures first reassert O23, then freeze O24 checksum
`-117597382518472843802490205604634875775`, pre/post kind totals
`[113, 6, 48, 1127, 0, 0]` and `[113, 6, 48, 1281, 575, 0]`, and pre/post
geometry totals `[1294, 168, 46011]` and `[2023, 270, 73848]`. Of 460 records,
161 are active, 299 are no-ops, exactly those 299 remain byte-logically
unchanged, and real KSR InternalVoid counts remain `[0, 0]`. Digest framing
records object/slot positions, record and surface boundaries, path counts,
contour/hole role and index, point counts, and end markers. The delimited
record-sequence digest is
`-65994586923856785425316699963519338136`; the exact event-sequence digest is
`-110138798119262824097709645699717637653`, with ordered event totals
`[161, 161, 161]`.

Thirty-one focused tests cover empty, disjoint, partial, full-cover, multiple,
holed, nested, mixed-winding, metadata, ordering, InternalVoid, provenance, all
inherited alignment, transactionality, ownership, cleanup, lifecycle,
metamorphism, and repeatable KSR evidence. The exact-source parent regression
set passes 149/149 and workspace Nextest passes 5,827 tests with 2 skipped.
Thirteen planned compiling mutations
and the reviewer-added retained-scale mutation are killed by their intended witnesses; the
commutative role-only intersection reversal is retained honestly as an
equivalent control, and final production restoration is byte-exact. Repository
Nextest, native/strict Clippy, Tier-1 WASM, browser-WASM/Playwright, formatting,
LOC, forbidden-pattern, dependency, rollback, staging, independent-review, and
exact-commit CI evidence are the release gate recorded with the implementation.

The next source-cited rewrite boundary is the
`PrintObject::prepare_infill` call to `discover_horizontal_shells` at
`PrintObject.cpp:618`, owned by `PrintObject::discover_horizontal_shells` at
lines 3955-4161. O19-O24 remain temporary source-compatibility state, not an
Ares-owned pipeline. O24 adds no public API, persisted format, dependency,
migration, fallback, or production InternalVoid producer. Mechanical rollback
restores O23 terminal consumption and removes only O24 state/wiring/tests/docs,
the mixed adapter, the private InternalVoid vocabulary updates, and shared
helper selection while retaining all O23 behavior.

Task 22O.25 ports the first coherent operation of
`PrintObject::discover_horizontal_shells` at `PrintObject.cpp:3955-3972`,
stopping before the `ensure_vertical_shell_thickness == evstAll` gate at line
3974. For each aligned region record in planned layer-array order, it preserves
the exact raw-empty `extra_solid_infills` short circuit, parses nonempty
schedules through the shared typed option, applies one-based matching to the
zero-based planned index, and retags every exact Internal `fill_surfaces` entry
to InternalSolid in place. It does not inspect stored layer IDs or sparse
infill density and does not mutate `slices`, geometry, ordering, metadata, or
allocation identity.

The shared schedule parser now exposes only a crate-private raw-string seam;
the JSON entry delegates to it. Numeric components are restricted to the
source-sized positive signed-`i32` domain, explicit ranges and one-based
matching use checked addition, and native and browser-WASM boundaries return
`invalid extra_solid_infills pattern` rather than overflow or trap. Existing
strict Ares malformed-token behavior is retained rather than broadening O25
into a rewrite of Orca's permissive `std::stoi` prefixes.

O25 validates the complete inherited O24 alignment before its first schedule
visit, stages every decision while borrowing O24, and mutates only after the
whole project succeeds. A later parse failure therefore exposes no partial
promotion. The successor moves the exact boxed predecessor, object records,
O19-O24 sidecars, and all nested allocations; cleanup reconstructs O24 and
delegates to its iterative disposal. Public project slicing invokes O25 once
after O24, disposes it, and remains `ProjectSlicingIncomplete`.

Two independent KSR captures first reassert O24 and then freeze O25 checksum
`58727684244877231975278290246623082466`, record-sequence digest
`160750122870413723145549886803558415603`, event-sequence digest
`95826544899519698779358289371798515623`, and unchanged surface digest
`-107673730348313625723619859456104452971`. All 460 aligned records retain kind
totals `[113, 6, 48, 1281, 575, 0]` and geometry totals
`[2023, 270, 73848]`; event totals are `[460, 0, 0, 0, 0]` for raw visits,
nonempty guards, parser calls, matcher calls, and promoted surfaces. Commits are
zero, and prepare/disposal counts are exactly one. A typed archive mutation
independently promotes all 1,281 Internal surfaces across 460 matching records,
ending with Internal/InternalSolid totals `0/1856` while preserving the full
allocation graph.

Forty-two focused O25/shared-option tests, 191 explicit O21-O25 regressions,
and 5,856 workspace tests with 2 skipped pass. Fourteen compiling behavioral
mutations cover parsing, one-based planned-index matching, density independence,
transactionality, inherited alignment, holed geometry, full nested ownership,
cleanup, lifecycle precedence, metamorphism, active typed archives, real KSR,
and executed browser-WASM numeric boundaries. Native and strict Clippy checks,
four WASM checks, optimized default/feature export audits, two 10-test
Playwright runs, formatting, LOC, dependency, forbidden-pattern, rollback, and
byte-exact mutation restoration are green. Both independent six-dimensional
review paths are approved; exact pushed-SHA Tier-1 CI remains the release gate.

The next source boundary is the EnsureAll early return at
`PrintObject.cpp:3974-3976`. All horizontal-shell surface gathering, layer and
thickness windows, geometric propagation, safety offsets, density/ensure-mode
branches, collection rebuilding, external-surface processing, fill generation,
seams, motion, G-code, and post-processing remain deferred. O19-O25 are
source-compatibility state only. Rollback restores O24 terminal consumption and
removes O25 state/wiring/tests/docs plus the narrow crate-private raw parser
seam while retaining O24 behavior and the legacy density-gated helper.

Task 22O.26 ports the complete executable remainder of
`PrintObject::discover_horizontal_shells` at `PrintObject.cpp:3974-4150`,
ending before debug SVG output. It preserves the per-record EnsureAll skip,
Top/Bottom/BottomBridge source order, directional count-or-strict-thickness
windows, and serial visibility of earlier neighbor rebuilds to later source
gathers. Every option and flow is taken from the aligned resolved record: the
source record owns layer counts and thickness, while each visited neighbor owns
external-flow width. The existing coordinate scale remains derived from the
resolved printable area.

The rewrite performs source gathering from `slices` then `fill_surfaces`, flat
contour-before-hole topology, the ordered asymmetric Miter-5 opening, the exact
safety intersection and mode-dependent empty control flow, density/mode narrow
wall filters, and Miter-3 repair expansion. InternalVoid participates only in
the reachable repair clip; rebuilding drops it, unions solid geometry, appends
fresh InternalSolid and Internal surfaces, and reconstructs top/bottom groups
from their first metadata-complete template.
`RegionSurface::clone_with_expolygon` changes only geometry. The stable geometry
error is `horizontal-shell propagation geometry
is outside the supported Clipper range`.

O26 validates the complete inherited graph before cloning, operates on a
whole-project working clone, and commits only dirty records after every object
succeeds. Geometry-equal executed rebuilds are still dirty; untouched records
keep allocation identity. Failure leaves the O25 graph unchanged. The
successor retains the boxed O25 predecessor and all O19-O24 sidecars without a
durable propagation sidecar; cleanup delegates iteratively. Public project
slicing invokes O26 once after O25, disposes it, and remains
`ProjectSlicingIncomplete`.

Repeated parent-bound KSR capture preserves the O25 checksum and freezes the
unchanged surface digest
`-107673730348313625723619859456104452971`, event digest
`55157732452648897477979936233453742487`, 460 record visits, 460 EnsureAll
skips, zero source-kind visits, zero geometry, and zero commits. A typed
Moderate archive capture freezes raw event totals
`[460, 460, 0, 1380, 1010, 547, 143]` for fill clones, record visits,
EnsureAll skips, source-kind visits, neighbor visits, rebuilds, and dirty
commits. All 547 rebuilds follow nonempty intersections and commit 143 distinct
dirty records. It freezes surface digest
`55371787254720044626064449746884984931`, event digest
`71433667081695804905700384637078674080`, and 5,469 ordered geometry events.

Forty-five final O26-focused tests plus six opening and one surface-template
test pass; the complete workspace passes 5,908 tests with 2 skipped. Thirty-three
compiling behavioral mutations are killed and production is restored before
final formatting, native check, and strict Clippy gates. Controlled production
witnesses prove a later source gathers an exact external fragment rebuilt by an
earlier source, every ordered failure retains an identical original/sidecar
fingerprint with zero commits, clean inner geometry and all O19-O24 sidecars
retain allocations, and a geometry-equal production rebuild is still dirty.
Resolved archive/model-part witnesses independently vary count, thickness,
density, both flow owners, scale, and non-slicing names. Optimized native and
browser-WASM checks, export audit, and two 11-test Playwright executions cover
the unchanged `sliceProject` boundary. Final independent six-dimensional and
default-model OpenCode reviews approve O26; the exact pushed-SHA Tier-1 matrix
remains its release gate.

The next cited rewrite boundary is the `prepare_infill` call to
`PrintObject::process_external_surfaces` immediately after
`discover_horizontal_shells` (`PrintObject.cpp:624-642` in the pinned tree),
owned by the corresponding `PrintObject` and `LayerRegion` external-surface
implementation. External-surface processing, infill combination, fill
generation, toolpaths, seams, motion, G-code, and post-processing remain
deferred. O19-O26 remain temporary source-compatibility state. Mechanical
rollback restores O25 terminal consumption and removes O26 state, wiring,
tests, docs, path-opening adapter, and narrow surface-template seam while
retaining O25 unchanged.

Task 22O.27 rewrites the direct supplied-seed slice of
`Algorithm/RegionExpansion.cpp` and the reached Clipper 6.4.2 offset branches.
Its exact upstream boundary is `EndType::etClosedLine`,
`EndType::etOpenRound`, `ClipperOffset::AddPath`, `FixOrientations`, reached
`DoOffset` branches, `RegionExpansionParameters::build`, the direct
`propagate_waves(const WaveSeeds &, ...)` overload, and its bbox/wavefront
helpers. The Rust destination is crate-private `geometry::region_expansion`
and the existing ARD-0024 offset kernel; no second geometry engine, dependency,
public option, or persisted state is introduced.

The offset extension distinguishes exact point equality from strict positive
shortest-edge filtering, preserves raw closure before filtering, mixed
ClosedPolygon/ClosedLine orientation, strict near-zero behavior, one-point
join discretization, two-sided ClosedLine order, and OpenRound side/cap order.
The parameter builder preserves source `f32` calculations, double-literal
reduction, scale-derived `f64` tolerances, and f32-sum-to-f64 max-inflation
rounding. Direct propagation processes only contiguous `(boundary, src)`
groups, keeps one offsetter configured across `clear()`, trims the selected
boundary contour then holes by the truncated inflated bbox, and performs
ordered staged Round expansion with Clipper-operation-order orientation,
clockwise sign/reversal, and Positive/Positive clipping. Final polygons retain
literal Clipper order and source/boundary IDs; the first `ClipperError` is
returned directly.

Twenty-one O27-focused tests freeze six end-type cases, five bit-exact
parameter cases, nine propagation cases, and the bbox constructor. They cover
Normal and LargeBed scales, open/closed/multi-step complete paths, holes,
contiguous and separated groups, strict bbox truncation, Positive versus
NonZero, clockwise sign/reversal, and three range-failure sites. Twenty-eight
compiling mutations cover precision/reassociation/scale, group order, staged
steps, and error operation order as well as the geometry branches. The complete
geometry regression passes 77
tests and the workspace passes 5,929 tests with 2 skipped. Native all-target
check, strict workspace Clippy, four wasm32 checks, optimized WASM/export
audit, two 11-test Playwright runs, formatting, LOC, dependency,
forbidden-pattern, unchanged-lifecycle, and rollback audits are green. The
independent six-dimensional reviewer approved after its repair/re-review loop,
and the separate default-model OpenCode reviewer returned
`VERDICT: APPROVE`. Exact pushed-SHA Tier-1 remains the release gate. ARD-0024
remains accepted and unchanged.

O27 is not an external-surface lifecycle stage and changes no KSR checkpoint.
O28 takes the next ClipperZ-backed `RegionExpansion.cpp::wave_seeds` boundary:
expanded/opened Z paths, Z-fill intersections, split reconciliation,
source/boundary ID recovery, and the closed-seed AABB fallback. Source-taking
propagation, merge helpers, `LayerRegion::process_external_surfaces`,
`PrintObject::process_external_surfaces`, fill generation, toolpaths, seams,
motion, G-code, and post-processing remain deferred. Mechanical rollback
removes only O27 RegionExpansion/end-type code, tests, and docs while retaining
O26 unchanged.

Task 22O.28 rewrites pinned `Algorithm::wave_seeds` from
`Algorithm/RegionExpansion.cpp:88-391`, its `ClipperZUtils.hpp` conversion and
visitor path, the four-direction `Polyline.hpp` merge, and the reached bundled
ClipperZ metadata sites. The Rust destination is crate-private
`geometry::region_expansion::wave_seeds` plus optional Z provenance in the
single ARD-0024 indexed kernel. Public 2-D geometry, manifests, adapters, and
the O26 lifecycle remain unchanged.

The same kernel now carries geometry-private `KernelPoint { xy, z }` records,
while ordinary equality and every geometry predicate remain XY-only. Existing
2-D adapters assign zero Z. Z execution preserves endpoint priority,
direction-sensitive horizontal and strictly-simple fills, output survivor and
join metadata, one execution-local sorted/deduplicated intersection table, and
optional PolyTree Z sidecars. `wave_seeds` preserves source contour/hole offset
signs and IDs, NonZero intersection order, repeated endpoints, split
swap-pop/reprocessing, four ID-recovery branches, and deliberate release drops.
Its fallback AABB is lazy and retains contour-only inflated boxes, literal
`min + max / 2` centroid arithmetic, longest-axis X ties, QuickSelect order,
left-first traversal, and outer/hole boundary semantics. Optional sorting uses
the accepted fixed MSVC STL 14.44 comparator control flow on an index
permutation with no geometry or index tie-break.

Twenty-five focused Z tests, 39 focused wave-seed tests, 211 Clipper tests, and
53 RegionExpansion tests freeze complete ordered XYZ/ID/XY vectors, debug and
release divergence, ordinary 2-D equivalence, O27 handoff, and lifecycle
inactivity. The full workspace passes 5,994 tests twice with 2 skipped. All 23
specified behavioral mutations plus a strict shortest-edge mutation are killed
and followed by restored GREEN runs. Pinned debug/`NDEBUG` C++ diagnostics
cover inside, crossing, hole, split, multiple-ID, overlapping-fallback, and the
release-only shared-vertex case; the accepted ARD-0024 MSVC comparator literals
cover comparator-equivalent groups over 32. Original compiling-RED chronology
is unavailable and is recorded as a limitation rather than reconstructed.

Native check, strict all-feature Clippy, formatting, four wasm32 checks, two
optimized WASM builds, export and syntax audits, and two 11-test Playwright
runs are green. Static audits prove the file boundary, LOC limits, no staged
files, no new dependency or forbidden construct, and no lifecycle/API change.
A disposable exact-state rollback returns cleanly to predecessor
`f361bb73b558b4e50bfa4fa712afcd63df44ba9f` and leaves the primary worktree
byte-for-byte unchanged. Final documented-state independent six-dimensional
and separate default-model OpenCode reviews both return `VERDICT: APPROVE`.
Implementation commit `7eb0d27` and documentation commit `be33437` are pushed;
exact-SHA Tier-1 run `31156094839` passed Linux, macOS, Windows, formatting,
WASM, export, and two browser executions at
`be334375be871eb12ca98c98d889b65a92d13a37`.

O28 remains a geometry prerequisite and changes no KSR checkpoint or G-code
byte. The next bounded source boundary is the source-taking
`propagate_waves(const ExPolygons &, const ExPolygons &,
const RegionExpansionParameters &)` overload and its scalar overload at
`Algorithm/RegionExpansion.cpp:463-478`, composing O28 seed discovery with
unchanged O27 propagation. `propagate_waves_ex`, expansion merge helpers,
`LayerRegion`/`PrintObject` external-surface orchestration, fill generation,
toolpaths, seams, motion, G-code, and post-processing remain deferred.
Mechanical rollback removes O28 Z/seed/AABB modules, private metadata seams,
tests, and O28 documentation while retaining all O27 code and the exact O26
lifecycle.

Task 22O.29 composes the next pinned source boundary,
`Algorithm/RegionExpansion.cpp:463-466,468-477` with declarations at
`Algorithm/RegionExpansion.hpp:74-83`. Its Rust destinations are the two
crate-private `geometry::region_expansion` wrappers
`propagate_waves_from_sources` and `propagate_waves_from_sources_with_steps`.
The parameter wrapper requests literal sorted O28 discovery and directly invokes
unchanged O27 propagation. The scalar wrapper builds once and delegates once,
using the same retained explicit `CoordinateScale`.

Complete composition evidence includes the compact and sorted/unsorted ordered
vectors and full scalar polygons with 16 Normal-scale points and 128
LargeBed-scale points. The final composition filter passes 5/5 and the complete
RegionExpansion filter passes 58/58. Ten runtime mutations are killed/restored,
one differently typed signature mutation is compiler-rejected, and source
inspection—not a false behavioral mutation claim—confirms exactly one scalar
builder call followed by exactly one parameter-wrapper call. The frozen
six-argument scalar signature has one function-scoped, reasoned
`#[expect(clippy::too_many_arguments)]` because the workspace threshold is five;
no lint `allow` was added. Final allowed-Rust LOC are 172 (`propagate.rs`), 55
(`region_expansion.rs`), 150 (`geometry.rs`), 5 (test root), and 264
(`composition.rs`).

The original compiling RED artifact is truthfully bounded: the earlier
eight-test `/tmp/task22o29-red-focused-all.txt` has seven empty-stub assertion
failures and one `scalar_scale_outputs_differ` pass while both wrapper stubs
returned empty because that test compared explicit pipelines. The final shard
was subsequently consolidated and strengthened into five tests, including
valid discovery before propagation failure. It has no fresh chronological RED.
Mutation kills and restored GREEN runs are post-hoc recurrence evidence, not
original RED evidence.

O29 introduces no new architecture decision and leaves ARD-0024 unchanged. It
adds no option, public export, lifecycle wiring, checkpoint, persisted state, or
G-code byte; public slicing remains on O26 and returns
`ProjectSlicingIncomplete`. Rollback removes only O29 wrappers, private
reexports/signature assertions, composition tests/registration, and O29 docs,
retaining O27, O28, and the O26 lifecycle. The restored final local state
passes composition 5/5, RegionExpansion 58/58, O26 lifecycle 3/3, workspace
5,999/5,999 with 2 skipped, native all-target check, warning-denying Clippy,
rustfmt, four WASM checks, two optimized WASM builds, export/syntax audits, two
11/11 Playwright runs, static audits, and disposable rollback. Final
documented-state independent six-dimensional and default-model OpenCode
rereviews both return literal `VERDICT: APPROVE`. O29 was released as
implementation commit `55c2c23` and documentation commit `118f6a7`; exact-SHA
Tier-1 run `31168584784` passed all format, WASM/browser, Linux, Windows, and
macOS jobs at `118f6a72b33926efe41ced1c931f9a51b26b2945`.

The next bounded source boundary is direct supplied-seed
`propagate_waves_ex` at `Algorithm/RegionExpansion.cpp:480-503`. Its source
scalar overload, expansion/merge helpers, external-surface
processing, fill generation, toolpaths, seams, motion, G-code, and
post-processing remain deferred.

Task 22O.30 locally ports the direct supplied-seed boundary at
`Algorithm/RegionExpansion.cpp:480-503` and `RegionExpansion.hpp:85-92` into
crate-private `RegionExpansionEx` and `propagate_waves_ex`. It reuses unchanged
O27 propagation and the sole ARD-0024 indexed NonZero `union_ex` kernel. The
post-propagation debug assertion is nondecreasing by boundary then source;
conversion preserves adjacent expanded-group order, singleton contours,
complete hole/island topology, and IDs.

Complete pinned-source vectors cover a natural one-seed/two-contour hole,
multi-island output, boundary/source transitions, comparator conflict,
release-unsorted adjacency, zero output, and direct error precedence. The
focused shard passes 6/6 in debug and release; RegionExpansion passes 64/64,
PolyTree 6/6, O26 lifecycle 3/3, and workspace 6,005/6,005 with 2 skipped.
Sixteen runtime mutations are killed, one type-shape mutation is
compiler-rejected, and the two oracle-demonstrated semantic survivors
(Positive for valid positive-clipped material and repeated singleton union) are
reported rather than misclassified. Native/WASM/browser gates are green and
final LOC are 74, 218, 62, 156, 6, and 263.

O30 adds no architecture decision, public export, Option, lifecycle,
checkpoint, persisted state, KSR golden change, or G-code byte. Public slicing
continues to return `ProjectSlicingIncomplete`. Exact static and disposable
rollback gates are green; final independent six-dimensional and default-model
OpenCode implementation reviews both return literal `VERDICT: APPROVE`.
O30 was released as commits `0a19939`/`6ccb145`; exact-SHA Tier-1 run
`31184069746` passed all five jobs at
`6ccb145dbb1867e5724538fb071795a7fd4179f0`.

Task 22O.31 ports source/scalar `propagate_waves_ex` at
`RegionExpansion.cpp:506-520` and `RegionExpansion.hpp:94-100`. The private
wrapper performs exactly one parameter build, sorted O28 seed discovery with
the built tiny expansion and same explicit scale, and unchanged O30 delegation.
Its five-test shard passes debug/release and complete RegionExpansion passes
69/69; nine runtime mutations are killed, one signature mutation is compiler
rejected, and one discovery-scale focused survivor is disclosed with exact
forwarding fixed by structural audit. O31 adds no architecture decision,
public export, Option, lifecycle, checkpoint, persisted state, KSR golden
change, or G-code byte. It was released as commits `7113f7c`/`1f89dd3`;
exact-SHA Tier-1 run `31196271880` passed all five jobs at
`1f89dd34c9226a96b92ddc1711c317ff6ce7b7b0`.

Task 22O.32 locally ports `expand_expolygons` at
`RegionExpansion.cpp:522-534` and `RegionExpansion.hpp:102-108`. The private
helper preallocates `src.len()` raw-polygon slots, delegates once to O29 with
unchanged scalar inputs and explicit scale, and moves every returned polygon to
its source-ID slot. Empty slots, complete point topology, source-index slot
order, and per-slot propagation order are preserved; boundary IDs are discarded
exactly as upstream. The five-test shard passes debug/release; RegionExpansion
passes 74/74. Thirteen runtime mutations are killed, two type-shape mutations
are compiler-rejected, and structural equivalences are disclosed. Initial
independent and default-model OpenCode reviews approve. After a test-only
Clippy repair, workspace Nextest passes 6,015/6,015 with 2 skipped; native,
WASM build/export/syntax, static, and exact-O31 rollback gates are green. Local
Chromium cannot launch because `libglib-2.0.so.0` is absent, so exact-SHA CI
must pass the two browser runs. O32 adds no architecture decision, public
export, Option, lifecycle, checkpoint, persisted state, KSR golden change, or
G-code byte. Final independent six-dimensional and default-model OpenCode
reviews both approve. Implementation/documentation commits `2e7168f`/`699f02b`
were pushed; exact-SHA Tier-1 run `31213611275` passed all five jobs, including
both browser runs, at `699f02b2bbc3d797f53edf5f8c65dd2614830ecb`. O32 is
released.

Task 22O.33 locally ports `merge_expansions_into_expolygons` at
`RegionExpansion.cpp:536-587` and `RegionExpansion.hpp:110-111`. The private
helper sorts movable records by source ID through O28's fixed-MSVC index
permutation, moves untouched sources unchanged, appends source contour and holes
after expansion polygons, applies the fixed unscaled 10/Miter/3 safety-offset
union, and uses the O28 AABB sampler with explicit scale to retain the
source-connected component. It adds no new engine, architecture decision,
Option, public export, lifecycle, checkpoint, persistence, adapter, golden
expectation, or G-code byte.

The chronological stub RED produced ten meaningful failures and one equivalent
pass. After review repaired a missing true-zero witness and the temporary C++
oracle's moved-buffer defect, focused debug/release pass 13/13,
RegionExpansion passes 87/87, and corrected debug/`NDEBUG` oracle output is
byte-identical. Thirteen runtime mutations are killed, one signature mutation
is compiler-rejected, and four structural/equivalent survivors are disclosed.
Repaired independent and default-model OpenCode initial implementation reviews
approve. A test-only function-pointer type alias repair resolved Clippy, then
the complete exact candidate was rerun: focused debug/release 13/13, AABB 8/8,
O32 5/5, RegionExpansion 87/87, PolyTree 6/6, offset 58/58, lifecycle 3/3,
workspace 6,028/6,028 with 2 skipped, native lint/format, four WASM checks, two
optimized builds, export, and JavaScript syntax gates pass. Both local browser
attempts fail before test code only because `libglib-2.0.so.0` is unavailable;
exact-SHA CI must pass both runs. Final static audit passes, and disposable
exact-O32 rollback proves candidate/primary byte identity while passing
RegionExpansion 74/74, PolyTree 6/6, and lifecycle 3/3. After repairing exact
oracle inputs and stale release text, the complete suite was rerun and final
independent and default-model OpenCode rereviews both approved.
Implementation/documentation commits `b9e65fd`/`0f6f801` were pushed;
exact-SHA Tier-1 run `31228800274` passed all five jobs, including both browser
runs, at `0f6f80130d28c0cc629e8561e46d187b137a8206`. O33 is released.

Task 22O.34 locally ports `expand_merge_expolygons` at
`RegionExpansion.hpp:113` and `RegionExpansion.cpp:589-594`. The private
composition borrows the full source vector for O29, propagates O29 errors before
any merge work, then moves the original source and complete ordered expansion
records into O33 with the same explicit scale. It adds no builder, direct seed
call, ordering step, clone, shortcut, validation, public export, Option,
lifecycle, checkpoint, persistence, adapter, golden expectation, or G-code
byte.

The historical stub run reported 0/5, with only four failures attributable to
the empty body; the deleted fifth failed in direct O29 setup. The replacement
non-empty handoff witness is recorded only as post-body recurrence/GREEN
evidence. Focused debug/release pass 5/5 and RegionExpansion passes 92/92. Six
runtime mutations are killed, one signature mutation is compiler-rejected, and
two behaviorally equivalent scale substitutions plus one
valid-O29-unreachable O33-error swallowing survivor are truthfully disclosed.
Post-mutation
restoration and the initial static audit pass. The default-model OpenCode
initial review approved, while independent review required physical placement
after O33 and non-vacuous multiple-source, multiple-hole ordering/ownership
evidence. Both repairs are present and verified. The repaired exact candidate
passes focused debug/release 5/5, O29 5/5, O33 13/13, RegionExpansion 92/92,
PolyTree 6/6, offset 58/58, lifecycle 3/3, workspace 6,033/6,033 with 2 skipped,
check, warning-denying Clippy, rustfmt, four WASM checks, two optimized builds,
export, and JavaScript syntax. Both local Playwright attempts fail before test
code only because Chromium cannot load `libglib-2.0.so.0`; exact-SHA CI retains
both runs. Disposable exact-O33 rollback proves candidate/primary byte identity
and the 87/6/58/3 baseline suites. The repaired candidate's independent
six-dimensional and default-model OpenCode rereviews both approve.
Implementation/documentation commits `f499058`/`25460c2` were pushed;
exact-SHA Tier-1 run `31259140846` passed all five jobs, including both browser
runs, at `25460c2abfc5bf94104f41b05df5af2dfac419ee`. O34 is released.

Task 22O.35 locally ports `LayerRegion.cpp:147-163,166-171,439-484` and
`ClipperUtils.hpp:19,27,407-408`. The inactive private helper performs ordered
move extraction for one surface kind, one O29 call per zone with post-success
flags and wrapping boundary rebasing, one O33 merge, explicit Miter/3 closing,
conditional zone differences, and output materialization with Orca metadata
defaults plus the supplied bridge angle. Selected source records retain their
metadata with empty moved geometry; nonmatching point-buffer ownership remains
unchanged.

After two test-only pre-RED repairs with both stubs unchanged, the authoritative
compiling RED ran 13 tests with two truthful equivalent passes and 11 intended
failures. Focused debug/release pass 13/13; offset 62/62, O29 5/5, O33 13/13,
O34 5/5, and RegionExpansion 92/92 also pass. Fourteen runtime mutations are
killed, one signature mutation is compiler-rejected, and four equivalent
miter/rebasing/scale mutations are truthfully retained as structural survivors.
Exact restoration, warning-denying focused Clippy, rustfmt, LOC/private
visibility/forbidden audits, and both initial independent/default-model
OpenCode reviews pass.

The complete documented implementation candidate passes focused debug/release
13/13, offset 62/62, O29 5/5, O33 13/13, O34 5/5, RegionExpansion 92/92,
PolyTree 6/6, lifecycle 3/3, workspace Nextest 6,046/6,046 with 2 skipped,
all native/static/WASM/build/export gates, and exact-O34 rollback with
5/92/6/58/3 baseline suites. Both local Playwright attempts fail before test
code only because Chromium cannot load `libglib-2.0.so.0`; they are not passes
and exact-SHA CI retains both runs. Final independent six-dimensional and
default-model OpenCode implementation reviews both approve with no required
changes.

Implementation/documentation commits `984bc01`/`c6f23ce` were pushed;
exact-SHA Tier-1 run `31269521736` passed all five jobs and both browser
executions at `c6f23ce1a9350ca76241d007f804f3fcfa22c352`. O35 is released but
remains inactive. It adds no Option, public API, lifecycle, adapter, golden
expectation, or G-code byte; its partial mutation order must be contained by a
future staged owned caller. Public slicing still consumes O26 and returns
`ProjectSlicingIncomplete`.

Task 22O.36 locally ports pinned `LayerRegion.cpp:353-356,358-393`. Its
translation-unit-local, inactive crate-private helper composes O28 sorted seed
discovery and O30 ExPolygon propagation across ordered O35 `ExpansionZone`s,
rebases both boundary-ID streams by the full prior-zone ExPolygon domain,
commits per-zone flags after both fallible calls, and move-appends ordered
anchors and expansions. Explicit `CoordinateScale`, `u32` casts, and
`wrapping_add` preserve Ares' platform-neutral replacement for Orca's implicit
scale and C++ `unsigned` behavior.

The compiling empty-stub RED failed 0/6. Focused debug/release pass 6/6; O35,
O28, O30, O31, RegionExpansion, external-surface, PolyTree, offset, and O26
lifecycle regressions pass 13/39/6/5/92/15/6/62/3. The exact pinned Orca CLI
sliced the KSR input successfully in a disposable environment; original-helper
Debug/NDEBUG vectors are byte-identical and match complete committed Rust
literals. The one-at-a-time campaign has 13 runtime kills, two compiler
rejections, and two truthful sorted/scale equivalent survivors. Exact
restoration, rustfmt, LOC/private visibility, and both initial independent and
default-model reviews pass. A final-review test-only repair now explicitly
proves nonempty O28 output before first/later O30 errors; production and the
flag/no-partial assertions are unchanged, and the shard is 295 LOC.

The repaired complete candidate passes O36 debug/release 6/6, focused suites
13/39/6/5/92/15/6/62/3, workspace 6,052 passed with 2 skipped, native lint/
format, four WASM checks, two optimized builds, export/JavaScript audits, and
exact-O35 rollback 13/92/6/62/3. Both local Playwright runs remain pre-test
`libglib-2.0.so.0` launch failures, not passes. Both final implementation
rereviews approve.

Implementation/documentation commits `b546e6f`/`3e927ed` were pushed;
exact-SHA Tier-1 run `31280579891` passed all five jobs and both browser
executions at `3e927ed569d3db8d6f5c08b7843fb049fcc86412`. O36 is released but
remains inactive. It changes no Option, public lifecycle, adapter, golden
expectation, or G-code byte. Public slicing still consumes O26 and returns
`ProjectSlicingIncomplete`. The next source boundary is `Bridge`, `group_id`,
and `get_grouped_bridges` at `LayerRegion.cpp:174-260`; bridge direction/merge
and `process_external_surfaces` orchestration remain deferred.

Task 22O.37 locally ports pinned `LayerRegion.cpp:174-260`: the source-shaped
`Bridge` record, literal parent traversal, and adjacent-window overlap grouping.
The inactive crate-private implementation moves ordered source ExPolygons,
retains expansion-end sentinels and absent angles, caches contour-only bounds,
ignores holes, and evaluates each ordered pair through equal-source, exact
inclusive bbox, and one fallible NonZero contour intersection before lower-root
union. It returns the raw, non-normalized parent forest.

The initialization-only stub compiled and ran ten tests with six body-dependent
failures and four disclosed stub-equivalent passes. The pinned original-Orca CLI
sliced the KSR project to a disposable nonempty G-code that was deleted without
content ingestion; its linked original helper passed 45 assertions and emitted
byte-identical Debug/`NDEBUG` vectors matching complete Rust literals. The
repaired candidate passes O37 debug/release 10/10 and focused O36/O35/O28/O30/
RegionExpansion/external-surface/PolyTree/boolean-path/offset/O26 suites
6/13/39/6/92/25/15/11/62/3. A private pair-helper extraction fixes Clippy
nesting without changing tests or source order; warning-denying Clippy and
rustfmt pass. The post-repair exact-byte campaign kills thirteen runtime
mutations, compiler-rejects two, truthfully records one bbox-comparison
equivalent survivor, and restores all five hashes. Body/test shards are 96/289
LOC, and both repaired initial implementation reviews approve.

Implementation/documentation commits `a0caa5a`/`4d83d15` were pushed;
exact-SHA Tier-1 run `31291016394` passed all five jobs and both browser
executions at `4d83d15832c7905d7ea9727d14c07c5a75eb7312`. O37 is released but
remains crate-private and inactive. It changes no Option, lifecycle, adapter,
golden expectation, or G-code byte. Public slicing still consumes O26 and
returns `ProjectSlicingIncomplete`.

Task 22O.38 locally ports pinned `BridgeDetector.hpp:75-119`,
`PrincipalComponents2D.hpp:12-20` / `PrincipalComponents2D.cpp:8-138`,
`Line.hpp:180`, and the cited Eigen 5.0.1 normalization boundary. The inactive
crate-private helper preserves explicit request-local scale, source-associated
signed `f32` moment accumulation, mixed-width eigen casts, f64 edge-normal
normalization, `ceil(atan2*1000)` first-emplace, original cost order, strict
minimum, and final perpendicular rotation. Its private deterministic adapter
reproduces the audited MSVC STL 14.44 FNV/hash-double, eight-bucket, occupied-
bucket front insertion, unique-emplace, 8-to-64 growth, and rehash-group
iteration target without host hash order or a platform branch.

The return-only stub compiled 18 tests with one shape-equivalent pass and 17
body-dependent failures. Fresh pinned-Orca E2E metadata records success and a
nonzero 6,338,289-byte disposable G-code deleted without content ingestion.
Debug/`NDEBUG` helper output is byte-identical, and the repaired independent
MSVC model supplies complete behavior-named ordering vectors. A test-only local
type alias resolves warning-denying Clippy without an allowance. The final
one-at-a-time record kills fourteen runtime mutations, compiler-rejects one,
and truthfully discloses four bounded equivalent survivors; contaminated early
mutation evidence is excluded. Production hashes restore exactly. Focused
debug/release pass 18/18, O37/O36 pass 10/6, complete geometry passes 442/442,
Clippy/rustfmt pass, and both repaired initial implementation rereviews approve.

The exact documented candidate passes focused debug/release, complete geometry
and bounded predecessor regressions, workspace Nextest 6,080/6,080 with two
skipped, all-target check, warning-denying Clippy, rustfmt, four WASM checks,
two optimized builds, bindgen/export/JavaScript audit, static audit, and exact-
O37 rollback. Both local Playwright attempts failed before test code because
Chromium could not load `libglib-2.0.so.0`; neither was treated as a pass.
Implementation/documentation commits `04920e0`/`2d6154d` were pushed;
exact-SHA Tier-1 run `31303115603` passed all five jobs and both browser
executions at `2d6154d401c3c954bed69de6ba631a53af05f1a3`. O38 is released but
remains crate-private and inactive. It changes no typed Option, lifecycle,
adapter, fixture branch, golden expectation, or G-code byte. Public slicing
still consumes O26 and returns `ProjectSlicingIncomplete`.

Task 22O.39 locally implements the exact source-cited composition at pinned
`LayerRegion.cpp:262-308`. The crate-private helper borrows O36 anchors and
zones, mutates only O37 bridge angles, and calls O38 after supplied-order anchor
lookup, contour/hole conversion, exact `(1e-4 / scale.factor()) as f32`
Miter-3 expansion, and non-recombining open-path difference. It freezes one
forward cursor, source-width cast/wrapping behavior, direct Clipper error order,
earlier-angle commit, unchanged stored contour and hole buffers, and
`PI + atan2(y,x)` without lifecycle activation.

The repaired fresh-cycle RED contains 11 body-dependent failures and two
stub-equivalent passes; final focused debug/release is 14/14. The pinned
Debug/`NDEBUG` helper matrix is byte-identical with 12 passing assertions and
explicit multiple-bridge/missing-boundary coverage. Complete repeated/multi
literals, contour/hole pointer identity, M01-M28 mutation coverage, exact
restoration, and both implementation rereviews pass. Complete exact-final-byte
native/WASM/static/exact-O38 rollback verification also passes, including
workspace Nextest 6,094/6,094 with two skipped and warning-denying Clippy. Both
local Playwright attempts failed before test execution on missing
`libglib-2.0.so.0`; neither was treated as a pass. Implementation/documentation
commits `2038e93491de89e33f12ecb5379132a013bfc996` /
`c84119ee6871a176ec94117bc16f7e402c9caf96` were pushed, and exact-SHA Tier-1
run `31317150231` passed all five jobs and both browser executions at the
documentation SHA. O39 is released but inactive.

Task 22O.40 locally implements the source-cited `merge_bridges` boundary at
`LayerRegion.cpp:310-351`. The crate-private function consumes O37/O39 bridge
records and sorted O36 expansion records, resolves every bridge to its root,
flattens each member contour then holes and matching expansion contours/holes,
applies Miter-3 morphological closing per group, and emits default
`BottomBridge` surfaces with only the root angle. Its Rust ownership model
removes the C++ iterator field and uses temporary contiguous source ranges;
pointer identity, partial mutation, and malformed internal inputs are not API
contracts.

The compiling stub RED failed the first emitted-surface assertion. Eight O40
behavior tests pass, including exact contour/hole/order output from an
independently compiled pinned-Orca Debug/`NDEBUG` flat-closing oracle and a
three-bridge case that detects accidental global closing. O35-O40 focused
regressions pass 69/69; workspace Nextest passes 6,101/6,101 with two skipped;
warning-denying Clippy, rustfmt, native/wasm32 checks, diff, LOC, include, and
fixture-branch audits pass. The public normalized KSR golden still fails at the
pre-core CLI `--options` requirement. The same six-dimensional review thread
approved the repaired candidate with zero findings. O40 remains crate-private,
inactive, and unreleased. `expand_bridges_detect_orientations` at
`LayerRegion.cpp:395-437`, external-surface lifecycle integration, Options,
adapters, and downstream G-code remain deferred.

Six O41 tests cover the sorted zone-major composition, exact no-op and mixed
surface ownership, default output metadata, selective zone clipping, and first
and later expansion-error mutation order. Focused O41 passes 6/6 and all
external-surface regressions pass 53/53; workspace warning-denying Clippy,
rustfmt, diff, and LOC checks pass. The normalized KSR probe remains the
expected RED at the CLI `--options` boundary.

Workspace Nextest passes 6,107/6,107 with two skipped. The initial independent
review requested the direct sorting and first/later error-ledger tests; after
those repairs and complete gate reruns, the same six-dimensional reviewer
approved O41 without remaining findings.

## Task 22O.41 bridge-orientation orchestration boundary

O41 ports pinned
`LayerRegion.cpp:395-437::expand_bridges_detect_orientations` into one inactive
crate-private composition seam. It moves only `BottomBridge` geometry, invokes
the O36/O37/O39/O40 rewrite helpers in upstream order, sorts anchors and
expansions by the upstream keys, and clips only zones whose expansion stage set
`expanded_into`. The explicit `CoordinateScale` remains platform-neutral and
all Clipper errors propagate without fallback. This creates no new public API
or architecture decision; active `process_external_surfaces`, configuration,
adapters, and downstream G-code remain deferred.

## Task 22O.42 external-surface successor boundary

O42 replaces the inactive external-surface helper scaffold with one active,
deep crate-private successor for pinned
`LayerRegion.cpp:486-623::process_external_surfaces`. Its record boundary owns
the destructive surface normalization; its lifecycle adapter consumes
`PreparedPostHorizontalShellPropagation` immediately after O26, matching
`PrintObject.cpp:610-641`. The adapter resolves no new configuration: it uses
the surviving composed `RegionOptions`, integer-scaled Classic prelude fields,
record model rotation, global print spiral mode, and `CoordinateScale`.

The owned stage mutates records directly and drops/disposes the graph on a
Clipper failure. It deliberately does not clone the full project for rollback,
because no failed successor is externally observable. The only new surface
operation sets inherited thickness on a freshly defaulted `RegionSurface`.
Lower-layer covered-area caching is not part of this boundary because the
pinned active callee does not read it. Public slicing advances through O42 and
still terminates explicitly at the next missing upstream stage.

The repaired boundary is covered by 19/19 focused O42 tests, 72/72 complete
external-surface tests, and a 119/119 O24-O26/O40-O42 regression band. Public
activation and disposal are observed through `slice_project`; all 460 KSR
records carry independent traversal evidence; and a real 3MF minimum-area
mutation changes controlled adapter output. Workspace Nextest passes
6,126/6,126 with 27 slow and two skipped, while warning-denying Clippy,
rustfmt, WASM, diff, LOC, and include audits pass. The final independent
standards, specification, and upstream-parity re-review returned unconditional
approval with no findings.

## Task 22O.43 internal-bridge candidate successor boundary

O43 activates the first owned successor for pinned
`PrintObject.cpp:2467-2591::PrintObject::bridge_over_infill` after O42. The
pinned-disabled `clip_fill_surfaces` call remains a documented identity, not a
Rust lifecycle type. The successor stores stable candidate indices and owned
polygon paths while preserving the O42 graph for later source-cited bridge
angle and surface-commit slices.

The stage projects only existing composed inputs: each object's complete
effective-region set for Lightning, each aligned record's current solid
spacing, lower-region density, filter policy, and coordinate scale. It adds no
Option, parser default, public API, fallback, platform branch, or approximate
CrossHatch substitution. An aligned empty lower record contributes empty
geometry; absent physical lower links are skipped. Geometry failures consume
and dispose the unobservable predecessor rather than cloning it for rollback.

The repaired boundary passes O43 35/35, the O24-O26/O40-O43 band 154/154, and
workspace Nextest 6,161/6,161 with 27 slow and two skipped. Warning-denying
workspace Clippy, rustfmt, WASM, and static audits pass. Candidate discovery is
therefore available to the next exact `bridge_over_infill` rewrite slice, but
public slicing remains explicitly incomplete and the normalized golden remains
RED at the pre-core CLI contract.

The final independent standards and specification/upstream reviews approve
the repaired O43 boundary unconditionally with no remaining findings.

## Task 22O.44 boundary-connection dependency

O44 adds the crate-private source-owned `fill::connect` implementation of
pinned `Fill::connect_infill`. The interface owns ordered polylines, borrows an
ExPolygon boundary, receives source-typed anchor fields plus explicit
`CoordinateScale`, and returns exact connected/hooked polylines or the first
checked geometry error. Its stable-index graph hides Orca's pointer-linked
working state, and both active comparator-equivalent sorts reuse the audited
MSVC 14.44 control flow without a host sort or added tie-break.

This is intentionally a dependency, not another prepared-project checkpoint.
It does not consume O43, call the legacy infill scaffold, parse configuration,
or alter the public lifecycle. The 41 focused tests, 76-test dependency band,
194-test predecessor band, 6,201-test workspace run, warning-denying Clippy,
rustfmt, WASM, static audits, restored Orca harnesses, and repaired independent
reviews pass. The public KSR path remains terminal after O43 until a later
source-cited slice ports complete CrossHatch generation and its anchor-map
transaction.

## Task 22O.45 CrossHatch fill-surface dependency

O45 adds the dependency-first crate-private `fill::cross_hatch` transaction
for pinned public `Fill::fill_surface`, complete `FillCrossHatch`, the active
O44 dispatch, and multiline one. Its deep owned-result interface accepts one
raw ExPolygon plus already-resolved source-typed fields and explicit
`CoordinateScale`, then owns inset-component order, lattice generation, open
clipping, strict filtering, O44 connection, and rotate-back. The legacy
`infills` scaffold remains an uncalled compatibility shell.

Exact public Orca, raw-pattern-order, and arithmetic harnesses cover both
scales and source arithmetic/order. The LargeBed fixture correction preserves
`scaled<coord_t>` truncation toward zero, and all four LargeBed cases pass.
Sixteen reversible arithmetic/composition/f32-repeat-ratio mutants are RED and
production is restored byte-for-byte. O45 passes 34/34 focused tests, its
305-test dependency band, the 228-test predecessor band, and the 6,235-test
workspace run with 30 slow and two skipped; warning-denying Clippy, rustfmt,
wasm32, and static audits pass. Final independent source/specification and
standards reviews unconditionally approve this implemented and gate-verified
state.

This dependency does not consume O43 or create a public prepared checkpoint.
Public slicing still disposes O43 and returns `ProjectSlicingIncomplete`, and
the unchanged missing-`--options` golden probe remains the expected RED. O45
therefore makes no public option, activation, or G-code parity claim. O46 is
scheduled to port the public Layer anchoring result with its exact-corpus
bridge-angle/pattern grouping plus nominal sparse Flow/angle projection.
Complete generic `group_fills`, the
transaction-local lower-layer anchor map, Lightning generation, and later
bridge/fill/toolpath/motion/G-code/CLI behavior remain source-cited future
slices.

## Task 22O.46 sparse-anchoring dependency

O46 implements the crate-private rewrite of pinned
`Layer::generate_sparse_infill_polylines_for_anchoring`, returning only final
ordered sparse polylines. Its private implementation processes every retained
KSR group through mutual priority geometry before filtering `Internal`, using
the KSR-observable decreasing f32 bridge-angle and explicit pattern-rank ordered
coalescing, then calls O45 with exact nominal sparse Flow, angle, density,
anchors, accumulated Z, zero overlap, explicit scale, and `dont_sort=false`.

The seam does not create a prepared successor or production lower-layer map.
Public slicing remains terminal at O43. Generic grouping, post-priority
non-sparse repair, other patterns, adaptive/Lightning state, map ownership,
bridge commit, extrusion, motion, G-code, and CLI activation remain deferred.
The strict global fixed-MSVC proof rebuilt 209 affected objects per mode and
confirmed Debug/Release identity: 103 calls, 1,507 endpoint records/zero ties,
1,439 arc records/2,700 ties across 30 calls and 82 classes, and the normative
186-path / 5,941-point ordered digest `917adc6e...`. The exact per-layer table
has SHA-256 `bf531afc...`; the previous Linux, hybrid, and captured-input
results are diagnostic or rejected. Final focused 6/6, dependency 625/625, and
workspace 6,241/6,241 Nextest runs, rustfmt, warning-denying workspace Clippy,
core/browser wasm32, diff/LOC/static audits, and the unchanged ignored golden
progress probe pass; the 18-case reversible mutation audit is fully killed and
restored byte-exact; independent source/specification and standards rereviews
approve unconditionally.

## Task 22O.47 deep sparse bridge-area dependency

O47 consumes the already-resolved embedded `sparse_infill_density` attached to
each retained lower-layer view and ports pinned
`PrintObject.cpp:2819-2846::gather_areas_w_depth`. The operation uses no raw
configuration map or external option file: it receives typed density, planned
`print_z`, post-O42 surfaces, caller-resolved target bridge height, and the
existing coordinate scale.

The 18-layer KSR regression also derives its target height from embedded
`bridge_line_width`, `bridge_flow`, and `nozzle_diameter`, then freezes 115
flat Polygons / 5,641 points and ordered SHA-256
`f28db7dd3fc63155752ba5c33d4cd6338b2e311d83eb973c473d7f65268aa92a`.
Focused tests prove per-layer density changes affect classification rather than
a fixture-name or layer-index branch.

This remains an unwired dependency. The later bridge transaction is deferred;
no fallback, external options, public lifecycle activation, or G-code output is
introduced.

## Task 22O.48 thick solid-infill bridge Flow dependency

O48 consumes effective embedded `internal_solid_filament_id`,
`bridge_line_width`, `bridge_flow`, and `nozzle_diameter` through the typed
project graph. The source-cited `frSolidInfill, thick_bridge=true` resolver
preserves selector element-zero fallback, percent/absolute width evaluation,
f64 square root followed by f32 multiplication, circular thread spacing, and
volume bits in the existing Flow model.

The O47 KSR regression now derives target height exclusively through O48 and
retains its exact ordered geometry digest. Final verification passes focused
7/7, combined O47/O48 16/16, dependency 597/597, and workspace 6,257/6,257
Nextest, warning-denying workspace Clippy, rustfmt, core/browser wasm32, diff,
LOC, and static audits. Independent six-axis repair/re-review ends in
unconditional approval. No raw option map, external JSON, fixture identity
branch, legacy Flow, or lifecycle activation is introduced.

## Task 22O.49 internal bridge angle override dependency

O49 consumes effective embedded `internal_bridge_angle`,
`relative_bridge_angle`, and `align_infill_direction_to_model` through the O43
candidate region and uses the already-retained print-object occurrence rotation
from `PerimeterInputRecord`. It ports pinned `PrintObject.cpp:3253-3267` with
exact `PI * degrees / 180` ordering, relative addition, absolute replacement,
and absolute-only model alignment.

Three KSR tests traverse all 43 candidates. The default archive remains exact
zero pass-through; two semantic archive mutations enable the same pi/2
occurrence rotation and positive angle, proving that absolute mode adds the
rotation while relative mode ignores it. Final verification passes focused
8/8, dependency 605/605, and workspace 6,265/6,265 Nextest,
warning-denying workspace Clippy, rustfmt, core/browser wasm32, diff, LOC, and
static audits. Independent six-axis repair/re-review ends in unconditional
approval. No raw option map, fixture identity branch, legacy
`InfillOptions` fallback, lifecycle activation, or G-code output is introduced.

## Task 22O.50 nearest anchor-line tree dependency

O50 ports the balanced indexed-line tree reached by
`PrintObject.cpp::determine_bridging_angle`, including exact implicit layout,
median-of-three QuickSelect, pinned-Eigen bbox distance conversions, recursive
traversal and strict tie ownership, and source-order line projection. It
introduces no option: the tree borrows existing geometry lines and returns the
nearest original index, squared distance, and projected point.

Literal tests derived from a standalone driver using the actual pinned Orca
and Eigen templates cover empty/degenerate/interior projections, equal
centroids, non-power-of-two layouts, containment, right-first bbox ties,
coordinates above 2^53, and `HI_RANGE`. Final gates pass focused 8/8,
dependency 613/613, and workspace 6,273/6,273 Nextest, warning-denying Clippy,
rustfmt, core/browser wasm32, diff, LOC, and static audits. The seam remains
crate-private and unwired; automatic direction aggregation and the bridge
transaction remain deferred.

## Task 22O.51 automatic bridge-angle dependency

O51 consumes no new option surface. It accepts the already typed dominant
`ProcessInfillPattern`, runtime coordinate scale, borrowed bridge polygons, and
borrowed anchor lines, then ports pinned
`PrintObject.cpp::determine_bridging_angle` on O50. Exact integer-scaled 2-mm
sampling, Eigen/f32 cast order, ordered numeric direction buckets, periodic
windows, strict score ownership, fallback, and Hilbert/Octagram adjustments are
preserved.

Final gates pass focused 9/9, dependency 622/622, and workspace 6,282/6,282
Nextest, warning-denying Clippy, rustfmt, core/browser wasm32, diff, LOC, and
static audits. No raw option map, fixture identity branch, host sort/hash
bucket, public lifecycle activation, or G-code output is introduced.

## Task 22O.52 indexed line query dependency

O52 consumes no new option surface. It extends O50's borrowed line tree with the pinned `intersections_with_line<true>` and `outside` operations required by `PrintObject.cpp::construct_anchored_polygon`, including original-line indices, truncating source intersection arithmetic, fixed-MSVC equal-key sorting, shared-vertex ownership, and exact X/Y parity fallback.

Final gates pass focused 8/8, dependency 630/630, and workspace 6,290/6,290 Nextest, warning-denying Clippy, rustfmt, core/browser wasm32, diff, LOC, and static audits. Five reversible mutations are killed and production restored byte-exact. No raw option map, external input, host sort, public lifecycle activation, or G-code output is introduced.

## Task 22O.53 anchored bridge polygon dependency

O53 consumes no new option surface. It accepts the existing typed O48 `Flow`, runtime coordinate scale, borrowed bridge polygons and anchor lines, then ports pinned `PrintObject.cpp:2939-3111::construct_anchored_polygon` on O50-O52. Exact rotation, centered scanlines, anchor extension, fixed-MSVC section ordering, trace identity, flat Paths safety union, and inverse rotation are preserved.

Pinned-C++/fixed-MSVC-replayed normal and large-scale literals match exactly. Final gates pass focused 20/20, dependency 650/650, workspace 6,310/6,310, warning-denying Clippy, wasm32, Windows/macOS cross-checks, rustfmt, diff, LOC, and static audits. Nineteen mutations are killed and restored byte-exact. No raw option map, external input, fixture identity branch, lifecycle activation, or G-code output is introduced.

## Task 22O.54 bridge candidate-layer clustering dependency

O54 consumes O43 candidate geometry, planned layer Z, runtime coordinate scale,
and the effective region-zero O48 thick solid bridge Flow. It ports pinned
`PrintObject.cpp:2763-2818`: sequential flat union of rounded 7-mm inflated
candidate AABBs followed by exact strict Z-gap and previous-tail intersection
clustering.

Actual-source/fixed-MSVC-order normal and large-scale literals match exactly.
Final gates pass focused 11/11, dependency 661/661, workspace 6,321/6,321,
warning-denying Clippy, wasm32, Windows/macOS cross-checks, rustfmt, diff, LOC,
and static audits. Fifteen mutations are killed and restored byte-exact,
including raw-nozzle, ignored bridge-width, and ignored bridge-flow bypasses. No raw
option map, candidate-region flow selection, scheduler, terminal adapter,
lifecycle activation, or G-code output is introduced.

## Task 22O.55 bridge candidate ordering dependency

O55 consumes no new option. It moves owned O43 candidates through pinned
`PrintObject.cpp:3127-3153` ordering: fixed-MSVC minimum-X/minimum-Y first sort,
then stable tail distance ordering from the post-sort front maximum. Task-local
source-shaped bounding keys preserve undefined extent semantics, and payload
allocations remain unchanged.

Pinned candidate/BoundingBox/Eigen/fixed-MSVC literals match exactly. Final
gates pass focused 12/12, dependency 673/673, workspace 6,333/6,333, strict
Clippy, wasm32, Windows/macOS, formatting/static checks, and thirteen
killed/restored mutations. No raw option map, host-dependent first sort,
geometry clone, lifecycle activation, or G-code output is introduced.

## Task 22O.56 lower-cluster bridge subtraction dependency

O56 consumes no new option surface. It accepts the transaction-composed,
already-promoted O55-front candidate-region target height and borrowed
postprocessed history from earlier jobs in the same O54 cluster. It ports
pinned `PrintObject.cpp:3160-3179`: exact bottom-Z arithmetic, reverse inclusive
history traversal, source-order flattening, and one unconditional flat
difference.

Pinned-source/fixed-MSVC-order literals match exactly. Final gates pass focused
10/10, dependency 683/683, workspace 6,343/6,343, strict Clippy, wasm32,
Windows/macOS, formatting/static checks, and ten killed/restored mutations. No
raw option map, pre-expansion-history fallback, repeated difference, lifecycle
activation, or G-code output is introduced.

## Task 22O.57 current-layer bridge expansion context dependency

O57 consumes only typed `ProcessInfillPattern` ownership from each current-layer
region plus transaction-composed O46 lower-layer lines, O56 deep area, O48
front-candidate-region scaled spacing, and `CoordinateScale`. It ports pinned
`PrintObject.cpp:3181-3205`: exact expansion/shrink arithmetic, Top/Internal/
InternalSolid/all-fill/Lightning selection, scale-epsilon closing, deep
intersection, and ordered anchor clipping. Spacing is strictly positive, and
source epsilon is computed directly from the coordinate-scale factor without an
integer intermediate.

No raw option map, default option fallback, pattern inference, filesystem input,
lifecycle activation, or G-code output is introduced. Pinned actual-source
ordered literals match exactly. Final gates pass focused 15/15, dependency
698/698, workspace 6,358/6,358, strict Clippy, wasm32, Windows/macOS,
formatting/static checks, and nineteen killed behavioral mutations including
both operation-order transformations.

## Task 22O.58 candidate bridge area filtering dependency

O58 consumes no new option surface. The future composer supplies the current
O55 candidate's O43 polygons, O57 deep/unsupported/expansion geometry, and that
candidate region's O48 `scaled_spacing()`. The private seam ports pinned
`PrintObject.cpp:3215-3224` without raw-option lookup, fallback Flow, pattern
inference, lifecycle activation, or G-code output.

Removed actual-source literals match exactly; fifteen mutations, including
repeated-union and two competing-error-order variants, were killed and source
restored byte-exact. Final gates pass focused 10/10, dependency 708/708,
workspace 6,368/6,368, strict Clippy, wasm32, Windows/macOS,
formatting/static/clean-Orca/no-staged checks, and independent six-axis
re-review approval.

## Task 22O.59 candidate boundary polyline dependency

O59 consumes no new option surface. The future composer supplies the O58
candidate area and the selected candidate region's exact O48-derived
`scaled_spacing()` and `spacing()` values. The private seam ports pinned
`PrintObject.cpp:3226-3233` without raw-option lookup, fallback Flow, pattern
inference, lifecycle activation, or G-code output.

Removed actual-source literals match exactly; nineteen mutations, including
explicit ascending output sorting, were killed and source restored byte-exact. Final gates pass focused 10/10, dependency
718/718, workspace 6,378/6,378, strict Clippy, wasm32, x86_64/aarch64 Windows
and macOS, formatting/static/clean-Orca/no-staged checks.

## Task 22O.60 candidate bridge angle composition dependency

O60 consumes no new raw option surface. The future composer supplies O58
`area_to_be_bridge`, O57 anchors, O59 fallback boundaries, the candidate
region's already-resolved `sparse_infill_pattern` plus O49 override options,
retained `model_rotation_rad`, and coordinate scale. The private seam ports
pinned `PrintObject.cpp:3242-3267` and `Polyline.hpp:169-193` without inferring
Flow/options, reviving the source's commented-out `infill_direction` behavior,
activating lifecycle, or emitting G-code.

The real-KSR test traverses all 43 O43 candidates to the resolved CrossHatch
region and retained rotation, then verifies exact O60 output bits and complete
input preservation. Focused 7/7, dependency 2,354/2,354, workspace
6,385/6,385, strict lint/format, five portability builds, and nineteen mutation
kills pass; final independent six-axis re-review approved unconditionally.

## Task 22O.61 candidate anchored bridge dependency

O61 consumes no new raw option surface. The future composer supplies O57 anchors
and Lightning area, O58 bridge area, owned O59 boundaries, exact O48 Flow, O60
angle, and retained coordinate scale. The private seam ports
`PrintObject.cpp:3268-3272` without inferred options, lifecycle activation, or
G-code output; collision and surface commit remain deferred.

Real-KSR provenance reaches the embedded CrossHatch region, exact O48 Flow, O60
angle, and retained scale without input mutation. Focused/KSR 9/9, dependency
2,363/2,363, workspace 6,394/6,394, strict gates, portability, and twenty-three
mutation kills pass; final independent six-axis implementation review approved
unconditionally.

## Task 22O.62 candidate collision reconstruction dependency

O62 consumes no new raw option surface. It receives exact O48 Flow, O61 owned
boundaries/initial bridge polygons, borrowed prior-completed O43-shaped records,
original candidate area, current angle, and retained coordinate scale from the
future composer. Those records must contain `new_polygons` history postprocessed
at source lines `3292-3297` and appended at `3304-3305` in exact append order;
raw O43 candidate geometry is forbidden and producing history remains deferred. The
private seam ports `PrintObject.cpp:3274-3288` without option inference,
postprocessing, commit, lifecycle activation, or G-code output.

Focused 8/8, dependency 2,371/2,371, workspace 6,402/6,402, strict gates, five
portability builds, and 26/26 mutation kills pass. Final independent six-axis
implementation review approved unconditionally.

## Task 22O.63 bridge postprocessing dependency

O63 consumes no new raw option surface. It receives exact O48 Flow, owned O62
collision state and expansion area, borrowed limiting/total-fill/total-top
areas, and retained coordinate scale from the future composer. The private seam
ports `PrintObject.cpp:3290-3298` without option inference, candidate commit,
lifecycle activation, or G-code output.

Focused 7/7, dependency 2,378/2,378, workspace 6,409/6,409, strict gates, five
portability builds, and 25/25 mutation kills pass. Final independent six-axis
implementation review approved unconditionally.

## Task 22O.64 bridge candidate commit dependency

O64 consumes no new raw option surface. It receives O43 stable candidate
identity and successful owned O63 state from the future composer. The private
seam ports `PrintObject.cpp:3304-3310` candidate append and per-layer swap/clear
without option inference, geometry work, second-pass behavior, lifecycle
activation, or G-code output.

Focused 6/6, dependency 2,384/2,384, workspace 6,415/6,415, strict gates, five
portability builds, and 16/16 mutation kills pass. Final independent six-axis
implementation review approved unconditionally.

## Task 22O.65 bridge rewrite-area dependency

O65 consumes no new raw option surface. The future composer supplies O64
current/upper committed candidates, each upper candidate's Task 22N-resolved
normal solid-infill Flow, and retained object scale. The future composer only
projects source/layer/region identity to that record. The private seam ports
`PrintObject.cpp:3318-3319,3322-3336` without option inference, map traversal, region
rewrite, lifecycle activation, or G-code output.

Focused 9/9, dependency 2,393/2,393, workspace 6,424/6,424, strict gates, five
portability builds, and 24/24 compiling mutation kills pass. Final independent
six-axis implementation review approved unconditionally.

## Task 22O.66 region bridge ensuring-area dependency

O66 consumes no new raw option surface. The future composer supplies all current
region fill surfaces, O65 `additional_ensuring_areas`, the region's exact Task
22N-resolved normal solid-infill Flow, and retained object scale. The private
seam ports `PrintObject.cpp:3341-3343` without option inference, surface-kind
filtering, region mutation, lifecycle activation, or G-code output.

Focused 6/6, dependency 782/782, workspace 6,442/6,442, strict and five
portability gates, and 18/18 compiling mutation kills pass. Final independent
review is pending.

Internal infill/solid rebuilding, bridge conversion, region replacement, second
pass, composer, CLI, and full golden parity remain deferred.

Focused 12/12, dependency 776/776, workspace 6,436/6,436, strict gates, five
portability builds, and 30/30 compiling mutation kills, including safety
difference/intersection, pass. Final independent
six-axis implementation review approved unconditionally.

## Task 22O.67 internal infill rebuild dependency

O67 consumes no new raw option surface. The future composer supplies current
region surfaces plus exact O65 cut and O66 ensuring geometry. The private seam
ports `PrintObject.cpp:3345-3350` with two default no-safety differences and
fresh default-metadata Internal records, without option inference, region
mutation, lifecycle activation, or G-code output.

## Task 22O.68 internal bridge surface conversion dependency

O68 consumes no new raw option surface. The future composer supplies current
region surfaces and the exact O64 committed candidate history. The private seam
ports pinned `PrintObject.cpp:3352-3367`: stable region/source-index matching,
default-NonZero candidate union, source metadata copying, InternalBridge retag,
angle replacement, and ordered fresh output without region mutation, lifecycle
activation, or G-code output.

Focused 6/6, dependency 788/788, workspace 6,448/6,448, strict gates, five
portability builds, and 14/14 compiling mutation kills pass. Solid recomposition,
composer, CLI, and full golden parity remain deferred.

## Task 22O.69 internal solid recomposition dependency

O69 consumes no new raw option surface. The future composer supplies current
region surfaces plus exact O65 cut and O66 additional-ensuring geometry. The
private seam ports pinned `PrintObject.cpp:3368-3374`: stable InternalSolid
selection, ordered ensuring append, one no-safety difference, one safety union,
and fresh default-metadata InternalSolid output without region mutation,
lifecycle activation, or G-code output.

Focused 6/6, dependency 794/794, workspace 6,454/6,454, strict gates, five
portability builds, and 26/26 compiling mutation kills pass. Region commit,
composer, CLI, and full golden parity remain deferred.

## Task 22O.70 region bridge surface commit dependency

O70 consumes no new raw option surface. The future composer supplies the
current region's owned fill collection and the complete borrowed O67/O68/O69
rebuilt sequence. The private seam ports pinned `PrintObject.cpp:3385-3386`:
stable removal of prior InternalSolid/Internal surfaces followed by ordered
copy-append, without option inference, geometry, lifecycle activation, or
G-code output.

Focused 3/3, workspace 6,457/6,457 with two skipped, strict gates, five
portability builds, and 15/15 compiling mutation kills pass. Composer wiring,
the second internal-bridge pass, CLI, and full golden parity remain deferred.

## Task 22O.71 first internal bridge transaction dependency

O71 consumes no new raw option type. It composes the already typed object and
region options, nozzle diameters, planned layers, normal solid-infill Flow, and
O43 candidate provenance through pinned `PrintObject.cpp:2725-2761,3114-3389`.
The active rewrite boundary supports the fixture-reachable single-region,
non-Lightning CrossHatch path and rejects an active unported sparse anchoring
pattern instead of substituting a fallback. Lightning/adaptive/support-cubic,
generic other-pattern generation, and the optional second internal-bridge
pass remain explicitly deferred.

The transaction boundary rejects only active adaptive/support-cubic octree
states; density-zero or objects without nonempty fill surfaces retain Orca's
inactive no-op. It also rejects active unported anchor templates, direction
controls, surface kinds, densities/lengths, and effective extruder ordering at
the external project boundary, rather than silently applying the reduced KSR
grouping model. No new raw option or compatibility fallback was introduced.

Final evidence is focused O71 16/16, bridge dependency 240/240, workspace
6,473/6,473 with two skipped, strict Clippy/rustfmt/diff, WASM plus two Windows
and two macOS checks, and unconditional independent review. The KSR prepared
surface checkpoint has 47 `InternalBridge` surfaces, 15,689 ordered points,
17 bridge-bearing planned layers, and SHA-256
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`.
This does not claim complete G-code parity; the public lifecycle remains
`ProjectSlicingIncomplete` after O71 disposal.

## Task 22O.72 infill-combination identity-gate parity

O72 introduces no raw option or option type. It reads the exact already typed
per-region `infill_combination: OrcaBool` and
`sparse_infill_density: Percent` retained by the project-slice graph and ports
the source branch at pinned `PrintObject.cpp:4172-4174`. Effective global,
object, and part overrides are resolved before this region-level decision:

```text
!infill_combination || sparse_infill_density == 0.0  => unchanged successor
 infill_combination && sparse_infill_density != 0.0  =>
     UnsupportedProjectFeature("infill_combination")
```

The comparison is exact, not epsilon-based and not `<= 0`. Enabled combination
with exact-zero density is admitted as the source identity branch. Disabled
combination does not activate or gate on
`infill_combination_max_layer_height`, because pinned Orca reads that option
only inside the deferred active body at `4188-4190`; existing typed parsing and
validation remain unchanged.

The public exact-zero path retains O43's real candidate inventory. Following
`PrintObject.cpp:2737-2753` and `Fill/Fill.cpp:855-902,1394-1508`, O71 produces
an empty sparse-anchor line set for that density and continues with its existing
boundary-derived bridge angle; it does not erase candidates or make the whole
bridge transaction a no-op.

The embedded KSR project carries `infill_combination = "0"`,
`infill_combination_max_layer_height = "100%"`, and
`sparse_infill_density = "15%"`. It therefore crosses O72 unchanged and retains
the O71 digest
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`.

The older `InfillOptions` projection and
`crates/ares-core/src/infills/combination.rs` scaffold are not option-parity
evidence and are not used as a fallback. Full active parity for
`infill_combination_max_layer_height`, sparse/internal-solid filament IDs,
nozzle selection, pattern-dependent clearance, flows, and surface rewrites is
deferred with `PrintObject.cpp:4176-4287`. Until that exact source body is
ported, enabled nonzero-density projects remain explicitly unsupported.

Public slicing invokes O72 and still returns `ProjectSlicingIncomplete` after
disposing `PreparedPostInfillCombination`; no placeholder G-code is emitted.

Final evidence passes focused 14/14, prepare-infill 255/255, and workspace
6,486/6,486 with two configured skips. Six compiling mutations, including the
promoted Orca `0.00011f` normalization threshold, were killed and byte-exactly
restored. Strict Clippy/rustfmt, six Tier-1 target checks,
LOC/static/diff/no-staged gates, and a clean pinned Orca worktree pass.
Complete G-code parity remains deferred.

## Task 22O.73 base fill-grouping parity

O73 introduces no raw option, parser, or public option type. Its
single graph-native seam borrows `PreparedPostExternalSurfaces` and projects
the already typed effective object, print, and region values used by pinned
`Fill/Fill.cpp:829-1067`. Global, object, part, and material inheritance remain
owned by the existing option materialization stages; callers must not rebuild
an `InfillOptions` or raw config map.

The admitted projection consumes these existing typed values where selected by
the source surface role:

- print nozzle diameters and initial-layer line width;
- object layer height, generic line width, `thick_bridges`, and
  `thick_internal_bridges`;
- sparse, top, bottom, and internal-solid patterns and densities;
- sparse and solid directions plus their rotation-template strings and
  `align_infill_direction_to_model`;
- sparse, internal-solid, top, and bottom filament selectors;
- sparse, internal-solid, top, bridge, skin, and skeleton line widths,
  bridge flow ratio, and the current/effective surface thickness;
- sparse, internal-solid, top, bridge, and percent-valued internal-bridge
  speeds;
- fill multiline, infill anchors, both lateral lattice angles, symmetric-Y,
  lock/skin depths and densities, overhang angle, and Gyroid optimization.

No option is reinterpreted as an Ares-specific grouping control. Source Flow
role is chosen from surface classification before the eventual extrusion role:
top uses top-solid Flow, all other solids and bridges use solid-infill Flow,
and sparse internal uses infill Flow. Top, bottom, and internal-solid output
roles may then override the group filament selector without changing the Flow
role already chosen. Filament selectors stay one-based in
`SurfaceFillParams`; only nozzle-vector lookup subtracts one.

First-layer width selection, role-width fallback, float-or-percent conversion,
and automatic widths follow `PrintRegion.cpp:25-53` and
`Flow.cpp:20-36,129-143`. In particular, automatic top-solid width is one
nozzle diameter while automatic sparse/solid width is the source
`1.125f * nozzle`. Standard bridge Flow applies the bridge ratio while
preserving ordinary Flow spacing and `flow.bridge == false`; thick round bridge
Flow has `flow.bridge == true`. `params.bridge` is a separate group-key field.

Role speed is zero except for the exact source roles. Internal bridge speed is
resolved from its float-or-percent value against bridge speed before the f32
group field is stored. Solid/bridge anchors become 1000 mm. Sparse anchor and
maximum are cast to f32 before percent multiplication by nominal sparse
spacing, then the anchor is clamped with source `std::min` first-operand
identity for signed-zero equivalence. `overlap` remains source default zero
throughout this slice.

Pattern and extrusion-role order use explicit pinned ranks; Rust enum layout is
not option-parity evidence. The result pattern wraps existing
`ProcessInfillPattern` and reserves a slice-private `ConcentricInternal` value
for O74. It does not add that synthetic source value, `ipSupportBase`, or
`ipCount` to the user-configurable process enum, and it does not reuse the
incomplete legacy public `InfillPattern`.

Empty rotation-template strings use direct configured degrees-to-radians
conversion. Nonempty `sparse_infill_rotate_template` and
`solid_infill_rotate_template` remain explicitly unsupported at O73 because
the pinned metalanguage and random/pseudorandom joints at
`Fill.cpp:25-214` are not yet ported. The older simple-list parser, legacy
`InfillOptions`, and a host RNG are not fallbacks. Model-direction alignment is
applied from the already materialized transform rotation with the source f32
cast order.

LockedZag consumes its existing typed lock, skin, skeleton, width, and density
options to produce the source sidecars. Its density-map identity is ordinary
f32 order, while its Flow-map identity is only `mm3_per_mm`; neither may be
substituted with full Flow equality. The layer-wide params record intentionally
retains source conditional assignment: lock/skin depth and symmetric-Y can
remain sticky when a later surface changes pattern.

The existing public `multi_region_layer_slices` gate remains the owner of
multi-region rejection. O73 preserves source-shaped region and no-overlap
fields for the admitted single region but does not claim multi-region graph,
override, ordered joining, or union parity and must not fall back to region
zero. `detect_narrow_internal_solid_infill` is not consumed by O73; the
complete O74 tail is `Fill.cpp:349-827,1069-1186`, including its KSR-active
narrow-solid behavior at `1152-1186`.

Adaptive, support-cubic, Lightning, and other configured patterns may retain
their grouping metadata, but generator creation and their octree/generator
options remain downstream caller behavior. O73 adds no filler dispatch,
extrusion, motion, G-code, or CLI option-consumption claim.

The verified O73 KSR result uses pre-narrow metadata SHA-256
`a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`
and canonical geometry SHA-256
`062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
plus layer-table SHA-256
`ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`,
over 460 preserved layer slots. Its aggregate contract is 477 groups, 1,882
fill ExPolygons, 174 fill holes, 2,056 fill paths, 107,540 fill points, and
2,547 no-overlap ExPolygons. The fill totals exclude the no-overlap section;
the canonical geometry digest includes both. It must also distinguish 33
groups with `params.bridge` from 22 with bridge-flagged Flow. These are
acceptance values under O38's fixed-MSVC bridge-direction policy; the complete
Linux PRE/POST provenance is recorded in the O73 ADR and specification and is
nonnormative. Explicit `assert_ne!` witnesses reject the distinct O74
aggregate totals and each of its three hashes.

Final exact-tree evidence passed focused `task22o73` 19/19 with 6,451 skipped,
prepare-infill 277/277 with 26 slow and 6,193 skipped, and workspace
6,508/6,508 with 27 slow and two configured skips. Strict workspace
all-target/all-feature Clippy with `-D warnings`, rustfmt, diff, all six Tier-1
checks, zero-staged/Cargo-unchanged/forbidden-production/lifecycle-static
checks, and clean pinned Orca at
`8500fcdccaa10b5099ac20d252af3a7c560046f1` passed. All changed/new Rust files
remained below 400 LOC: `project_slice.rs` was the maximum changed file at 381
LOC and `group_fills/params/projection.rs` was the maximum new production
shard at 369 LOC. Thirty-one compiling behavioral mutations were killed and
byte-exactly restored; one additional contour/hole insertion-order mutation was
a behaviorally equivalent survivor on normalized valid ExPolygons and was not
counted as a kill. The nine restored production hashes matched the exact
manifest recorded in the O73 ADR, specification, and plan. Independent
source/specification and standards rereviews closed unconditionally. Public
slicing remains `ProjectSlicingIncomplete` at O72, O73 remains crate-private
and unwired, and O46 replacement remains deferred to a later activation slice
even though O74 now completes the full grouping tail.

## Task 22O.74 implemented full fill-grouping parity

O74 is implemented, with exact-tree final gate counts and unconditional
independent review pending. It consumes no new raw option, parser, or public
option type. Its one graph-native seam borrows
`PreparedPostExternalSurfaces`; the existing option materialization graph
remains the sole owner of global, object, part, and material inheritance:

```rust
project_slice::group_fills::group_fills(
    &PreparedPostExternalSurfaces,
    object_index,
    layer_index,
) -> Result<GroupedFills, SliceError>
```

The only newly reached configuration read is the effective object boolean
`detect_narrow_internal_solid_infill` at pinned
`Fill/Fill.cpp:1152-1154`. False returns the complete O73 base behavior through
the same full seam; true runs the pinned line/non-line narrow classifier at
`349-827` over the original prefix of InternalSolid groups and applies the
source mutation/append at `1155-1186`. The option is not reinterpreted as a
raw Ares control and is never passed as a separate argument.

The old `crates/ares-core/src/infills/narrow_internal.rs` behavior—one
rectangle-width heuristic followed by a path generator—is not option parity
for this source slice. O74 neither calls it nor uses it as a fallback. The O46
reduced sparse-anchoring grouping is likewise not reused; O74 only makes a
future replacement at source `Fill.cpp:1394-1407` possible and does not modify
that caller.

`SurfaceFillParams` carries source `idx`. The value is assigned as the
comparator-order base-group ordinal at `Fill.cpp:1020-1024`, remains excluded
from the grouping comparator, and is copied unchanged into a partial appended
narrow group at `1172-1174`. It is source identity, not a configurable option
and not necessarily the final vector index. POST metadata must serialize this
field directly.

The InternalVoid body at `1069-1150` consumes no additional option in O74. Its
inner repair is unreachable for source-produced groups: `855-861` observes but
does not project voids, `1028-1051` excludes them from group construction, and
`1086-1097` searches only the filtered group result. Implemented parity is the
observable no-op. Raw void surfaces must not be smuggled into the tail to make
the commented repair intent active.

The KSR true-option acceptance preserves 460 ordered layer slots and
requires 536 groups, 2,218 fill ExPolygons, 152 holes, 2,370 fill paths,
110,610 fill points, and 2,928 no-overlap ExPolygons. Under O38's fixed-MSVC
direction-map replay, the required metadata, canonical-geometry, and
layer-table SHA-256 values are respectively
`cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387`,
`c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c`, and
`8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2`.
The Linux libstdc++ POST triplet—metadata
`36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff`,
geometry
`13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c`, and
table `15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a`—is
nonnormative provenance for the predecessor tie-order difference.

The false-option regression retains O73's full 460-layer PRE totals and hashes
`a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`,
`062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
and `ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`
through `group_fills`; POST remains the O74 success target. Canonicalization is
confined to the oracle and may not alter production order.

Raw-order POST evidence pins layer-1 metadata
`b466abfd76770f5e776b9df3866cf12b07b836bee2a8a7ba721c66ae1f2851bf`,
layer-1 authoritative geometry
`0938758d43750be165712735f6f5e1b6a1ae8fbb52a7f551b101118e1083c856`,
and layer-45/layer-70 authoritative geometry
`33bf737e3d836096a20a821fcf1ace79dccda10973203408ba87ddee5ee25d64` /
`7a8e9ec6e0aa2b1a8cd6bd8d1e9c261719b77168427f113fa051e7f5c551be71`.
The fixed-MSVC source-backed table rows are:

```text
1\t2\t29\t0\t723\t5,5\t0,29\t5,5
45\t4\t75\t15\t29423\t6,5,0,4\t0,29,1,20\t10,5,6,4
70\t8\t70\t0\t626\t2,6,6,6,6,6,5,4\t0,0,0,0,0,0,29,20\t9,10,10,10,10,10,5,4
```

The layer-45/layer-70 geometry hashes above use the same source-backed ordered
raw records, not canonical-sort substitutes.

The source-backed C++ oracle grammar deliberately omits
`Flow::mm3_per_mm`. Rust-only focused tests assert its exact `f64::to_bits()`
values, including partial-split copy value `0x3fbb_4fc3_4000_0000`. This
protects Rust grouping/ownership without changing either aggregate hash
triplet or claiming a C++ grammar field that does not exist.

The public-seam corpus killed the vibration-filter identity substitution,
`4 mm -> 3 mm`, maximum skips `2 -> 1`, exact two-skip `>= 2 -> > 2`,
removal depth `> 5 -> >= 5`, exact `4 mm` `< -> <=`, touch-back removal,
final normal expansion `0.5 * spacing -> 0`, a zero non-line closing delta,
and hard-coded Normal scale. The KSR checkpoint specifically killed the
filter/threshold/skip/depth/touch-back/final-expansion subset; graph-native
focused tests killed exact-4-mm, zero-closing-delta, and hardcoded-scale
changes. The two skip
mutations produced 2,223 / 2,375 / 110,582 and 2,217 / 2,369 / 110,597
fill-ExPolygon/path/point totals. Next-section reset removal,
inclusive-Y-to-strict-Y, `558-559` correction, `candidates_begin` correction,
early-closure removal, reconnection `< -> <=`, one-coordinate-unit non-line
spacing, and premature f32 scale/cast changes survived; they remain by
pinned-source/static review and are not reported as kills. FIFO/LIFO
pending-order and duplicate-queue cases are monotone-closure/static-review
cases.

O74 removes `group_fills_base`, `BaseGroupedFills`, and the returned
InternalVoid continuation bit rather than preserving compatibility aliases.
It adds no `PreparedPostGroupFills`, lifecycle status, public API, Cargo
feature, raw option view, or O46 wiring. `project_slice.rs` changes only its
inactive-module reason. Public slicing therefore remains
`ProjectSlicingIncomplete` at O72 until a later source-cited lifecycle slice.

### Final evidence — pending

Exact focused/dependency/workspace command counts, strict
lint/format/Tier-1/diff/static results, and unconditional independent
source/specification and standards approval remain an intentionally unfilled
placeholder. Implemented status does not imply those final results.

### Task 22O.75: full-grouping sparse anchoring

O75 ports pinned `Fill/Fill.cpp:1394-1407` by replacing O46's temporary reduced
anchoring grouping with O74 `group_fills`. The graph-native entry receives the
prepared external-surface graph and aligned object/layer indices, filters the
complete owned result for `Internal`, and feeds existing CrossHatch generation
from grouped spacing, angle, multiline, anchor lengths, geometry, and source
`float(0.01 * density)` conversion.

`sparse_anchoring/grouping.rs`, `SparseAnchoringLayer`, and the reduced direct
tests are deleted without fallback. The fixed-MSVC KSR 18-layer anchoring
oracle remains 186 paths / 5,941 points with aggregate digest
`917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef`.
Focused anchoring/grouping/transaction runs passed 1/1, 35/35, and 17/17;
workspace Nextest passed 6,516/6,516 with 27 slow and two configured skips.
Core strict Clippy, rustfmt, diff, static deletion, and sub-400-LOC gates passed.
O75 does not activate lifecycle output; complete G-code parity remains pending.

### Task 22O.76: CrossHatch fill entities

O76 ports pinned `Fill/Fill.cpp:1213-1224,1234-1357` and
`FillBase.cpp:133-184` for the first `Layer::make_fills` vertical slice. The
crate-private graph seam consumes complete groups, selects CrossHatch, and
creates ordered owned extrusion collections with grouped role and Internal Flow
metadata. Focused bits are `mm3_per_mm=0x3fb4d7aca0000000`,
`width=0x3ee66666`, and `height=0x3e4ccccd`.

Three graph-native tests pass for output metadata/order, repeatability and
immutability, non-CrossHatch non-fallback, and atomic range error. Strict
workspace Clippy, rustfmt, diff, and LOC checks pass. This remains a
CrossHatch-only inactive slice; the complete KSR fill-entity oracle and G-code
parity wait for the remaining source filler classes and lifecycle.

### Task 22O.77: rectilinear vertical segmentation

O77 ports pinned `FillRectilinear.cpp:357-496,759-993`. Its private source
prerequisite creates rotated outer/inner offset contour inventories and exact
rational vertical intersections with outer/inner low/high identity and source
ordering. Three focused tests cover rectangle/hole/offset topology, rational
rounding, rotation, repeatability/immutability, and range error. Strict core
Clippy, rustfmt, diff, and LOC gates pass. Link construction, monotonic traversal,
entities, lifecycle, and G-code remain pending.

### Task 22O.78: rectilinear contour links

O78 ports pinned `FillRectilinear.cpp:994-1214`. O77 intersections now carry
source adjacent/same-line contour links with horizontal/up/down direction and
valid/invalid/too-long quality. Two focused link tests plus three segmentation
regressions pass; strict core Clippy, rustfmt, diff, and LOC gates pass. Pinch,
monotonic region chaining, complete fillers/entities, lifecycle, and G-code
remain pending.

### Task 22O.79: rectilinear pinch intersections

O79 ports pinned `FillRectilinear.cpp:1216-1312`: disconnected inner runs gain
ordered phony outer pairs and all affected link indices are remapped. Two O79
focused tests plus five O77/O78 regressions pass; strict core Clippy, rustfmt,
diff, and LOC gates pass. Monotonic region generation/chaining, filler entities,
lifecycle, and G-code remain pending.

### Task 22O.80: monotonic region generation

O80 ports pinned `FillRectilinear.cpp:1590-1629,1711-1931`. Source-ordered seed
runs and exclusive adjacent overlaps produce owned monotonic region boundaries
and flip parity without mutating the linked input. Two focused tests pass;
strict core Clippy, rustfmt, diff, and LOC gates pass. Neighbors/path lengths,
chaining, filler entities, lifecycle, and G-code remain pending.

### Task 22O.81: monotonic region neighbors

O81 ports pinned `FillRectilinear.cpp:2079-2179`. O80 boundaries use O78
horizontal links to populate sorted unique symmetric left/right neighbor
indices. Two focused tests and all 1,179 task22o core regressions pass; strict
core Clippy, rustfmt, diff, and LOC gates pass. Region lengths, ant chaining,
filler entities, lifecycle, and G-code remain pending.

### Task 22O.82: rectilinear contour context

O82 ports the retained ownership boundary of pinned
`FillRectilinear.cpp:357-457,759-993`. Rotated source geometry, ordered
outer/inner contours, and indexed O77 lines now share one owned slice. Two
focused tests pass; strict core Clippy, rustfmt, diff, and LOC gates pass.
Perimeter measurement/emission, chaining, entities, lifecycle, and G-code remain
pending.

### Task 22O.83: rectilinear perimeter primitives

O83 ports pinned `FillRectilinear.cpp:38-116,459-685`. O82 indexed contours now
drive directed/wrapped arc lengths and exact forward/reverse adjacent/same-line
vertex emission. A RED same-segment oracle caught an incorrect full-loop append;
two focused and seven O77-O79 regression tests pass. Strict core Clippy,
rustfmt, diff, and LOC gates pass. Corrected links, region chaining, entities,
lifecycle, and G-code remain pending.

### Task 22O.84: source rectilinear links

O84 replaces O78 approximations with pinned `FillRectilinear.cpp:994-1214`.
Directed retained-contour distance selects adjacent/same-line links; skipped
inner intersections, same-side traps, exact arc-length gates, and invalid
symmetry follow source. Compile RED rejected the bare-lines seam. Two focused
and 15 O77-O83 regression tests pass; strict core Clippy, rustfmt, diff,
approximation-removal, and LOC gates pass. Region chaining, entities, lifecycle,
and G-code remain pending.

### Task 22O.85: monotonic region costs

O85 ports pinned `FillRectilinear.cpp:1989-2077,2179-2188`. Dual orientation
traversal keeps f32 order, half perimeter cost, split-gap distance, coordinate
unscaling, and common-minimum normalization. Compile RED proved the missing
seam; two focused and both O84 regression tests pass. Strict core Clippy,
rustfmt, diff, and LOC gates pass. Inter-region matrix/chaining, entities,
lifecycle, and G-code remain pending.

### Task 22O.86: monotonic path matrix

O86 ports pinned `FillRectilinear.cpp:1590-1709`. Dense orientation-addressed
entries lazily cache exact f32 endpoint length/visibility and retain independent
pheromone. Compile RED proved the missing module; two focused and both O85
regression tests pass. Strict core Clippy, rustfmt, diff, and LOC gates pass. Ant
simulation/RNG, path selection, entities, lifecycle, and G-code remain pending.

### Task 22O.87: monotonic ant chain

O87 ports pinned `FillRectilinear.cpp:2190-2582`. Default MT19937-64,
precedence-constrained greedy/ant traversal, source probability and pheromone
order, strict best replacement, and no-op 3-opt produce owned region/orientation
chains. Compile RED proved missing modules; three focused and both O86 regression
tests pass. Strict core Clippy, rustfmt, diff, and LOC gates pass. Polyline
emission, entities, lifecycle, and G-code remain pending.

### Task 22O.88: monotonic polyline emission

O88 ports pinned `FillRectilinear.cpp:2584-2753`. O87 chains emit exact outer
endpoints, vertical runs, retained contour arcs, split paths, scale-aware
filtering, and phony-pinch merging. Compile RED proved the missing emitter; two
focused and all three O87 regression tests pass. Strict core Clippy, rustfmt,
diff, and LOC gates pass. Filler orchestration/rotation, entities, lifecycle,
and G-code remain pending.

### Task 22O.89: monotonic surface filler

O89 ports pinned `FillBase.cpp:255-324` and
`FillRectilinear.cpp:2755-2908,3404-3421`. Explicit parameters drive direction,
layer alternation, offsets, density/solid spacing, retained scanlines, O79-O88,
and inverse rotation without re-offsetting geometry. Compile RED proved the
missing module; two focused and five O77/O88 regression tests pass. Strict core
Clippy, rustfmt, diff, and LOC gates pass. Grouped entities, lifecycle, and
G-code remain pending.

### Task 22O.90: monotonic fill entities

O90 ports the Monotonic/MonotonicLine part of pinned `Fill.cpp:1213-1374` and
`FillBase.cpp:133-155`. Grouped effective state drives O89, the dense source
link gate and zero-anchor MonotonicLine policy, then exact ordered role/flow
collections. Compile RED proved missing dispatch; two focused, three O76, and
two O89 regression tests pass. Strict core Clippy, rustfmt, diff, and LOC gates
pass. Remaining fillers/thin fills, lifecycle, and G-code remain pending.

### Task 22O.91: layer fill entity stage

O91 ports pinned `Fill.cpp:1213-1384` ownership across every object/layer after
combination and advances the public lifecycle transactionally. KSR traversal
also ports pinned O77 endpoint-overlap classification, O79 any-side vertical
connectivity, and the O80 zigzag reachability invariant. Three focused and
O79/O80/O90 regression tests pass; strict core Clippy, rustfmt, diff, and LOC
gates pass. Thin fills, perimeter/fill ordering, motion, and G-code remain
pending.

### Task 22O.92: thin fill append

O92 ports pinned `Fill.cpp:1376-1384`. Retained thin-fill paths/loops move after
fill collections with exact order, 3D points, role/flow metadata, and ownership.
Compile RED proved the missing field; KSR freezes 2,285 entities/paths and 5,401
points. All three O91 tests and strict core Clippy, rustfmt, diff, and LOC gates
pass. Island ordering, motion, and G-code remain pending.

### Task 22O.93: layer-region extrusion ownership

O93 ports pinned `Layer.hpp:43-76`. O92 layer outputs now own retained perimeter
collections beside generated fills and moved thin fills, preserving source
order and draining predecessor inventories. Compile RED proved the missing
boundary; KSR freezes 2,881 collections, 5,243 loops, 5,483 paths, and 111,933
points. Three lifecycle/repeatability tests and strict core gates pass. Island
sorting/chaining, motion, and G-code remain pending.

### Task 22O.94: extrusion island assignment

O94 ports pinned `GCode.cpp:4970-5048` for KSR's single region/tool. Source
bbox-area order, half-open bounds, contour containment, and fallback-island
assignment now own generated fills, appended thin fills, and perimeters. KSR
freezes 3,350 total / 2,881 nonempty / zero nonempty-fallback islands and exact
1,658/2,285/2,881 entity inventories. Three tests and strict core gates pass.
Multi-region/tool/wiping, chaining, motion, and G-code remain pending.
