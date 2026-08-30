# OrcaSlicer option coverage summary

283 of 348 executable option domains pass (1054 generated cases).

| status | option | type | cases | upstream | first result |
|---|---|---|---:|---|---|
| PASS | accel_to_decel_enable | coBool | 2 | src/libslic3r/PrintConfig.cpp:3272 |  |
| PASS | accel_to_decel_factor | coPercent | 3 | src/libslic3r/PrintConfig.cpp:3279 |  |
| PASS | activate_air_filtration | coBools | 2 | src/libslic3r/PrintConfig.cpp:1886 |  |
| PASS | activate_air_filtration_during_print | coBools | 2 | src/libslic3r/PrintConfig.cpp:1893 |  |
| PASS | activate_air_filtration_on_completion | coBools | 2 | src/libslic3r/PrintConfig.cpp:1899 |  |
| PASS | activate_chamber_temp_control | coBools | 2 | src/libslic3r/PrintConfig.cpp:6599 |  |
| UNBOUNDED | adaptive_bed_mesh_margin | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2283 | bounded min/max not explicit in definition block |
| PASS | adaptive_pressure_advance | coBools | 2 | src/libslic3r/PrintConfig.cpp:2353 |  |
| UNBOUNDED | adaptive_pressure_advance_bridges | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2401 | bounded min/max not explicit in definition block |
| UNBOUNDED | adaptive_pressure_advance_model | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2369 | non boolean/enum/range option |
| PASS | adaptive_pressure_advance_overhangs | coBools | 2 | src/libslic3r/PrintConfig.cpp:2394 |  |
| PASS | additional_cooling_fan_speed | coInts | 3 | src/libslic3r/PrintConfig.cpp:4780 |  |
| PASS | additional_fan_full_speed_layer | coInts | 3 | src/libslic3r/PrintConfig.cpp:4799 |  |
| PASS | align_infill_direction_to_model | coBool | 2 | src/libslic3r/PrintConfig.cpp:2979 |  |
| FAIL | alternate_extra_wall | coBool | 2 | src/libslic3r/PrintConfig.cpp:5059 | true: unsupported project feature: alternate_extra_wall |
| PASS | auxiliary_fan | coBool | 2 | src/libslic3r/PrintConfig.cpp:3824 |  |
| PASS | bbl_calib_mark_logo | coBool | 2 | src/libslic3r/PrintConfig.cpp:5478 |  |
| PASS | bbl_use_printhost | coBool | 2 | src/libslic3r/PrintConfig.cpp:814 |  |
| UNBOUNDED | bed_custom_model | coString | 0 | src/libslic3r/PrintConfig.cpp:733 | non boolean/enum/range option |
| UNBOUNDED | bed_custom_texture | coString | 0 | src/libslic3r/PrintConfig.cpp:727 | non boolean/enum/range option |
| UNBOUNDED | bed_exclude_area | coPoints | 0 | src/libslic3r/PrintConfig.cpp:719 | non boolean/enum/range option |
| UNBOUNDED | bed_mesh_max | coPoint | 0 | src/libslic3r/PrintConfig.cpp:2262 | non boolean/enum/range option |
| UNBOUNDED | bed_mesh_min | coPoint | 0 | src/libslic3r/PrintConfig.cpp:2250 | non boolean/enum/range option |
| UNBOUNDED | bed_mesh_probe_distance | coPoint | 0 | src/libslic3r/PrintConfig.cpp:2274 | non boolean/enum/range option |
| PASS | bed_temperature_formula | coEnum | 2 | src/libslic3r/PrintConfig.cpp:2591 |  |
| UNBOUNDED | before_layer_change_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:1147 | non boolean/enum/range option |
| UNBOUNDED | best_object_pos | coPoint | 0 | src/libslic3r/PrintConfig.cpp:3818 | non boolean/enum/range option |
| UNBOUNDED | bottom_shell_layers | coInt | 0 | src/libslic3r/PrintConfig.cpp:1156 | bounded min/max not explicit in definition block |
| UNBOUNDED | bottom_shell_thickness | coFloat | 0 | src/libslic3r/PrintConfig.cpp:1167 | bounded min/max not explicit in definition block |
| FAIL | bottom_solid_infill_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1377 | min: layer 1 island lifecycle differs: expected [[Extruder { extrusion: "-4", feed: "3600" }], [Extruder { extrusion: "4", feed: "2400" }], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "103.362", y: "103.627", z: "0.2" }, end: Position { x: "104.152", y: "103.014", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "114.69", y: "114.65", z: "0.2" }, end: Position { x: "113.69", y: "114.654", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "1800" }, WipeEnd]], actual [[Extruder { extrusion: "-4", feed: "3600" }], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "103.362", y: "103.627", z: "0.2" }, end: Position { x: "104.152", y: "103.014", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "114.69", y: "114.65", z: "0.2" }, end: Position { x: "113.69", y: "114.654", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "1800" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }, Extruder { extrusion: "0", feed: "2100" }]] |
| FAIL | bottom_surface_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:6764 | seeded: layer 1 deposition 1 differs: expected Deposition { feature: "Bottom surface", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "106.013", y: "106.013", z: "0.2" }, end: Position { x: "106.013", y: "106.772", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.02332", feed: 2100.0, acceleration: "500", fans: "0:0" }, actual Deposition { feature: "Bottom surface", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "106.013", y: "106.013", z: "0.2" }, end: Position { x: "113.987", y: "113.987", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.34653", feed: 2100.0, acceleration: "500", fans: "0:0" } |
| UNBOUNDED | bottom_surface_filament_id | coInt | 0 | src/libslic3r/PrintConfig.cpp:5799 | bounded min/max not explicit in definition block |
| FAIL | bottom_surface_pattern | coEnum | 28 | src/libslic3r/PrintConfig.cpp:2097 | 3dhoneycomb: orca-slicer failed (exit status: 238): bottom_surface_pattern: invalid value 3dhoneycomb |
| UNBOUNDED | bridge_acceleration | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3224 | bounded min/max not explicit in definition block |
| PASS | bridge_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1250 |  |
| PASS | bridge_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:1292 |  |
| FAIL | bridge_flow | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1327 | min: orca-slicer failed (exit status: 238): bridge_flow: invalid value 0.000000 |
| FAIL | bridge_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:1339 | seeded: orca-slicer failed (exit status: 205): Too small line width |
| PASS | bridge_no_support | coBool | 2 | src/libslic3r/PrintConfig.cpp:1933 |  |
| UNBOUNDED | bridge_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:1658 | bounded min/max not explicit in definition block |
| UNBOUNDED | brim_ears_detection_length | coFloat | 0 | src/libslic3r/PrintConfig.cpp:1770 | bounded min/max not explicit in definition block |
| PASS | brim_ears_max_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1758 |  |
| PASS | brim_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1723 |  |
| PASS | brim_object_gap | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1713 |  |
| FAIL | brim_type | coEnum | 7 | src/libslic3r/PrintConfig.cpp:1690 | painted: unsupported project feature: brim_type |
| PASS | brim_use_efc_outline | coBool | 2 | src/libslic3r/PrintConfig.cpp:1734 |  |
| PASS | brim_width | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1680 |  |
| PASS | calib_flowrate_topinfill_special_order | coBool | 2 | src/libslic3r/PrintConfig.cpp:4277 |  |
| UNBOUNDED | chamber_minimal_temperature | coInts | 0 | src/libslic3r/PrintConfig.cpp:6629 | bounded min/max not explicit in definition block |
| UNBOUNDED | chamber_temperature | coInts | 0 | src/libslic3r/PrintConfig.cpp:6608 | bounded min/max not explicit in definition block |
| UNBOUNDED | change_extrusion_role_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:6691 | non boolean/enum/range option |
| UNBOUNDED | change_filament_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:6682 | non boolean/enum/range option |
| PASS | close_additional_fan_first_x_layers | coInts | 3 | src/libslic3r/PrintConfig.cpp:4790 |  |
| PASS | close_fan_the_first_x_layers | coInts | 3 | src/libslic3r/PrintConfig.cpp:1923 |  |
| PASS | combine_brims | coBool | 2 | src/libslic3r/PrintConfig.cpp:1744 |  |
| PASS | complete_print_exhaust_fan_speed | coInts | 3 | src/libslic3r/PrintConfig.cpp:1914 |  |
| PASS | cool_plate_temp | coInts | 3 | src/libslic3r/PrintConfig.cpp:971 |  |
| PASS | cool_plate_temp_initial_layer | coInts | 3 | src/libslic3r/PrintConfig.cpp:1031 |  |
| UNBOUNDED | cooling_tube_length | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4911 | bounded min/max not explicit in definition block |
| UNBOUNDED | cooling_tube_retraction | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4903 | bounded min/max not explicit in definition block |
| PASS | counterbore_hole_bridging | coEnum | 3 | src/libslic3r/PrintConfig.cpp:1547 |  |
| PASS | curr_bed_type | coEnum | 6 | src/libslic3r/PrintConfig.cpp:1080 |  |
| UNBOUNDED | default_acceleration | coFloat | 0 | src/libslic3r/PrintConfig.cpp:1865 | bounded min/max not explicit in definition block |
| UNBOUNDED | default_bed_type | coString | 0 | src/libslic3r/PrintConfig.cpp:1102 | non boolean/enum/range option |
| UNBOUNDED | default_filament_colour | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2447 | non boolean/enum/range option |
| UNBOUNDED | default_filament_profile | coStrings | 0 | src/libslic3r/PrintConfig.cpp:1874 | non boolean/enum/range option |
| UNBOUNDED | default_jerk | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3289 | bounded min/max not explicit in definition block |
| PASS | default_junction_deviation | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3298 |  |
| PASS | default_nozzle_volume_type | coEnums | 2 | src/libslic3r/PrintConfig.cpp:5360 |  |
| UNBOUNDED | default_print_profile | coString | 0 | src/libslic3r/PrintConfig.cpp:1880 | non boolean/enum/range option |
| UNBOUNDED | deretraction_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5463 | bounded min/max not explicit in definition block |
| PASS | detect_narrow_internal_solid_infill | coBool | 2 | src/libslic3r/PrintConfig.cpp:7320 |  |
| PASS | detect_overhang_wall | coBool | 2 | src/libslic3r/PrintConfig.cpp:5003 |  |
| FAIL | detect_thin_wall | coBool | 2 | src/libslic3r/PrintConfig.cpp:6674 | true: unsupported project feature: detect_thin_wall |
| PASS | disable_m73 | coBool | 2 | src/libslic3r/PrintConfig.cpp:5484 |  |
| FAIL | dont_filter_internal_bridges | coEnum | 3 | src/libslic3r/PrintConfig.cpp:1990 | nofilter: unsupported project feature: bridge_over_infill_anchor_surface_kind |
| PASS | dont_slow_down_outer_wall | coBools | 2 | src/libslic3r/PrintConfig.cpp:2428 |  |
| PASS | draft_shield | coEnum | 2 | src/libslic3r/PrintConfig.cpp:5706 |  |
| PASS | during_print_exhaust_fan_speed | coInts | 3 | src/libslic3r/PrintConfig.cpp:1905 |  |
| UNBOUNDED | elefant_foot_compensation | coFloat | 0 | src/libslic3r/PrintConfig.cpp:739 | bounded min/max not explicit in definition block |
| UNBOUNDED | elefant_foot_compensation_layers | coInt | 0 | src/libslic3r/PrintConfig.cpp:748 | bounded min/max not explicit in definition block |
| FAIL | elefant_foot_layers_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:759 | min: filament 1 length differs: expected 259.75mm, actual 262.34mm |
| PASS | emit_machine_limits_to_gcode | coBool | 2 | src/libslic3r/PrintConfig.cpp:4446 |  |
| FAIL | enable_arc_fitting | coBool | 2 | src/libslic3r/PrintConfig.cpp:3727 | true: layer 27 deposition 26 differs: expected Deposition { feature: "Sparse infill", width: "0.45", motion: MotionRecord { command: "G1", start: Position { x: "113.955", y: "111.036", z: "5.4" }, end: Position { x: "113.109", y: "111.882", z: "5.4" }, arc_center: [None, None], turns: None }, extrusion: "0.03969", feed: 908.0, acceleration: "500", fans: "0:255" }, actual Deposition { feature: "Sparse infill", width: "0.45", motion: MotionRecord { command: "G1", start: Position { x: "113.955", y: "106.891", z: "5.4" }, end: Position { x: "114.02", y: "105.98", z: "5.4" }, arc_center: [None, None], turns: None }, extrusion: "0.03029", feed: 908.0, acceleration: "500", fans: "0:255" } |
| FAIL | enable_extra_bridge_layer | coEnum | 4 | src/libslic3r/PrintConfig.cpp:1959 | apply_to_all: unsupported project feature: enable_extra_bridge_layer |
| PASS | enable_filament_ramming | coBool | 2 | src/libslic3r/PrintConfig.cpp:5989 |  |
| UNBOUNDED | enable_long_retraction_when_cut | coInt | 0 | src/libslic3r/PrintConfig.cpp:5210 | bounded min/max not explicit in definition block |
| PASS | enable_overhang_bridge_fan | coBools | 2 | src/libslic3r/PrintConfig.cpp:1207 |  |
| PASS | enable_overhang_speed | coBool | 2 | src/libslic3r/PrintConfig.cpp:1580 |  |
| PASS | enable_power_loss_recovery | coEnum | 3 | src/libslic3r/PrintConfig.cpp:3752 |  |
| PASS | enable_pressure_advance | coBools | 2 | src/libslic3r/PrintConfig.cpp:2340 |  |
| PASS | enable_prime_tower | coBool | 2 | src/libslic3r/PrintConfig.cpp:6812 |  |
| PASS | enable_support | coBool | 2 | src/libslic3r/PrintConfig.cpp:6054 |  |
| PASS | enable_tower_interface_cooldown_during_tower | coBool | 2 | src/libslic3r/PrintConfig.cpp:6999 |  |
| PASS | enable_tower_interface_features | coBool | 2 | src/libslic3r/PrintConfig.cpp:6993 |  |
| PASS | enable_wrapping_detection | coBool | 2 | src/libslic3r/PrintConfig.cpp:4107 |  |
| PASS | enforce_support_layers | coInt | 3 | src/libslic3r/PrintConfig.cpp:6164 |  |
| PASS | eng_plate_temp | coInts | 3 | src/libslic3r/PrintConfig.cpp:991 |  |
| PASS | eng_plate_temp_initial_layer | coInts | 3 | src/libslic3r/PrintConfig.cpp:1051 |  |
| PASS | ensure_vertical_shell_thickness | coEnum | 4 | src/libslic3r/PrintConfig.cpp:2055 |  |
| PASS | exclude_object | coBool | 2 | src/libslic3r/PrintConfig.cpp:3959 |  |
| UNBOUNDED | extra_loading_move | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4936 | bounded min/max not explicit in definition block |
| PASS | extra_perimeters_on_overhangs | coBool | 2 | src/libslic3r/PrintConfig.cpp:1519 |  |
| UNBOUNDED | extra_solid_infills | coString | 0 | src/libslic3r/PrintConfig.cpp:2987 | non boolean/enum/range option |
| UNBOUNDED | extruder_ams_count | coStrings | 0 | src/libslic3r/PrintConfig.cpp:5379 | non boolean/enum/range option |
| UNBOUNDED | extruder_clearance_height_to_lid | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2225 | bounded min/max not explicit in definition block |
| UNBOUNDED | extruder_clearance_height_to_rod | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2215 | bounded min/max not explicit in definition block |
| UNBOUNDED | extruder_clearance_radius | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2234 | bounded min/max not explicit in definition block |
| UNBOUNDED | extruder_colour | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2297 | non boolean/enum/range option |
| UNBOUNDED | extruder_offset | coPoints | 0 | src/libslic3r/PrintConfig.cpp:2305 | non boolean/enum/range option |
| UNBOUNDED | extruder_printable_area | coPointsGroups | 0 | src/libslic3r/PrintConfig.cpp:692 | non boolean/enum/range option |
| PASS | extruder_printable_height | coFloats | 3 | src/libslic3r/PrintConfig.cpp:788 |  |
| PASS | extruder_type | coEnums | 2 | src/libslic3r/PrintConfig.cpp:5335 |  |
| UNBOUNDED | extruder_variant_list | coStrings | 0 | src/libslic3r/PrintConfig.cpp:5372 | non boolean/enum/range option |
| PASS | extrusion_rate_smoothing_external_perimeter_only | coBool | 2 | src/libslic3r/PrintConfig.cpp:4763 |  |
| PASS | fan_cooling_layer_time | coFloats | 3 | src/libslic3r/PrintConfig.cpp:2437 |  |
| UNBOUNDED | fan_kickstart | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3849 | bounded min/max not explicit in definition block |
| FAIL | fan_max_speed | coFloats | 3 | src/libslic3r/PrintConfig.cpp:4711 | seeded: layer 2 control events differs: expected [";BEFORE_LAYER_CHANGE", ";0.4", "G92 E0", "", ";_SET_FAN_SPEED_CHANGING_LAYER", "; printing object cube10.stl id:0 copy 0", "; stop printing object cube10.stl id:0 copy 0", "M106 S114"], actual [";BEFORE_LAYER_CHANGE", ";0.4", "G92 E0", "", ";_SET_FAN_SPEED_CHANGING_LAYER", "; printing object cube10.stl id:0 copy 0", "; stop printing object cube10.stl id:0 copy 0"] |
| PASS | fan_min_speed | coFloats | 3 | src/libslic3r/PrintConfig.cpp:4771 |  |
| PASS | fan_speedup_overhangs | coBool | 2 | src/libslic3r/PrintConfig.cpp:3843 |  |
| UNBOUNDED | fan_speedup_time | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3830 | bounded min/max not explicit in definition block |
| PASS | filament_adaptive_volumetric_speed | coBools | 2 | src/libslic3r/PrintConfig.cpp:2645 |  |
| UNBOUNDED | filament_adhesiveness_category | coInts | 0 | src/libslic3r/PrintConfig.cpp:2684 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_change_extrusion_role_gcode | coStrings | 0 | src/libslic3r/PrintConfig.cpp:6700 | non boolean/enum/range option |
| UNBOUNDED | filament_change_length | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2892 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_colour | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2455 | non boolean/enum/range option |
| UNBOUNDED | filament_colour_type | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2477 | non boolean/enum/range option |
| UNBOUNDED | filament_cooling_before_tower | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2777 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_cooling_final_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2825 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_cooling_initial_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2758 | bounded min/max not explicit in definition block |
| PASS | filament_cooling_moves | coInts | 3 | src/libslic3r/PrintConfig.cpp:2734 |  |
| UNBOUNDED | filament_cost | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2925 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_density | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2864 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_deretraction_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:72 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_diameter | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2606 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_end_gcode | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2046 | non boolean/enum/range option |
| UNBOUNDED | filament_extruder_variant | coStrings | 0 | src/libslic3r/PrintConfig.cpp:5425 | non boolean/enum/range option |
| FAIL | filament_flow_ratio | coFloats | 3 | src/libslic3r/PrintConfig.cpp:2315 | min: orca-slicer failed (exit status: 238): filament_flow_ratio: invalid value 0 |
| UNBOUNDED | filament_flush_temp | coInts | 0 | src/libslic3r/PrintConfig.cpp:2530 | bounded min/max not explicit in definition block |
| PASS | filament_flush_volumetric_speed | coFloats | 3 | src/libslic3r/PrintConfig.cpp:2540 |  |
| UNBOUNDED | filament_ids | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2938 | non boolean/enum/range option |
| PASS | filament_ironing_flow | coPercents | 3 | src/libslic3r/PrintConfig.cpp:3493 |  |
| PASS | filament_ironing_inset | coFloats | 3 | src/libslic3r/PrintConfig.cpp:3517 |  |
| PASS | filament_ironing_spacing | coFloats | 3 | src/libslic3r/PrintConfig.cpp:3505 |  |
| UNBOUNDED | filament_ironing_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:3529 | bounded min/max not explicit in definition block |
| PASS | filament_is_support | coBools | 2 | src/libslic3r/PrintConfig.cpp:2900 |  |
| UNBOUNDED | filament_loading_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2691 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_loading_speed_start | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2699 | bounded min/max not explicit in definition block |
| PASS | filament_long_retractions_when_cut | coBools | 2 | src/libslic3r/PrintConfig.cpp:82 |  |
| UNBOUNDED | filament_map | coInts | 0 | src/libslic3r/PrintConfig.cpp:2489 | bounded min/max not explicit in definition block |
| PASS | filament_map_mode | coEnum | 4 | src/libslic3r/PrintConfig.cpp:2502 |  |
| UNBOUNDED | filament_max_volumetric_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2550 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_minimal_purge_on_wipe_tower | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2766 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_multi_colour | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2473 | non boolean/enum/range option |
| PASS | filament_multitool_ramming | coBools | 2 | src/libslic3r/PrintConfig.cpp:2840 |  |
| UNBOUNDED | filament_multitool_ramming_flow | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2856 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_multitool_ramming_volume | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2848 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_notes | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2463 | non boolean/enum/range option |
| UNBOUNDED | filament_printable | coInts | 0 | src/libslic3r/PrintConfig.cpp:2910 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_ramming_parameters | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2833 | non boolean/enum/range option |
| UNBOUNDED | filament_retract_before_wipe | coPercents | 0 | src/libslic3r/PrintConfig.cpp:81 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_retract_lift_above | coFloats | 0 | src/libslic3r/PrintConfig.cpp:68 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_retract_lift_below | coFloats | 0 | src/libslic3r/PrintConfig.cpp:69 | bounded min/max not explicit in definition block |
| FAIL | filament_retract_lift_enforce | coEnums | 4 | src/libslic3r/PrintConfig.cpp:70 | Bottom Only: layer 2 travel geometry count differs: expected 8, actual 14 |
| UNBOUNDED | filament_retract_restart_extra | coFloats | 0 | src/libslic3r/PrintConfig.cpp:73 | bounded min/max not explicit in definition block |
| FAIL | filament_retract_when_changing_layer | coBools | 2 | src/libslic3r/PrintConfig.cpp:78 | false: layer 50 island lifecycle differs: expected [[Extruder { extrusion: "-3.81129", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.558", y: "106.502", z: "10" }, end: Position { x: "113.488", y: "106.467", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09446", feed: "3000" }, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.488", y: "106.467", z: "10" }, end: Position { x: "113.556", y: "106.428", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09425", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "106.527", y: "105.795", z: "10" }, end: Position { x: "105.82", y: "106.502", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "1800" }, WipeEnd]], actual [[Extruder { extrusion: "-3.81129", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.558", y: "106.502", z: "10" }, end: Position { x: "113.488", y: "106.467", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09446", feed: "3000" }, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.488", y: "106.467", z: "10" }, end: Position { x: "113.556", y: "106.428", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09425", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }]] |
| UNBOUNDED | filament_retraction_distances_when_cut | coFloats | 0 | src/libslic3r/PrintConfig.cpp:83 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_retraction_length | coFloats | 0 | src/libslic3r/PrintConfig.cpp:65 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_retraction_minimum_travel | coFloats | 0 | src/libslic3r/PrintConfig.cpp:74 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_retraction_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:71 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_self_index | coInts | 0 | src/libslic3r/PrintConfig.cpp:5432 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_settings_id | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2933 | non boolean/enum/range option |
| FAIL | filament_shrink | coPercents | 3 | src/libslic3r/PrintConfig.cpp:2659 | min: unsupported project feature: filament_shrink |
| FAIL | filament_shrinkage_compensation_z | coPercents | 3 | src/libslic3r/PrintConfig.cpp:2672 | min: unsupported project feature: filament_shrinkage_compensation_z |
| PASS | filament_soluble | coBools | 2 | src/libslic3r/PrintConfig.cpp:2886 |  |
| UNBOUNDED | filament_stamping_distance | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2750 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_stamping_loading_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2743 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_start_gcode | coStrings | 0 | src/libslic3r/PrintConfig.cpp:5949 | non boolean/enum/range option |
| UNBOUNDED | filament_toolchange_delay | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2724 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_tower_interface_pre_extrusion_dist | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2785 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_tower_interface_pre_extrusion_length | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2793 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_tower_interface_print_temp | coInts | 0 | src/libslic3r/PrintConfig.cpp:2817 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_tower_interface_purge_volume | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2809 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_tower_ironing_area | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2801 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_type | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2872 | non boolean/enum/range option |
| UNBOUNDED | filament_unloading_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2707 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_unloading_speed_start | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2716 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_vendor | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2942 | non boolean/enum/range option |
| PASS | filament_wipe | coBools | 2 | src/libslic3r/PrintConfig.cpp:79 |  |
| UNBOUNDED | filament_wipe_distance | coFloats | 0 | src/libslic3r/PrintConfig.cpp:76 | bounded min/max not explicit in definition block |
| UNBOUNDED | filament_z_hop | coFloats | 0 | src/libslic3r/PrintConfig.cpp:66 | bounded min/max not explicit in definition block |
| FAIL | filament_z_hop_types | coEnums | 4 | src/libslic3r/PrintConfig.cpp:67 | Auto Lift: layer 2 travel feed differs: no match for Travel { motion: MotionRecord { command: "G1", start: Position { x: "106.594", y: "117.566", z: "0.8" }, end: Position { x: "106.594", y: "117.566", z: "0.4" }, arc_center: [None, None], turns: None }, feed: 9000.0, acceleration: "500" } |
| UNBOUNDED | file_start_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:5928 | non boolean/enum/range option |
| UNBOUNDED | filename_format | coString | 0 | src/libslic3r/PrintConfig.cpp:4967 | non boolean/enum/range option |
| FAIL | fill_multiline | coInt | 3 | src/libslic3r/PrintConfig.cpp:2996 | max: unsupported project feature: fill_multiline |
| UNBOUNDED | filter_out_gap_fill | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3698 | bounded min/max not explicit in definition block |
| PASS | first_layer_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1394 |  |
| PASS | first_layer_print_sequence | coInts | 3 | src/libslic3r/PrintConfig.cpp:1109 |  |
| PASS | first_x_layer_fan_speed | coFloats | 3 | src/libslic3r/PrintConfig.cpp:4808 |  |
| UNBOUNDED | flashforge_serial_number | coString | 0 | src/libslic3r/PrintConfig.cpp:861 | non boolean/enum/range option |
| PASS | flush_into_infill | coBool | 2 | src/libslic3r/PrintConfig.cpp:7013 |  |
| PASS | flush_into_objects | coBool | 2 | src/libslic3r/PrintConfig.cpp:7030 |  |
| PASS | flush_into_support | coBool | 2 | src/libslic3r/PrintConfig.cpp:7022 |  |
| UNBOUNDED | flush_multiplier | coFloats | 0 | src/libslic3r/PrintConfig.cpp:6845 | bounded min/max not explicit in definition block |
| UNBOUNDED | flush_volumes_matrix | coFloats | 0 | src/libslic3r/PrintConfig.cpp:6835 | bounded min/max not explicit in definition block |
| UNBOUNDED | flush_volumes_vector | coFloats | 0 | src/libslic3r/PrintConfig.cpp:6825 | bounded min/max not explicit in definition block |
| PASS | full_fan_speed_layer | coInts | 3 | src/libslic3r/PrintConfig.cpp:3445 |  |
| FAIL | fuzzy_skin | coEnum | 6 | src/libslic3r/PrintConfig.cpp:3540 | all: unsupported project feature: fuzzy_skin |
| PASS | fuzzy_skin_first_layer | coBool | 2 | src/libslic3r/PrintConfig.cpp:3581 |  |
| UNBOUNDED | fuzzy_skin_layers_between_ripple_offset | coInt | 0 | src/libslic3r/PrintConfig.cpp:3687 | bounded min/max not explicit in definition block |
| PASS | fuzzy_skin_mode | coEnum | 3 | src/libslic3r/PrintConfig.cpp:3588 |  |
| PASS | fuzzy_skin_noise_type | coEnum | 6 | src/libslic3r/PrintConfig.cpp:3611 |  |
| PASS | fuzzy_skin_octaves | coInt | 3 | src/libslic3r/PrintConfig.cpp:3647 |  |
| PASS | fuzzy_skin_persistence | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3656 |  |
| PASS | fuzzy_skin_point_distance | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3571 |  |
| PASS | fuzzy_skin_ripple_offset | coPercent | 3 | src/libslic3r/PrintConfig.cpp:3673 |  |
| UNBOUNDED | fuzzy_skin_ripples_per_layer | coInt | 0 | src/libslic3r/PrintConfig.cpp:3665 | bounded min/max not explicit in definition block |
| PASS | fuzzy_skin_scale | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3637 |  |
| PASS | fuzzy_skin_thickness | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3561 |  |
| PASS | gap_fill_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1454 |  |
| FAIL | gap_fill_target | coEnum | 3 | src/libslic3r/PrintConfig.cpp:1178 | everywhere: layer 2 deposition 1 differs: expected Deposition { feature: "Gap infill", width: "0.449969", motion: MotionRecord { command: "G1", start: Position { x: "106.539", y: "105.879", z: "0.4" }, end: Position { x: "105.879", y: "106.539", z: "0.4" }, arc_center: [None, None], turns: None }, extrusion: "0.03096", feed: 1800.0, acceleration: "500", fans: "0:255" }, actual Deposition { feature: "Gap infill", width: "0.449967", motion: MotionRecord { command: "G1", start: Position { x: "106.539", y: "105.879", z: "0.4" }, end: Position { x: "105.879", y: "106.539", z: "0.4" }, arc_center: [None, None], turns: None }, extrusion: "0.03096", feed: 1800.0, acceleration: "500", fans: "0:255" } |
| UNBOUNDED | gap_infill_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3707 | bounded min/max not explicit in definition block |
| PASS | gcode_add_line_number | coBool | 2 | src/libslic3r/PrintConfig.cpp:3738 |  |
| PASS | gcode_comments | coBool | 2 | src/libslic3r/PrintConfig.cpp:3965 |  |
| FAIL | gcode_flavor | coEnum | 13 | src/libslic3r/PrintConfig.cpp:3905 | mach3: orca-slicer failed (exit status: 238): gcode_flavor: invalid value mach3 |
| PASS | gcode_label_objects | coBool | 2 | src/libslic3r/PrintConfig.cpp:3951 |  |
| UNBOUNDED | grab_length | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2290 | bounded min/max not explicit in definition block |
| PASS | gyroid_optimized | coBool | 2 | src/libslic3r/PrintConfig.cpp:3008 |  |
| PASS | has_scarf_joint_seam | coBool | 2 | src/libslic3r/PrintConfig.cpp:4493 |  |
| UNBOUNDED | head_wrap_detect_zone | coPoints | 0 | src/libslic3r/PrintConfig.cpp:6669 | non boolean/enum/range option |
| PASS | high_current_on_filament_swap | coBool | 2 | src/libslic3r/PrintConfig.cpp:4919 |  |
| PASS | hole_to_polyhole | coBool | 2 | src/libslic3r/PrintConfig.cpp:7093 |  |
| UNBOUNDED | hole_to_polyhole_threshold | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:7102 | bounded min/max not explicit in definition block |
| PASS | hole_to_polyhole_twisted | coBool | 2 | src/libslic3r/PrintConfig.cpp:7115 |  |
| PASS | host_type | coEnum | 16 | src/libslic3r/PrintConfig.cpp:4853 |  |
| PASS | hot_plate_temp | coInts | 3 | src/libslic3r/PrintConfig.cpp:1001 |  |
| UNBOUNDED | hot_plate_temp_initial_layer | coInts | 0 | src/libslic3r/PrintConfig.cpp:1061 | bounded min/max not explicit in definition block |
| UNBOUNDED | idle_temperature | coInts | 0 | src/libslic3r/PrintConfig.cpp:7064 | bounded min/max not explicit in definition block |
| PASS | independent_support_layer_height | coBool | 2 | src/libslic3r/PrintConfig.cpp:6383 |  |
| UNBOUNDED | infill_anchor | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3137 | bounded min/max not explicit in definition block |
| UNBOUNDED | infill_anchor_max | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3165 | bounded min/max not explicit in definition block |
| PASS | infill_combination | coBool | 2 | src/libslic3r/PrintConfig.cpp:3974 |  |
| UNBOUNDED | infill_combination_max_layer_height | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:4093 | bounded min/max not explicit in definition block |
| PASS | infill_direction | coFloat | 3 | src/libslic3r/PrintConfig.cpp:2949 |  |
| UNBOUNDED | infill_jerk | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3335 | bounded min/max not explicit in definition block |
| PASS | infill_lock_depth | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4054 |  |
| PASS | infill_overhang_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3096 |  |
| PASS | infill_shift_step | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3982 |  |
| UNBOUNDED | infill_wall_overlap | coPercent | 0 | src/libslic3r/PrintConfig.cpp:4148 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_acceleration | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3254 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_infill_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3408 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_jerk | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3344 | bounded min/max not explicit in definition block |
| FAIL | initial_layer_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:3371 | min: invalid external perimeter flow spacing |
| UNBOUNDED | initial_layer_min_bead_width | coPercent | 0 | src/libslic3r/PrintConfig.cpp:7265 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_print_height | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3384 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3400 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_travel_acceleration | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3263 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_travel_jerk | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3362 | bounded min/max not explicit in definition block |
| UNBOUNDED | initial_layer_travel_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3416 | bounded min/max not explicit in definition block |
| UNBOUNDED | inner_wall_acceleration | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3188 | bounded min/max not explicit in definition block |
| UNBOUNDED | inner_wall_filament_id | coInt | 0 | src/libslic3r/PrintConfig.cpp:5020 | bounded min/max not explicit in definition block |
| PASS | inner_wall_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1414 |  |
| UNBOUNDED | inner_wall_jerk | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3317 | bounded min/max not explicit in definition block |
| FAIL | inner_wall_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:5029 | max: orca-slicer failed (exit status: 238): inner_wall_line_width: too large line width 4.000000 |
| UNBOUNDED | inner_wall_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5041 | bounded min/max not explicit in definition block |
| PASS | input_shaping_damp_x | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4695 |  |
| PASS | input_shaping_damp_y | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4703 |  |
| PASS | input_shaping_emit | coBool | 2 | src/libslic3r/PrintConfig.cpp:4662 |  |
| PASS | input_shaping_freq_x | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4677 |  |
| PASS | input_shaping_freq_y | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4686 |  |
| PASS | input_shaping_type | coEnum | 13 | src/libslic3r/PrintConfig.cpp:4668 |  |
| FAIL | interface_shells | coBool | 2 | src/libslic3r/PrintConfig.cpp:4197 | true: unsupported project feature: interface_shells |
| PASS | interlocking_beam | coBool | 2 | src/libslic3r/PrintConfig.cpp:4226 |  |
| UNBOUNDED | interlocking_beam_layer_count | coInt | 0 | src/libslic3r/PrintConfig.cpp:4252 | bounded min/max not explicit in definition block |
| UNBOUNDED | interlocking_beam_width | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4233 | bounded min/max not explicit in definition block |
| UNBOUNDED | interlocking_boundary_avoidance | coInt | 0 | src/libslic3r/PrintConfig.cpp:4268 | bounded min/max not explicit in definition block |
| UNBOUNDED | interlocking_depth | coInt | 0 | src/libslic3r/PrintConfig.cpp:4260 | bounded min/max not explicit in definition block |
| PASS | interlocking_orientation | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4242 |  |
| PASS | internal_bridge_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1268 |  |
| FAIL | internal_bridge_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:1309 | min: filament 1 length differs: expected 255.90mm, actual 262.34mm |
| PASS | internal_bridge_fan_speed | coInts | 3 | src/libslic3r/PrintConfig.cpp:3470 |  |
| FAIL | internal_bridge_flow | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1354 | min: orca-slicer failed (exit status: 238): internal_bridge_flow: invalid value 0.000000 |
| UNBOUNDED | internal_bridge_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:1670 | bounded min/max not explicit in definition block |
| UNBOUNDED | internal_solid_filament_id | coInt | 0 | src/libslic3r/PrintConfig.cpp:5781 | bounded min/max not explicit in definition block |
| UNBOUNDED | internal_solid_infill_acceleration | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3244 | bounded min/max not explicit in definition block |
| PASS | internal_solid_infill_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1444 |  |
| FAIL | internal_solid_infill_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:5808 | max: orca-slicer failed (exit status: 238): internal_solid_infill_line_width: too large line width 4.000000 |
| FAIL | internal_solid_infill_pattern | coEnum | 28 | src/libslic3r/PrintConfig.cpp:2106 | 3dhoneycomb: orca-slicer failed (exit status: 238): internal_solid_infill_pattern: invalid value 3dhoneycomb |
| UNBOUNDED | internal_solid_infill_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5820 | bounded min/max not explicit in definition block |
| PASS | ironing_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4351 |  |
| PASS | ironing_angle_fixed | coBool | 2 | src/libslic3r/PrintConfig.cpp:4361 |  |
| PASS | ironing_expansion | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4368 |  |
| PASS | ironing_fan_speed | coInts | 3 | src/libslic3r/PrintConfig.cpp:3481 |  |
| PASS | ironing_flow | coPercent | 3 | src/libslic3r/PrintConfig.cpp:4310 |  |
| PASS | ironing_inset | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4332 |  |
| PASS | ironing_pattern | coEnum | 2 | src/libslic3r/PrintConfig.cpp:4298 |  |
| PASS | ironing_spacing | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4322 |  |
| UNBOUNDED | ironing_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4342 | bounded min/max not explicit in definition block |
| FAIL | ironing_type | coEnum | 4 | src/libslic3r/PrintConfig.cpp:4281 | solid: filament 1 length differs: expected 274.88mm, actual 262.34mm |
| PASS | is_infill_first | coBool | 2 | src/libslic3r/PrintConfig.cpp:2181 |  |
| PASS | lateral_lattice_angle_1 | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3076 |  |
| PASS | lateral_lattice_angle_2 | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3086 |  |
| UNBOUNDED | layer_change_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:4415 | non boolean/enum/range option |
| UNBOUNDED | layer_height | coFloat | 0 | src/libslic3r/PrintConfig.cpp:771 | bounded min/max not explicit in definition block |
| PASS | lightning_overhang_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3106 |  |
| PASS | lightning_prune_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3116 |  |
| PASS | lightning_straightening_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:3127 |  |
| FAIL | line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:2410 | min: orca-slicer failed (exit status: 156): Flow::spacing() produced negative spacing. Did you set some extrusion width too small? |
| PASS | long_retractions_when_cut | coBools | 2 | src/libslic3r/PrintConfig.cpp:5214 |  |
| PASS | long_retractions_when_ec | coBools | 2 | src/libslic3r/PrintConfig.cpp:5229 |  |
| UNBOUNDED | machine_end_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:2028 | non boolean/enum/range option |
| UNBOUNDED | machine_load_filament_time | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2560 | bounded min/max not explicit in definition block |
| PASS | machine_max_acceleration_e | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1260 |  |
| UNBOUNDED | machine_max_acceleration_extruding | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4601 | bounded min/max not explicit in definition block |
| UNBOUNDED | machine_max_acceleration_retracting | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4615 | bounded min/max not explicit in definition block |
| UNBOUNDED | machine_max_acceleration_travel | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4626 | bounded min/max not explicit in definition block |
| PASS | machine_max_acceleration_x | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1257 |  |
| PASS | machine_max_acceleration_y | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1258 |  |
| PASS | machine_max_acceleration_z | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1259 |  |
| PASS | machine_max_jerk_e | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1276 |  |
| PASS | machine_max_jerk_x | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1273 |  |
| PASS | machine_max_jerk_y | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1274 |  |
| PASS | machine_max_jerk_z | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1275 |  |
| PASS | machine_max_junction_deviation | coFloats | 3 | src/libslic3r/PrintConfig.cpp:4570 |  |
| UNBOUNDED | machine_max_speed_e | coFloats | 0 | src/libslic3r/PrintConfig.hpp:1265 | bounded min/max not explicit in definition block |
| PASS | machine_max_speed_x | coFloats | 3 | src/libslic3r/PrintConfig.hpp:1262 |  |
| UNBOUNDED | machine_max_speed_y | coFloats | 0 | src/libslic3r/PrintConfig.hpp:1263 | bounded min/max not explicit in definition block |
| UNBOUNDED | machine_max_speed_z | coFloats | 0 | src/libslic3r/PrintConfig.hpp:1264 | bounded min/max not explicit in definition block |
| UNBOUNDED | machine_min_extruding_rate | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4581 | bounded min/max not explicit in definition block |
| UNBOUNDED | machine_min_travel_rate | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4591 | bounded min/max not explicit in definition block |
| UNBOUNDED | machine_pause_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:4454 | non boolean/enum/range option |
| UNBOUNDED | machine_start_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:5940 | non boolean/enum/range option |
| UNBOUNDED | machine_tool_change_time | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2578 | bounded min/max not explicit in definition block |
| UNBOUNDED | machine_unload_filament_time | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2569 | bounded min/max not explicit in definition block |
| PASS | make_overhang_printable | coBool | 2 | src/libslic3r/PrintConfig.cpp:4974 |  |
| PASS | make_overhang_printable_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4981 |  |
| UNBOUNDED | make_overhang_printable_hole_size | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4993 | bounded min/max not explicit in definition block |
| PASS | manual_filament_change | coBool | 2 | src/libslic3r/PrintConfig.cpp:5964 |  |
| UNBOUNDED | master_extruder_id | coInt | 0 | src/libslic3r/PrintConfig.cpp:5399 | bounded min/max not explicit in definition block |
| UNBOUNDED | max_bridge_length | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2019 | bounded min/max not explicit in definition block |
| UNBOUNDED | max_layer_height | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4721 | bounded min/max not explicit in definition block |
| UNBOUNDED | max_resonance_avoidance_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4653 | bounded min/max not explicit in definition block |
| UNBOUNDED | max_travel_detour_distance | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:948 | bounded min/max not explicit in definition block |
| UNBOUNDED | max_volumetric_extrusion_rate_slope | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4730 | bounded min/max not explicit in definition block |
| PASS | max_volumetric_extrusion_rate_slope_segment_length | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4751 |  |
| UNBOUNDED | min_bead_width | coPercent | 0 | src/libslic3r/PrintConfig.cpp:7275 | bounded min/max not explicit in definition block |
| UNBOUNDED | min_feature_size | coPercent | 0 | src/libslic3r/PrintConfig.cpp:7217 | bounded min/max not explicit in definition block |
| UNBOUNDED | min_layer_height | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4817 | bounded min/max not explicit in definition block |
| PASS | min_length_factor | coFloat | 3 | src/libslic3r/PrintConfig.cpp:7228 |  |
| UNBOUNDED | min_resonance_avoidance_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4645 | bounded min/max not explicit in definition block |
| UNBOUNDED | min_skirt_length | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5751 | bounded min/max not explicit in definition block |
| UNBOUNDED | min_width_top_surface | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:1498 | bounded min/max not explicit in definition block |
| UNBOUNDED | minimum_sparse_infill_area | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5772 | bounded min/max not explicit in definition block |
| UNBOUNDED | mmu_segmented_region_interlocking_depth | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4215 | bounded min/max not explicit in definition block |
| UNBOUNDED | mmu_segmented_region_max_width | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4206 | bounded min/max not explicit in definition block |
| UNBOUNDED | notes | coString | 0 | src/libslic3r/PrintConfig.cpp:4843 | non boolean/enum/range option |
| UNBOUNDED | nozzle_diameter | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4835 | bounded min/max not explicit in definition block |
| UNBOUNDED | nozzle_flush_dataset | coInts | 0 | src/libslic3r/PrintConfig.cpp:2602 | bounded min/max not explicit in definition block |
| UNBOUNDED | nozzle_height | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2242 | bounded min/max not explicit in definition block |
| PASS | nozzle_hrc | coInt | 3 | src/libslic3r/PrintConfig.cpp:3792 |  |
| UNBOUNDED | nozzle_temperature | coInts | 0 | src/libslic3r/PrintConfig.cpp:6644 | bounded min/max not explicit in definition block |
| UNBOUNDED | nozzle_temperature_initial_layer | coInts | 0 | src/libslic3r/PrintConfig.cpp:3436 | bounded min/max not explicit in definition block |
| UNBOUNDED | nozzle_temperature_range_high | coInts | 0 | src/libslic3r/PrintConfig.cpp:6661 | bounded min/max not explicit in definition block |
| UNBOUNDED | nozzle_temperature_range_low | coInts | 0 | src/libslic3r/PrintConfig.cpp:6653 | bounded min/max not explicit in definition block |
| PASS | nozzle_type | coEnums | 5 | src/libslic3r/PrintConfig.cpp:3772 |  |
| UNBOUNDED | nozzle_volume | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4894 | bounded min/max not explicit in definition block |
| PASS | nozzle_volume_type | coEnums | 2 | src/libslic3r/PrintConfig.cpp:5348 |  |
| PASS | only_one_wall_first_layer | coBool | 2 | src/libslic3r/PrintConfig.cpp:1513 |  |
| PASS | only_one_wall_top | coBool | 2 | src/libslic3r/PrintConfig.cpp:1491 |  |
| PASS | ooze_prevention | coBool | 2 | src/libslic3r/PrintConfig.cpp:4961 |  |
| PASS | other_layers_print_sequence | coInts | 3 | src/libslic3r/PrintConfig.cpp:1115 |  |
| UNBOUNDED | other_layers_print_sequence_nums | coInt | 0 | src/libslic3r/PrintConfig.cpp:1121 | bounded min/max not explicit in definition block |
| UNBOUNDED | outer_wall_acceleration | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3215 | bounded min/max not explicit in definition block |
| UNBOUNDED | outer_wall_filament_id | coInt | 0 | src/libslic3r/PrintConfig.cpp:5011 | bounded min/max not explicit in definition block |
| PASS | outer_wall_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1404 |  |
| UNBOUNDED | outer_wall_jerk | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3308 | bounded min/max not explicit in definition block |
| FAIL | outer_wall_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:2115 | min: invalid external perimeter flow spacing |
| UNBOUNDED | outer_wall_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2127 | bounded min/max not explicit in definition block |
| UNBOUNDED | overhang_1_4_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:1610 | bounded min/max not explicit in definition block |
| UNBOUNDED | overhang_2_4_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:1622 | bounded min/max not explicit in definition block |
| UNBOUNDED | overhang_3_4_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:1634 | bounded min/max not explicit in definition block |
| UNBOUNDED | overhang_4_4_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:1646 | bounded min/max not explicit in definition block |
| PASS | overhang_fan_speed | coInts | 3 | src/libslic3r/PrintConfig.cpp:1214 |  |
| PASS | overhang_fan_threshold | coEnums | 6 | src/libslic3r/PrintConfig.cpp:1227 |  |
| PASS | overhang_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1424 |  |
| PASS | overhang_reverse | coBool | 2 | src/libslic3r/PrintConfig.cpp:1526 |  |
| PASS | overhang_reverse_internal_only | coBool | 2 | src/libslic3r/PrintConfig.cpp:1534 |  |
| UNBOUNDED | overhang_reverse_threshold | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:1565 | bounded min/max not explicit in definition block |
| UNBOUNDED | parallel_printheads_bed_exclude_areas | coStrings | 0 | src/libslic3r/PrintConfig.cpp:712 | non boolean/enum/range option |
| PASS | parallel_printheads_count | coInt | 3 | src/libslic3r/PrintConfig.cpp:704 |  |
| UNBOUNDED | parking_pos_retraction | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4927 | bounded min/max not explicit in definition block |
| PASS | part_cooling_fan_min_pwm | coInt | 3 | src/libslic3r/PrintConfig.cpp:3861 |  |
| UNBOUNDED | pellet_flow_coefficient | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2639 | bounded min/max not explicit in definition block |
| PASS | pellet_modded_printer | coBool | 2 | src/libslic3r/PrintConfig.cpp:3939 |  |
| UNBOUNDED | physical_extruder_map | coInts | 0 | src/libslic3r/PrintConfig.cpp:2495 | bounded min/max not explicit in definition block |
| UNBOUNDED | post_process | coStrings | 0 | src/libslic3r/PrintConfig.cpp:5068 | non boolean/enum/range option |
| PASS | precise_outer_wall | coBool | 2 | src/libslic3r/PrintConfig.cpp:1484 |  |
| PASS | precise_z_height | coBool | 2 | src/libslic3r/PrintConfig.cpp:3717 |  |
| PASS | preferred_orientation | coFloat | 3 | src/libslic3r/PrintConfig.cpp:797 |  |
| PASS | preheat_steps | coInt | 3 | src/libslic3r/PrintConfig.cpp:5918 |  |
| PASS | preheat_time | coFloat | 3 | src/libslic3r/PrintConfig.cpp:5908 |  |
| UNBOUNDED | pressure_advance | coFloats | 0 | src/libslic3r/PrintConfig.cpp:2345 | bounded min/max not explicit in definition block |
| UNBOUNDED | prime_tower_brim_width | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6891 | bounded min/max not explicit in definition block |
| PASS | prime_tower_enable_framework | coBool | 2 | src/libslic3r/PrintConfig.cpp:6819 |  |
| PASS | prime_tower_flat_ironing | coBool | 2 | src/libslic3r/PrintConfig.cpp:6989 |  |
| UNBOUNDED | prime_tower_infill_gap | coPercent | 0 | src/libslic3r/PrintConfig.cpp:7005 | bounded min/max not explicit in definition block |
| PASS | prime_tower_skip_points | coBool | 2 | src/libslic3r/PrintConfig.cpp:6983 |  |
| UNBOUNDED | prime_tower_width | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6876 | bounded min/max not explicit in definition block |
| UNBOUNDED | prime_volume | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6852 | bounded min/max not explicit in definition block |
| UNBOUNDED | print_compatible_printers | coStrings | 0 | src/libslic3r/PrintConfig.cpp:1832 | non boolean/enum/range option |
| UNBOUNDED | print_extruder_id | coInts | 0 | src/libslic3r/PrintConfig.cpp:5405 | bounded min/max not explicit in definition block |
| UNBOUNDED | print_extruder_variant | coStrings | 0 | src/libslic3r/PrintConfig.cpp:5412 | non boolean/enum/range option |
| PASS | print_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:2327 |  |
| PASS | print_order | coEnum | 2 | src/libslic3r/PrintConfig.cpp:1847 |  |
| PASS | print_sequence | coEnum | 2 | src/libslic3r/PrintConfig.cpp:1836 |  |
| UNBOUNDED | print_settings_id | coString | 0 | src/libslic3r/PrintConfig.cpp:5111 | non boolean/enum/range option |
| UNBOUNDED | printable_area | coPoints | 0 | src/libslic3r/PrintConfig.cpp:686 | non boolean/enum/range option |
| FAIL | printable_height | coFloat | 3 | src/libslic3r/PrintConfig.cpp:779 | min: orca-slicer failed (exit status: 205): The object cube10.stl exceeds the maximum build volume height. |
| UNBOUNDED | printer_agent | coString | 0 | src/libslic3r/PrintConfig.cpp:829 | non boolean/enum/range option |
| UNBOUNDED | printer_extruder_id | coInts | 0 | src/libslic3r/PrintConfig.cpp:5385 | bounded min/max not explicit in definition block |
| UNBOUNDED | printer_extruder_variant | coStrings | 0 | src/libslic3r/PrintConfig.cpp:5392 | non boolean/enum/range option |
| UNBOUNDED | printer_model | coString | 0 | src/libslic3r/PrintConfig.cpp:5090 | non boolean/enum/range option |
| UNBOUNDED | printer_notes | coString | 0 | src/libslic3r/PrintConfig.cpp:5096 | non boolean/enum/range option |
| UNBOUNDED | printer_settings_id | coString | 0 | src/libslic3r/PrintConfig.cpp:5116 | non boolean/enum/range option |
| PASS | printer_structure | coEnum | 5 | src/libslic3r/PrintConfig.cpp:3801 |  |
| PASS | printer_technology | coEnum | 2 | src/libslic3r/PrintConfig.cpp:678 |  |
| UNBOUNDED | printer_variant | coString | 0 | src/libslic3r/PrintConfig.cpp:5105 | non boolean/enum/range option |
| PASS | printhost_authorization_type | coEnum | 2 | src/libslic3r/PrintConfig.cpp:915 |  |
| PASS | printhost_ssl_ignore_revoke | coBool | 2 | src/libslic3r/PrintConfig.cpp:901 |  |
| UNBOUNDED | printing_by_object_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:2037 | non boolean/enum/range option |
| UNBOUNDED | process_change_extrusion_role_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:5081 | non boolean/enum/range option |
| PASS | purge_in_prime_tower | coBool | 2 | src/libslic3r/PrintConfig.cpp:5983 |  |
| UNBOUNDED | raft_contact_distance | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5121 | bounded min/max not explicit in definition block |
| UNBOUNDED | raft_expansion | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5132 | bounded min/max not explicit in definition block |
| PASS | raft_first_layer_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:5141 |  |
| UNBOUNDED | raft_first_layer_expansion | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5151 | bounded min/max not explicit in definition block |
| FAIL | raft_layers | coInt | 3 | src/libslic3r/PrintConfig.cpp:5161 | max: unsupported project feature: raft_layers |
| FAIL | reduce_crossing_wall | coBool | 2 | src/libslic3r/PrintConfig.cpp:941 | true: layer 2 travel geometry count differs: expected 19, actual 14 |
| PASS | reduce_fan_stop_start_freq | coBools | 2 | src/libslic3r/PrintConfig.cpp:2422 |  |
| PASS | reduce_infill_retraction | coBool | 2 | src/libslic3r/PrintConfig.cpp:4953 |  |
| PASS | relative_bridge_angle | coBool | 2 | src/libslic3r/PrintConfig.cpp:1285 |  |
| PASS | required_nozzle_HRC | coInts | 3 | src/libslic3r/PrintConfig.cpp:2481 |  |
| UNBOUNDED | resolution | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5172 | bounded min/max not explicit in definition block |
| PASS | resonance_avoidance | coBool | 2 | src/libslic3r/PrintConfig.cpp:4637 |  |
| UNBOUNDED | retract_before_wipe | coPercents | 0 | src/libslic3r/PrintConfig.cpp:5188 | bounded min/max not explicit in definition block |
| UNBOUNDED | retract_length_toolchange | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5244 | bounded min/max not explicit in definition block |
| UNBOUNDED | retract_lift_above | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5266 | bounded min/max not explicit in definition block |
| UNBOUNDED | retract_lift_below | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5274 | bounded min/max not explicit in definition block |
| FAIL | retract_lift_enforce | coEnums | 4 | src/libslic3r/PrintConfig.cpp:5320 | Bottom Only: layer 2 travel geometry count differs: expected 8, actual 14 |
| UNBOUNDED | retract_restart_extra | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5439 | bounded min/max not explicit in definition block |
| UNBOUNDED | retract_restart_extra_toolchange | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5447 | bounded min/max not explicit in definition block |
| FAIL | retract_when_changing_layer | coBools | 2 | src/libslic3r/PrintConfig.cpp:5195 | false: layer 50 island lifecycle differs: expected [[Extruder { extrusion: "-3.81129", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.558", y: "106.502", z: "10" }, end: Position { x: "113.488", y: "106.467", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09446", feed: "3000" }, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.488", y: "106.467", z: "10" }, end: Position { x: "113.556", y: "106.428", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09425", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "106.527", y: "105.795", z: "10" }, end: Position { x: "105.82", y: "106.502", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "1800" }, WipeEnd]], actual [[Extruder { extrusion: "-3.81129", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.558", y: "106.502", z: "10" }, end: Position { x: "113.488", y: "106.467", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09446", feed: "3000" }, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.488", y: "106.467", z: "10" }, end: Position { x: "113.556", y: "106.428", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09425", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }]] |
| PASS | retraction_distances_when_cut | coFloats | 3 | src/libslic3r/PrintConfig.cpp:5221 |  |
| PASS | retraction_distances_when_ec | coFloats | 3 | src/libslic3r/PrintConfig.cpp:5235 |  |
| UNBOUNDED | retraction_length | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5201 | bounded min/max not explicit in definition block |
| UNBOUNDED | retraction_minimum_travel | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5181 | bounded min/max not explicit in definition block |
| UNBOUNDED | retraction_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:5455 | bounded min/max not explicit in definition block |
| PASS | role_based_wipe_speed | coBool | 2 | src/libslic3r/PrintConfig.cpp:5635 |  |
| PASS | scan_first_layer | coBool | 2 | src/libslic3r/PrintConfig.cpp:3745 |  |
| PASS | scarf_angle_threshold | coInt | 3 | src/libslic3r/PrintConfig.cpp:5546 |  |
| PASS | scarf_joint_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:5584 |  |
| UNBOUNDED | scarf_joint_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:5570 | bounded min/max not explicit in definition block |
| UNBOUNDED | scarf_overhang_threshold | coPercent | 0 | src/libslic3r/PrintConfig.cpp:5558 | bounded min/max not explicit in definition block |
| UNBOUNDED | seam_gap | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:5515 | bounded min/max not explicit in definition block |
| FAIL | seam_position | coEnum | 5 | src/libslic3r/PrintConfig.cpp:5490 | nearest: layer 1 deposition 59 differs: expected Deposition { feature: "Inner wall", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "105.73", y: "114.27", z: "0.2" }, end: Position { x: "105.73", y: "105.77", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.26118", feed: 1800.0, acceleration: "500", fans: "0:0" }, actual Deposition { feature: "Inner wall", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "105.73", y: "114.27", z: "0.2" }, end: Position { x: "105.73", y: "105.73", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.26241", feed: 1800.0, acceleration: "500", fans: "0:0" } |
| PASS | seam_slope_conditional | coBool | 2 | src/libslic3r/PrintConfig.cpp:5539 |  |
| PASS | seam_slope_entire_loop | coBool | 2 | src/libslic3r/PrintConfig.cpp:5604 |  |
| PASS | seam_slope_inner_walls | coBool | 2 | src/libslic3r/PrintConfig.cpp:5628 |  |
| UNBOUNDED | seam_slope_min_length | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5611 | bounded min/max not explicit in definition block |
| UNBOUNDED | seam_slope_start_height | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:5593 | bounded min/max not explicit in definition block |
| UNBOUNDED | seam_slope_steps | coInt | 0 | src/libslic3r/PrintConfig.cpp:5620 | bounded min/max not explicit in definition block |
| FAIL | seam_slope_type | coEnum | 3 | src/libslic3r/PrintConfig.cpp:5525 | all: layer 2 deposition count differs: expected 129, actual 97 |
| PASS | set_other_flow_ratios | coBool | 2 | src/libslic3r/PrintConfig.cpp:1387 |  |
| PASS | silent_mode | coBool | 2 | src/libslic3r/PrintConfig.cpp:4440 |  |
| PASS | single_extruder_multi_material | coBool | 2 | src/libslic3r/PrintConfig.cpp:5958 |  |
| PASS | single_extruder_multi_material_priming | coBool | 2 | src/libslic3r/PrintConfig.cpp:6014 |  |
| FAIL | single_loop_draft_shield | coBool | 2 | src/libslic3r/PrintConfig.cpp:5700 | true: filament 1 length differs: expected 260.62mm, actual 262.34mm |
| PASS | skeleton_infill_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:4018 |  |
| UNBOUNDED | skeleton_infill_line_width | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:4074 | bounded min/max not explicit in definition block |
| PASS | skin_infill_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:4031 |  |
| PASS | skin_infill_depth | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4044 |  |
| UNBOUNDED | skin_infill_line_width | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:4064 | bounded min/max not explicit in definition block |
| FAIL | skirt_distance | coFloat | 3 | src/libslic3r/PrintConfig.cpp:5673 | seeded: layer 1 deposition 138 differs: expected Deposition { feature: "Skirt", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "75.1", y: "79.206", z: "0.2" }, end: Position { x: "77.148", y: "77.149", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.08918", feed: 3000.0, acceleration: "500", fans: "0:0" }, actual Deposition { feature: "Skirt", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "75.1", y: "79.206", z: "0.2" }, end: Position { x: "77.148", y: "77.149", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.08917", feed: 3000.0, acceleration: "500", fans: "0:0" } |
| UNBOUNDED | skirt_height | coInt | 0 | src/libslic3r/PrintConfig.cpp:5692 | bounded min/max not explicit in definition block |
| FAIL | skirt_loops | coInt | 3 | src/libslic3r/PrintConfig.cpp:5733 | max: layer 1 deposition 70 differs: expected Deposition { feature: "Skirt", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "100.46", y: "116.644", z: "0.2" }, end: Position { x: "100.172", y: "115", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.05129", feed: 3000.0, acceleration: "500", fans: "0:0" }, actual Deposition { feature: "Skirt", width: "0.42", motion: MotionRecord { command: "G1", start: Position { x: "100.461", y: "116.644", z: "0.2" }, end: Position { x: "100.172", y: "115", z: "0.2" }, arc_center: [None, None], turns: None }, extrusion: "0.05129", feed: 3000.0, acceleration: "500", fans: "0:0" } |
| UNBOUNDED | skirt_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:5742 | bounded min/max not explicit in definition block |
| PASS | skirt_start_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:5682 |  |
| FAIL | skirt_type | coEnum | 2 | src/libslic3r/PrintConfig.cpp:5721 | perobject: unsupported project feature: skirt_type per-object |
| UNBOUNDED | slice_closing_radius | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6020 | bounded min/max not explicit in definition block |
| PASS | slicing_mode | coEnum | 3 | src/libslic3r/PrintConfig.cpp:6030 |  |
| PASS | slow_down_for_layer_cooling | coBools | 2 | src/libslic3r/PrintConfig.cpp:1858 |  |
| PASS | slow_down_layer_time | coFloats | 3 | src/libslic3r/PrintConfig.cpp:5762 |  |
| UNBOUNDED | slow_down_layers | coInt | 0 | src/libslic3r/PrintConfig.cpp:3426 | bounded min/max not explicit in definition block |
| UNBOUNDED | slow_down_min_speed | coFloats | 0 | src/libslic3r/PrintConfig.cpp:4826 | bounded min/max not explicit in definition block |
| PASS | slowdown_for_curled_perimeters | coBool | 2 | src/libslic3r/PrintConfig.cpp:1587 |  |
| FAIL | small_area_infill_flow_compensation | coBool | 2 | src/libslic3r/PrintConfig.cpp:4472 | true: filament 1 length differs: expected 259.24mm, actual 262.34mm |
| UNBOUNDED | small_area_infill_flow_compensation_model | coStrings | 0 | src/libslic3r/PrintConfig.cpp:4479 | non boolean/enum/range option |
| UNBOUNDED | small_perimeter_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:2137 | bounded min/max not explicit in definition block |
| UNBOUNDED | small_perimeter_threshold | coFloat | 0 | src/libslic3r/PrintConfig.cpp:2149 | bounded min/max not explicit in definition block |
| PASS | solid_infill_direction | coFloat | 3 | src/libslic3r/PrintConfig.cpp:2959 |  |
| UNBOUNDED | solid_infill_rotate_template | coString | 0 | src/libslic3r/PrintConfig.cpp:4007 | non boolean/enum/range option |
| UNBOUNDED | sparse_infill_acceleration | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:3234 | bounded min/max not explicit in definition block |
| FAIL | sparse_infill_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:2969 | min: layer 3 deposition 1 differs: expected Deposition { feature: "Inner wall", width: "0.45", motion: MotionRecord { command: "G1", start: Position { x: "105.675", y: "105.675", z: "0.6" }, end: Position { x: "114.325", y: "105.675", z: "0.6" }, arc_center: [None, None], turns: None }, extrusion: "0.28694", feed: 2105.0, acceleration: "500", fans: "0:255" }, actual Deposition { feature: "Inner wall", width: "0.45", motion: MotionRecord { command: "G1", start: Position { x: "105.675", y: "105.675", z: "0.6" }, end: Position { x: "114.325", y: "105.675", z: "0.6" }, arc_center: [None, None], turns: None }, extrusion: "0.28694", feed: 2005.0, acceleration: "500", fans: "0:255" } |
| UNBOUNDED | sparse_infill_filament_id | coInt | 0 | src/libslic3r/PrintConfig.cpp:4127 | bounded min/max not explicit in definition block |
| PASS | sparse_infill_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1434 |  |
| FAIL | sparse_infill_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:4136 | max: orca-slicer failed (exit status: 238): sparse_infill_line_width: too large line width 4.000000 |
| FAIL | sparse_infill_pattern | coEnum | 26 | src/libslic3r/PrintConfig.cpp:3017 | 3dhoneycomb: unsupported project feature: sparse_infill_pattern |
| UNBOUNDED | sparse_infill_rotate_template | coString | 0 | src/libslic3r/PrintConfig.cpp:3993 | non boolean/enum/range option |
| UNBOUNDED | sparse_infill_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:4174 | bounded min/max not explicit in definition block |
| PASS | spiral_finishing_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:5868 |  |
| FAIL | spiral_mode | coBool | 2 | src/libslic3r/PrintConfig.cpp:5829 | true: unsupported project feature: spiral_mode |
| PASS | spiral_mode_max_xy_smoothing | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:5844 |  |
| PASS | spiral_mode_smooth | coBool | 2 | src/libslic3r/PrintConfig.cpp:5837 |  |
| PASS | spiral_starting_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:5857 |  |
| FAIL | staggered_inner_seams | coBool | 2 | src/libslic3r/PrintConfig.cpp:5508 | true: layer 1 deposition count differs: expected 100, actual 99 |
| UNBOUNDED | standby_temperature_delta | coInt | 0 | src/libslic3r/PrintConfig.cpp:5896 | bounded min/max not explicit in definition block |
| UNBOUNDED | start_end_points | coPoints | 0 | src/libslic3r/PrintConfig.cpp:4945 | non boolean/enum/range option |
| PASS | supertack_plate_temp | coInts | 3 | src/libslic3r/PrintConfig.cpp:961 |  |
| PASS | supertack_plate_temp_initial_layer | coInts | 3 | src/libslic3r/PrintConfig.cpp:1021 |  |
| PASS | support_air_filtration | coBool | 2 | src/libslic3r/PrintConfig.cpp:3899 |  |
| PASS | support_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6100 |  |
| PASS | support_base_pattern | coEnum | 6 | src/libslic3r/PrintConfig.cpp:6284 |  |
| UNBOUNDED | support_base_pattern_spacing | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6329 | bounded min/max not explicit in definition block |
| UNBOUNDED | support_bottom_interface_spacing | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6266 | bounded min/max not explicit in definition block |
| UNBOUNDED | support_bottom_z_distance | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6153 | bounded min/max not explicit in definition block |
| PASS | support_chamber_temp_control | coBool | 2 | src/libslic3r/PrintConfig.cpp:3892 |  |
| PASS | support_critical_regions_only | coBool | 2 | src/libslic3r/PrintConfig.cpp:6118 |  |
| UNBOUNDED | support_expansion | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6338 | bounded min/max not explicit in definition block |
| UNBOUNDED | support_filament | coInt | 0 | src/libslic3r/PrintConfig.cpp:6178 | bounded min/max not explicit in definition block |
| PASS | support_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1464 |  |
| UNBOUNDED | support_interface_bottom_layers | coInt | 0 | src/libslic3r/PrintConfig.cpp:6241 | bounded min/max not explicit in definition block |
| UNBOUNDED | support_interface_filament | coInt | 0 | src/libslic3r/PrintConfig.cpp:6213 | bounded min/max not explicit in definition block |
| PASS | support_interface_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1474 |  |
| PASS | support_interface_loop_pattern | coBool | 2 | src/libslic3r/PrintConfig.cpp:6206 |  |
| PASS | support_interface_not_for_body | coBool | 2 | src/libslic3r/PrintConfig.cpp:6187 |  |
| PASS | support_interface_pattern | coEnum | 5 | src/libslic3r/PrintConfig.cpp:6309 |  |
| UNBOUNDED | support_interface_spacing | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6255 | bounded min/max not explicit in definition block |
| UNBOUNDED | support_interface_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6275 | bounded min/max not explicit in definition block |
| UNBOUNDED | support_interface_top_layers | coInt | 0 | src/libslic3r/PrintConfig.cpp:6223 | bounded min/max not explicit in definition block |
| PASS | support_ironing | coBool | 2 | src/libslic3r/PrintConfig.cpp:6557 |  |
| PASS | support_ironing_flow | coPercent | 3 | src/libslic3r/PrintConfig.cpp:6577 |  |
| PASS | support_ironing_pattern | coEnum | 2 | src/libslic3r/PrintConfig.cpp:6565 |  |
| PASS | support_ironing_spacing | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6589 |  |
| FAIL | support_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:6194 | max: orca-slicer failed (exit status: 238): support_line_width: too large line width 4.000000 |
| PASS | support_material_interface_fan_speed | coInts | 3 | src/libslic3r/PrintConfig.cpp:3457 |  |
| PASS | support_multi_bed_types | coBool | 2 | src/libslic3r/PrintConfig.cpp:3945 |  |
| PASS | support_object_first_layer_gap | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6089 |  |
| PASS | support_object_skip_flush | coBool | 2 | src/libslic3r/PrintConfig.cpp:2588 |  |
| PASS | support_object_xy_distance | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6078 |  |
| PASS | support_on_build_plate_only | coBool | 2 | src/libslic3r/PrintConfig.cpp:6110 |  |
| PASS | support_parallel_printheads | coBool | 2 | src/libslic3r/PrintConfig.cpp:698 |  |
| PASS | support_remove_small_overhang | coBool | 2 | src/libslic3r/PrintConfig.cpp:6125 |  |
| UNBOUNDED | support_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6346 | bounded min/max not explicit in definition block |
| PASS | support_style | coEnum | 7 | src/libslic3r/PrintConfig.cpp:6355 |  |
| PASS | support_threshold_angle | coInt | 3 | src/libslic3r/PrintConfig.cpp:6391 |  |
| PASS | support_threshold_overlap | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:6404 |  |
| UNBOUNDED | support_top_z_distance | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6134 | bounded min/max not explicit in definition block |
| PASS | support_type | coEnum | 4 | src/libslic3r/PrintConfig.cpp:6061 |  |
| PASS | symmetric_infill_y_axis | coBool | 2 | src/libslic3r/PrintConfig.cpp:4084 |  |
| UNBOUNDED | temperature_vitrification | coInts | 0 | src/libslic3r/PrintConfig.cpp:2917 | bounded min/max not explicit in definition block |
| UNBOUNDED | template_custom_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:4463 | non boolean/enum/range option |
| PASS | textured_cool_plate_temp | coInts | 3 | src/libslic3r/PrintConfig.cpp:981 |  |
| PASS | textured_cool_plate_temp_initial_layer | coInts | 3 | src/libslic3r/PrintConfig.cpp:1041 |  |
| PASS | textured_plate_temp | coInts | 3 | src/libslic3r/PrintConfig.cpp:1011 |  |
| PASS | textured_plate_temp_initial_layer | coInts | 3 | src/libslic3r/PrintConfig.cpp:1070 |  |
| PASS | thick_bridges | coBool | 2 | src/libslic3r/PrintConfig.cpp:1941 |  |
| PASS | thick_internal_bridges | coBool | 2 | src/libslic3r/PrintConfig.cpp:1950 |  |
| UNBOUNDED | thumbnails | coString | 0 | src/libslic3r/PrintConfig.cpp:7122 | non boolean/enum/range option |
| PASS | thumbnails_format | coEnum | 5 | src/libslic3r/PrintConfig.cpp:7129 |  |
| UNBOUNDED | time_cost | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3883 | bounded min/max not explicit in definition block |
| UNBOUNDED | time_lapse_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:4424 | non boolean/enum/range option |
| FAIL | timelapse_type | coEnum | 2 | src/libslic3r/PrintConfig.cpp:5879 | 1: orca-slicer failed (exit status: 154): run found error, return -102, exit... |
| PASS | tool_change_on_wipe_tower | coBool | 2 | src/libslic3r/PrintConfig.cpp:5995 |  |
| UNBOUNDED | top_bottom_infill_wall_overlap | coPercent | 0 | src/libslic3r/PrintConfig.cpp:4161 | bounded min/max not explicit in definition block |
| UNBOUNDED | top_shell_layers | coInt | 0 | src/libslic3r/PrintConfig.cpp:6730 | bounded min/max not explicit in definition block |
| UNBOUNDED | top_shell_thickness | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6741 | bounded min/max not explicit in definition block |
| FAIL | top_solid_infill_flow_ratio | coFloat | 3 | src/libslic3r/PrintConfig.cpp:1366 | min: layer 50 island lifecycle differs: expected [[Extruder { extrusion: "-3.81129", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.558", y: "106.502", z: "10" }, end: Position { x: "113.488", y: "106.467", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09446", feed: "3000" }, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.488", y: "106.467", z: "10" }, end: Position { x: "113.556", y: "106.428", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09425", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "106.527", y: "105.795", z: "10" }, end: Position { x: "105.82", y: "106.502", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "1800" }, WipeEnd]], actual [[Extruder { extrusion: "-3.81129", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.558", y: "106.502", z: "10" }, end: Position { x: "113.488", y: "106.467", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09446", feed: "3000" }, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "113.488", y: "106.467", z: "10" }, end: Position { x: "113.556", y: "106.428", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-0.09425", feed: "3000" }, WipeEnd], [Extruder { extrusion: "4", feed: "2400" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "0", feed: "1800" }, Extruder { extrusion: "-2.8", feed: "3600" }, WipeStart, Wipe { motion: MotionRecord { command: "G1", start: Position { x: "106.527", y: "105.795", z: "10" }, end: Position { x: "105.82", y: "106.502", z: "10" }, arc_center: [None, None], turns: None }, extrusion: "-1.2", feed: "1800" }, WipeEnd]] |
| UNBOUNDED | top_surface_acceleration | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3206 | bounded min/max not explicit in definition block |
| FAIL | top_surface_density | coPercent | 3 | src/libslic3r/PrintConfig.cpp:6752 | min: unsupported project feature: top_surface_density |
| UNBOUNDED | top_surface_filament_id | coInt | 0 | src/libslic3r/PrintConfig.cpp:5790 | bounded min/max not explicit in definition block |
| UNBOUNDED | top_surface_jerk | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3326 | bounded min/max not explicit in definition block |
| FAIL | top_surface_line_width | coFloatOrPercent | 3 | src/libslic3r/PrintConfig.cpp:6709 | max: orca-slicer failed (exit status: 238): top_surface_line_width: too large line width 4.000000 |
| FAIL | top_surface_pattern | coEnum | 8 | src/libslic3r/PrintConfig.cpp:2074 | alignedrectilinear: unsupported project feature: top_surface_pattern |
| UNBOUNDED | top_surface_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6721 | bounded min/max not explicit in definition block |
| UNBOUNDED | travel_acceleration | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3197 | bounded min/max not explicit in definition block |
| UNBOUNDED | travel_jerk | coFloat | 0 | src/libslic3r/PrintConfig.cpp:3353 | bounded min/max not explicit in definition block |
| PASS | travel_slope | coFloats | 3 | src/libslic3r/PrintConfig.cpp:5297 |  |
| UNBOUNDED | travel_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6776 | bounded min/max not explicit in definition block |
| UNBOUNDED | travel_speed_z | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6784 | bounded min/max not explicit in definition block |
| PASS | tree_support_angle_slow | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6437 |  |
| PASS | tree_support_auto_brim | coBool | 2 | src/libslic3r/PrintConfig.cpp:6483 |  |
| PASS | tree_support_branch_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6415 |  |
| PASS | tree_support_branch_angle_organic | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6426 |  |
| PASS | tree_support_branch_diameter | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6507 |  |
| PASS | tree_support_branch_diameter_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6517 |  |
| PASS | tree_support_branch_diameter_organic | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6531 |  |
| PASS | tree_support_branch_distance | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6449 |  |
| PASS | tree_support_branch_distance_organic | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6459 |  |
| UNBOUNDED | tree_support_brim_width | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6489 | bounded min/max not explicit in definition block |
| PASS | tree_support_tip_diameter | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6496 |  |
| UNBOUNDED | tree_support_top_rate | coPercent | 0 | src/libslic3r/PrintConfig.cpp:6469 | bounded min/max not explicit in definition block |
| PASS | tree_support_wall_count | coInt | 3 | src/libslic3r/PrintConfig.cpp:6541 |  |
| UNBOUNDED | upward_compatible_machine | coStrings | 0 | src/libslic3r/PrintConfig.cpp:1788 | non boolean/enum/range option |
| PASS | use_3mf | coBool | 2 | src/libslic3r/PrintConfig.cpp:821 |  |
| FAIL | use_firmware_retraction | coBool | 2 | src/libslic3r/PrintConfig.cpp:5471 | true: orca-slicer failed (exit status: 238): use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe |
| FAIL | use_relative_e_distances | coBool | 2 | src/libslic3r/PrintConfig.cpp:7146 | false: orca-slicer failed (exit status: 205): "G92 E0" was found in before_layer_change_gcode, which is incompatible with absolute extruder addressing. |
| UNBOUNDED | volumetric_speed_coefficients | coStrings | 0 | src/libslic3r/PrintConfig.cpp:2655 | non boolean/enum/range option |
| FAIL | wall_direction | coEnum | 2 | src/libslic3r/PrintConfig.cpp:2188 | cw: layer 1 deposition count differs: expected 99, actual 100 |
| UNBOUNDED | wall_distribution_count | coInt | 0 | src/libslic3r/PrintConfig.cpp:7208 | bounded min/max not explicit in definition block |
| FAIL | wall_generator | coEnum | 2 | src/libslic3r/PrintConfig.cpp:7155 | arachne: unsupported project feature: wall_generator |
| FAIL | wall_loops | coInt | 3 | src/libslic3r/PrintConfig.cpp:5051 | min: filament 1 length differs: expected 196.24mm, actual 26.72mm |
| PASS | wall_maximum_deviation | coFloat | 3 | src/libslic3r/PrintConfig.cpp:7253 |  |
| PASS | wall_maximum_resolution | coFloat | 3 | src/libslic3r/PrintConfig.cpp:7242 |  |
| PASS | wall_sequence | coEnum | 3 | src/libslic3r/PrintConfig.cpp:2158 |  |
| PASS | wall_transition_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:7195 |  |
| UNBOUNDED | wall_transition_filter_deviation | coPercent | 0 | src/libslic3r/PrintConfig.cpp:7180 | bounded min/max not explicit in definition block |
| UNBOUNDED | wall_transition_length | coPercent | 0 | src/libslic3r/PrintConfig.cpp:7169 | bounded min/max not explicit in definition block |
| PASS | wipe | coBools | 2 | src/libslic3r/PrintConfig.cpp:6794 |  |
| FAIL | wipe_before_external_loop | coBool | 2 | src/libslic3r/PrintConfig.cpp:5650 | true: layer 1 travel geometry count differs: expected 12, actual 11 |
| UNBOUNDED | wipe_distance | coFloats | 0 | src/libslic3r/PrintConfig.cpp:6801 | bounded min/max not explicit in definition block |
| PASS | wipe_on_loops | coBool | 2 | src/libslic3r/PrintConfig.cpp:5643 |  |
| UNBOUNDED | wipe_speed | coFloatOrPercent | 0 | src/libslic3r/PrintConfig.cpp:5661 | bounded min/max not explicit in definition block |
| UNBOUNDED | wipe_tower_bridging | coFloat | 0 | src/libslic3r/PrintConfig.cpp:7038 | bounded min/max not explicit in definition block |
| PASS | wipe_tower_cone_angle | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6902 |  |
| PASS | wipe_tower_extra_flow | coPercent | 3 | src/libslic3r/PrintConfig.cpp:7054 |  |
| UNBOUNDED | wipe_tower_extra_rib_length | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6941 | bounded min/max not explicit in definition block |
| PASS | wipe_tower_extra_spacing | coPercent | 3 | src/libslic3r/PrintConfig.cpp:7045 |  |
| UNBOUNDED | wipe_tower_filament | coInt | 0 | src/libslic3r/PrintConfig.cpp:6966 | bounded min/max not explicit in definition block |
| PASS | wipe_tower_fillet_wall | coBool | 2 | src/libslic3r/PrintConfig.cpp:6959 |  |
| UNBOUNDED | wipe_tower_max_purge_speed | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6912 | bounded min/max not explicit in definition block |
| PASS | wipe_tower_no_sparse_layers | coBool | 2 | src/libslic3r/PrintConfig.cpp:6006 |  |
| PASS | wipe_tower_rib_width | coFloat | 3 | src/libslic3r/PrintConfig.cpp:6950 |  |
| UNBOUNDED | wipe_tower_rotation_angle | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6884 | bounded min/max not explicit in definition block |
| PASS | wipe_tower_type | coEnum | 2 | src/libslic3r/PrintConfig.cpp:5972 |  |
| PASS | wipe_tower_wall_type | coEnum | 3 | src/libslic3r/PrintConfig.cpp:6925 |  |
| UNBOUNDED | wipe_tower_x | coFloats | 0 | src/libslic3r/PrintConfig.cpp:6860 | bounded min/max not explicit in definition block |
| UNBOUNDED | wipe_tower_y | coFloats | 0 | src/libslic3r/PrintConfig.cpp:6868 | bounded min/max not explicit in definition block |
| UNBOUNDED | wiping_volumes_extruders | coFloats | 0 | src/libslic3r/PrintConfig.cpp:6976 | bounded min/max not explicit in definition block |
| UNBOUNDED | wrapping_detection_gcode | coString | 0 | src/libslic3r/PrintConfig.cpp:4432 | non boolean/enum/range option |
| UNBOUNDED | wrapping_detection_layers | coInt | 0 | src/libslic3r/PrintConfig.cpp:4113 | bounded min/max not explicit in definition block |
| UNBOUNDED | wrapping_exclude_area | coPoints | 0 | src/libslic3r/PrintConfig.cpp:4120 | non boolean/enum/range option |
| UNBOUNDED | xy_contour_compensation | coFloat | 0 | src/libslic3r/PrintConfig.cpp:7083 | bounded min/max not explicit in definition block |
| UNBOUNDED | xy_hole_compensation | coFloat | 0 | src/libslic3r/PrintConfig.cpp:7073 | bounded min/max not explicit in definition block |
| FAIL | z_hop | coFloats | 3 | src/libslic3r/PrintConfig.cpp:5255 | seeded: layer 1 travel geometry differs: no match for Travel { motion: MotionRecord { command: "G1", start: Position { x: "2.3", y: "10", z: "0.2" }, end: Position { x: "2.3", y: "10", z: "1.996" }, arc_center: [None, None], turns: None }, feed: 9000.0, acceleration: "500" } |
| FAIL | z_hop_types | coEnums | 4 | src/libslic3r/PrintConfig.cpp:5282 | Auto Lift: layer 2 travel feed differs: no match for Travel { motion: MotionRecord { command: "G1", start: Position { x: "106.594", y: "117.566", z: "0.8" }, end: Position { x: "106.594", y: "117.566", z: "0.4" }, arc_center: [None, None], turns: None }, feed: 9000.0, acceleration: "500" } |
| UNBOUNDED | z_offset | coFloat | 0 | src/libslic3r/PrintConfig.cpp:6044 | bounded min/max not explicit in definition block |
| PASS | zaa_dont_alternate_fill_direction | coBool | 2 | src/libslic3r/PrintConfig.cpp:4397 |  |
| FAIL | zaa_enabled | coBool | 2 | src/libslic3r/PrintConfig.cpp:4378 | true: unsupported project feature: zaa_enabled |
| PASS | zaa_min_z | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4404 |  |
| PASS | zaa_minimize_perimeter_height | coFloat | 3 | src/libslic3r/PrintConfig.cpp:4385 |  |
