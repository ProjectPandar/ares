use super::{assert_project_value, assert_rejected, changed_case};

const DOMAINS: &[(&str, &[&str])] = &[
    ("bed_temperature_formula", &["by_first_filament", "by_highest_temp"]),
    ("enable_power_loss_recovery", &["printer_configuration", "enable", "disable"]),
    (
        "gcode_flavor",
        &[
            "marlin", "klipper", "reprapfirmware", "repetier", "marlin2", "reprap",
            "teacup", "makerware", "sailfish", "smoothie", "mach3", "machinekit",
            "no-extrusion",
        ],
    ),
    ("printer_structure", &["undefine", "corexy", "i3", "hbot", "delta"]),
    ("wipe_tower_type", &["type1", "type2"]),
    (
        "input_shaping_type",
        &[
            "Default", "MZV", "ZV", "ZVD", "ZVDD", "ZVDDD", "EI", "EI2", "2HUMP_EI",
            "EI3", "3HUMP_EI", "DAA", "Disable",
        ],
    ),
    (
        "host_type",
        &[
            "prusalink", "prusaconnect", "octoprint", "crealityprint", "duet", "flashair",
            "astrobox", "repetier", "mks", "esp3d", "obico", "flashforge", "simplyprint",
            "elegoolink", "3dprinteros", "moonraker",
        ],
    ),
    ("printer_technology", &["FFF", "SLA"]),
    ("printhost_authorization_type", &["key", "user"]),
    ("thumbnails_format", &["PNG", "JPG", "QOI", "BTT_TFT", "COLPIC"]),
    ("draft_shield", &["disabled", "enabled"]),
    ("print_order", &["default", "as_obj_list"]),
    ("print_sequence", &["by layer", "by object"]),
    ("skirt_type", &["combined", "perobject"]),
    ("timelapse_type", &["0", "1"]),
    ("wipe_tower_wall_type", &["rectangle", "cone", "rib"]),
    ("filament_map_mode", &["Auto For Flush", "Auto For Match", "Manual"]),
    (
        "curr_bed_type",
        &[
            "Default Plate", "Supertack Plate", "Cool Plate", "Engineering Plate",
            "High Temp Plate", "Textured PEI Plate", "Textured Cool Plate",
        ],
    ),
];

#[test]
fn every_typed_scalar_enum_domain_is_exact_and_complete() {
    assert_eq!(DOMAINS.len(), 18);
    for &(key, tokens) in DOMAINS {
        for &token in tokens {
            assert_project_value(key, token);
        }
        assert_rejected(key, "__invalid__");
        assert_rejected(key, &format!(" {} ", tokens[0]));
        if tokens[0].chars().any(|character| character.is_ascii_alphabetic()) {
            assert_rejected(key, &changed_case(tokens[0]));
        }
    }
}
