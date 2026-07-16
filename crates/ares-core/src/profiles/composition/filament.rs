use crate::FilamentOptions;

pub(super) fn compose_filaments(filaments: Vec<FilamentOptions>) -> (FilamentOptions, Vec<usize>) {
    let variant_cardinalities = filaments
        .iter()
        .map(|filament| filament.gcode.filament_extruder_variant.0.len())
        .collect();
    let mut filaments = filaments.into_iter();
    let mut composed = filaments
        .next()
        .expect("profile selection guarantees at least one filament");
    for filament in filaments {
        composed.append(filament);
    }
    (composed, variant_cardinalities)
}
