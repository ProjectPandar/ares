//! Captured processor replay fixtures carry effective filament properties
//! in CONFIG_BLOCK; the post-processor must not use bootstrap defaults.
pub(super) fn settings(reference: &str) -> crate::ProjectSettings {
    let mut settings = crate::ProjectSettings::default();
    for (key, destination) in [
        (
            "filament_diameter",
            &mut settings.filament.gcode.filament_diameter,
        ),
        (
            "filament_density",
            &mut settings.filament.gcode.filament_density,
        ),
        ("filament_cost", &mut settings.filament.gcode.filament_cost),
    ] {
        let prefix = format!("; {key} = ");
        let raw = reference
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap();
        destination.0 = raw
            .split(',')
            .map(|value| crate::OrcaFloat(value.trim().parse().unwrap()))
            .collect();
    }
    settings
}
