use super::super::{ProcessorLimits, estimate::Estimate};

fn used_filament(gcode: &[u8]) -> f64 {
    let settings = crate::ProjectSettings::default();
    let config = ProcessorLimits::default().with_settings(&settings);
    let lines = std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    Estimate::record(&lines, 0.0, &config, None)
        .print_statistics
        .total_used_filament
}

#[test]
fn m83_local_relative_e_survives_g90() {
    let gcode = b"M83\nG1 X1 E10\nG90\nG1 X2 E10\n";

    assert_eq!(format!("{:.2}", used_filament(gcode)), "20.00");
}

#[test]
fn g91_makes_e_relative_even_after_m82() {
    let gcode = b"M82\nG92 E0\nG1 X1 E10\nG91\nG1 X1 E10\nG90\nG1 X3 E25\n";

    assert_eq!(format!("{:.2}", used_filament(gcode)), "25.00");
}

// Native processor E2E failure inventory: premature ToolChange role flush,
// repeated/revisited role loss, ignored support/tower roles, flushing counted
// as extrusion, or missing EOF flush. Reference comes from the validated ELF
// map reader, not from a second implementation of the conversion formula.
#[test]
fn role_cache_boundaries_match_native_processor() {
    let reference = include_str!(
        "../../../../../../../tests/parity/role-filament-usage/cache-boundaries.gcode"
    );
    let native: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../../tests/parity/role-filament-usage/cache-boundaries-map.json"
    ))
    .unwrap();
    let mut settings = super::super::replay_config_tests::settings(reference);
    settings.project.gcode.filament_map.0 = vec![crate::OrcaInt(1), crate::OrcaInt(1)];
    let config = ProcessorLimits::default().with_settings(&settings);
    let lines = reference.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::record(&lines, 0.0, &config, None);
    let statistics = serde_json::to_value(estimate.print_statistics).unwrap();
    let roles = statistics["used_filaments_per_role"]
        .as_object()
        .expect("Native has role usage across ToolChange and EOF");
    let rows = native["roles"].as_array().unwrap();
    assert_eq!(roles.len(), rows.len());
    for row in rows {
        let role = row["role"].as_str().unwrap();
        let pair = roles[role].as_array().unwrap();
        assert_eq!(pair.len(), 2);
        for (index, value) in pair.iter().enumerate() {
            assert_eq!(
                value.as_f64().unwrap().to_bits(),
                row["bits"][index].as_u64().unwrap(),
                "{role}: {index} (meters, grams)"
            );
        }
    }
}
