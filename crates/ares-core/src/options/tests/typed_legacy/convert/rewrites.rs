use super::{assert_assign, scalar};

#[test]
fn every_rewrite_mapping_and_nonmatching_branch_is_exact() {
    let matches = [
        ("curr_bed_type", "SuperTack Plate", "curr_bed_type", "Supertack Plate"),
        ("timelapse_type", "2", "timelapse_type", "0"),
        ("support_type", "normal", "support_type", "normal(manual)"),
        ("support_type", "tree", "support_type", "tree(manual)"),
        ("support_type", "hybrid(auto)", "support_type", "tree(auto)"),
        ("support_base_pattern", "none", "support_base_pattern", "hollow"),
        ("overhang_fan_threshold", "5%", "overhang_fan_threshold", "10%"),
        ("enable_power_loss_recovery", "TrUe", "enable_power_loss_recovery", "enable"),
        ("enable_power_loss_recovery", "1", "enable_power_loss_recovery", "enable"),
        ("enable_power_loss_recovery", "FaLsE", "enable_power_loss_recovery", "disable"),
        ("enable_power_loss_recovery", "0", "enable_power_loss_recovery", "disable"),
        ("ensure_vertical_shell_thickness", "1", "ensure_vertical_shell_thickness", "ensure_all"),
        ("ensure_vertical_shell_thickness", "0", "ensure_vertical_shell_thickness", "ensure_moderate"),
        ("rotate_solid_infill_direction", "1", "solid_infill_rotate_template", "0,90"),
        ("rotate_solid_infill_direction", "0", "solid_infill_rotate_template", "0"),
        ("ironing_angle", "-45", "ironing_angle", "0"),
        ("draft_shield", "limited", "draft_shield", "disabled"),
        ("filament_map_mode", "Auto", "filament_map_mode", "Auto For Flush"),
        ("wall_direction", "auto", "wall_direction", "ccw"),
    ];

    for (source, input, target, output) in matches {
        assert_assign(scalar(source, input), target, output);
    }

    let nonmatches = [
        ("curr_bed_type", "superTack Plate", "curr_bed_type"),
        ("timelapse_type", "02", "timelapse_type"),
        ("support_type", "normal(auto)", "support_type"),
        ("support_base_pattern", "None", "support_base_pattern"),
        ("overhang_fan_threshold", "15%", "overhang_fan_threshold"),
        ("enable_power_loss_recovery", "yes", "enable_power_loss_recovery"),
        ("ensure_vertical_shell_thickness", "2", "ensure_vertical_shell_thickness"),
        ("rotate_solid_infill_direction", "2", "solid_infill_rotate_template"),
        ("ironing_angle", "45", "ironing_angle"),
        ("draft_shield", "enabled", "draft_shield"),
        ("filament_map_mode", "auto", "filament_map_mode"),
        ("wall_direction", "clockwise", "wall_direction"),
    ];

    for (source, input, target) in nonmatches {
        assert_assign(scalar(source, input), target, input);
    }
}

#[test]
fn every_wall_order_spelling_and_unknown_value_rename_to_wall_sequence() {
    let cases = [
        ("inner wall/outer wall/infill", "inner wall/outer wall"),
        ("infill/inner wall/outer wall", "inner wall/outer wall"),
        ("outer wall/inner wall/infill", "outer wall/inner wall"),
        ("infill/outer wall/inner wall", "outer wall/inner wall"),
        ("inner-outer-inner wall/infill", "inner-outer-inner wall"),
        ("outer wall/infill/inner wall", "outer wall/infill/inner wall"),
    ];

    for (input, output) in cases {
        assert_assign(scalar("wall_infill_order", input), "wall_sequence", output);
    }
}

#[test]
fn global_replacements_are_applied_everywhere_and_nonmatches_are_retained() {
    let variant_sources = [
        "nozzle_volume_type",
        "default_nozzle_volume_type",
        "printer_extruder_variant",
        "print_extruder_variant",
        "filament_extruder_variant",
        "extruder_variant_list",
    ];
    for source in variant_sources {
        assert_assign(
            scalar(source, "Normal/Big Traffic/Normal"),
            source,
            "Standard/High Flow/Standard",
        );
        assert_assign(scalar(source, "HighFlow"), source, "HighFlow");
    }
    assert_assign(
        scalar("extruder_type", "DirectDrive+DirectDrive"),
        "extruder_type",
        "Direct Drive+Direct Drive",
    );
    assert_assign(
        scalar("extruder_type", "Bowden"),
        "extruder_type",
        "Bowden",
    );
}

#[test]
fn all_six_pattern_rules_rewrite_only_exact_zig_zag() {
    let sources = [
        "sparse_infill_pattern",
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
        "ironing_pattern",
        "support_ironing_pattern",
    ];
    for source in sources {
        assert_assign(scalar(source, "zig-zag"), source, "rectilinear");
        assert_assign(scalar(source, "zigzag"), source, "zigzag");
    }
}

#[test]
fn filament_tokens_rebuild_all_tokens_only_when_exact_token_changes() {
    assert_assign(
        scalar("filament_type", "\"ASA-Aero\";PLA;\"PETG\""),
        "filament_type",
        "\"ASA-AERO\";\"PLA\";\"PETG\"",
    );
    assert_assign(
        scalar("filament_type", "\"ASA-Aero+\";PLA"),
        "filament_type",
        "\"ASA-Aero+\";PLA",
    );
    assert_assign(
        scalar("filament_type", "\"ASA-Aero\";"),
        "filament_type",
        "\"ASA-AERO\"",
    );
    assert_assign(scalar("filament_type", ""), "filament_type", "");
}
