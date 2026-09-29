//! Authored legal probes for schema-unbounded string/point domains
//! (`option_coverage::domains::authored_probes`).

/// Per-key authored legal probes for schema-unbounded string/point
/// domains. Every listed value is accepted by upstream validation
/// (free-form g-code templates, note fields, `XxY` polygon lists), so the
/// probes exercise real emit paths on both sides rather than a
/// value-less baseline.
pub(super) fn authored_probes(key: &str) -> Option<&'static [(&'static str, &'static str)]> {
    // Free-form g-code templates: any text is legal input; a non-empty
    // template with a placeholder exercises the placeholder engine.
    // `{layer_num}` is only defined once layer context exists
    // (`GCode.cpp:4669` sets it per layer); printer-scoped templates
    // rendered outside layer context (`file_start_gcode`, pauses,
    // custom/object transitions) throw on unknown variables
    // (`PlaceholderParser.cpp:997-1000`), so they get a literal probe.
    const TEMPLATE: &[(&str, &str)] = &[
        ("probe-empty", ""),
        ("probe-text", ";ARES_PROBE {layer_num}"),
    ];
    const TEMPLATE_LITERAL: &[(&str, &str)] = &[("probe-empty", ""), ("probe-text", ";ARES_PROBE")];
    const POLYGON: &[(&str, &str)] = &[("probe-rect", "10x10,20x10,20x20,10x20")];
    const POINT: &[(&str, &str)] = &[("probe", "5x5")];
    // Free-form metadata/note strings — content never alters toolpaths;
    // a distinct probe exercises profile round-trip equality.
    const NOTE: &[(&str, &str)] = &[("probe", "ares-parity-probe")];
    let probes = match key {
        // G-code template strings (`*_gcode`) — free-form by schema.
        // Layer-context templates — `{layer_num}` is defined here.
        // `before_layer_change_gcode` must keep the baseline `G92 E0`
        // under relative extruder addressing, so no empty probe.
        "before_layer_change_gcode" => &[("probe-text", "G92 E0\n;ARES_PROBE {layer_num}")],
        "layer_change_gcode" => TEMPLATE,
        // Contexts without layer variables get a literal probe only.
        "machine_pause_gcode"
        | "printing_by_object_gcode"
        | "change_filament_gcode"
        | "change_extrusion_role_gcode"
        | "process_change_extrusion_role_gcode"
        | "filament_change_extrusion_role_gcode"
        | "filament_start_gcode"
        | "filament_end_gcode"
        | "machine_start_gcode"
        | "machine_end_gcode"
        | "template_custom_gcode"
        | "wrapping_detection_gcode"
        | "file_start_gcode" => TEMPLATE_LITERAL,
        // `time_lapse_gcode` needs the same probe but is wrapped in
        // `;TIMELAPSE` handling — still free-form.
        "time_lapse_gcode" => TEMPLATE_LITERAL,
        // Free-form note/colour/profile metadata strings.
        "filament_notes"
        | "printer_notes"
        | "model_notes"
        | "notes"
        | "print_settings_id"
        | "filament_settings_id"
        | "printer_model"
        | "printer_variant"
        | "printer_agent"
        | "flashforge_serial_number"
        | "filament_type"
        | "filament_vendor"
        | "filament_ids"
        | "filament_extruder_variant"
        | "printer_extruder_variant"
        | "print_extruder_variant"
        | "extruder_variant_list"
        | "extruder_ams_count"
        | "filament_colour"
        | "default_filament_colour"
        | "extruder_colour"
        | "filament_multi_colour"
        | "filament_colour_type"
        | "default_bed_type"
        | "default_print_profile"
        | "default_filament_profile"
        | "bed_custom_texture"
        | "bed_custom_model" => NOTE,
        // An empty string is the legal "feature off" value for these
        // model/post-process/compatibility fields — it exercises
        // parse+serialize on both sides without external scripts or models.
        "adaptive_pressure_advance_model"
        | "small_area_infill_flow_compensation_model"
        | "volumetric_speed_coefficients"
        | "parallel_printheads_bed_exclude_areas"
        | "upward_compatible_machine" => &[("probe-empty", "")],
        // Full-bed rectangle: `printable_area` must still contain the placed
        // model, so probe the whole Ender-3 envelope rather than a sub-rect.
        "printable_area" => &[("probe-bed", "0x0,220x0,220x220,0x220")],
        // `extruder_printable_area` (coPointsGroups): one polygon per
        // extruder; the single-extruder profile takes one bed-sized group.
        "extruder_printable_area" => &[("probe-bed", "0x0,220x0,220x220,0x220")],
        // Infill rotation templates take a comma list of degrees.
        "solid_infill_rotate_template" | "sparse_infill_rotate_template" => &[("probe", "30,60")],
        // Thumbnail format `WxH` — one legal additional size.
        "thumbnails" => &[("probe", "300x300")],
        // Vitrification temperature (coInts, default 100).
        "temperature_vitrification" => &[("probe", "110")],
        // Hot-plate temperature declares max 300, but no minimum.
        "hot_plate_temp_initial_layer" => &[
            ("probe-zero", "0"),
            ("probe-interior", "55"),
            ("probe-max", "300"),
        ],
        // Filament-printable bitmask (default 3: printable everywhere).
        "filament_printable" => &[("probe", "1")],
        // Long-retraction-when-cut toggle (coInt 0/1).
        "enable_long_retraction_when_cut" => &[("probe", "1")],
        // Polygon point lists (`XxY` pairs) — a legal small rectangle.
        // `wrapping_exclude_area` rejected by the oracle (orca rejects the
        // rect probe upstream), so it keeps its unexercised baseline.
        "bed_exclude_area" | "head_wrap_detect_zone" | "start_end_points" => POLYGON,
        // `extruder_offset`: one offset per extruder — a shifted probe
        // stays legal for a single-extruder profile.
        "extruder_offset" => &[("probe-zero", "0x0"), ("probe-shift", "1x0.5")],
        // `extra_solid_infills` accepts `N`, `N#K`, or `1,7,9` layer
        // patterns (`PrintConfig.cpp:2987-2991`); `2` (every 2nd layer
        // solid) is a legal probe that changes toolpaths on both sides.
        "extra_solid_infills" => &[("probe-2", "2")],
        // Single `XxY` points.
        "bed_mesh_min" | "bed_mesh_max" | "bed_mesh_probe_distance" => POINT,
        // `best_object_pos` is normalized `[0,1]` (`PrintConfig.cpp:3818-3822`).
        "best_object_pos" => &[("probe", "0.25x0.75")],
        // Schema `def->min` values that upstream slice-time `validate`
        // rejects (`PrintConfig.cpp` `if (x <= 0) error_message` blocks,
        // e.g. :2225-2240 clearance, `layer_height` :771-777): the
        // schema-legal `0` probe makes the oracle abort, so probe the
        // effective legal interior instead.
        "extruder_clearance_height_to_lid"
        | "extruder_clearance_height_to_rod"
        | "extruder_clearance_radius" => &[("probe-min", "0.5"), ("probe-max", "250")],
        "nozzle_height" => &[("probe-min", "1"), ("probe-max", "400")],
        // `printable_height` below the object height aborts the slice.
        "printable_height" => &[("probe-min", "50"), ("probe-max", "400")],
        // `0.4` nozzle caps layer heights strictly below nozzle_diameter
        // (the 0.4 equal-to-nozzle probe is rejected at slice time).
        "layer_height" | "initial_layer_print_height" => {
            &[("probe-min", "0.05"), ("probe-max", "0.36")]
        }
        "bridge_flow" | "internal_bridge_flow" => &[("probe-min", "0.5"), ("probe-max", "2.0")],
        "filament_flow_ratio" => &[("probe-min", "0.5"), ("probe-max", "1.5")],
        // `filament_diameter` 0.5mm is rejected by upstream validation
        // (standard sizes only).
        "filament_diameter" => &[("probe-min", "1.0"), ("probe-max", "3.0")],
        "adaptive_pressure_advance_bridges" => &[("probe-min", "0.5"), ("probe-max", "2.0")],
        // Absolute widths below ~0.2mm trip upstream's "Too small line
        // width" validation; probe a legal absolute + a percent value.
        "skeleton_infill_line_width" | "skin_infill_line_width" => {
            &[("probe-min", "0.3"), ("probe-percent", "150%")]
        }
        _ => return None,
    };
    Some(probes)
}
