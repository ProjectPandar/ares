use crate::{ExtruderType, ProjectSettings};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProcessNozzle {
    pub(crate) physical: usize,
    pub(crate) volume: i32,
}

// PrintConfig.cpp:777–814,833–837. Hybrid is not a physical preset variant.
pub(crate) fn process_nozzle_config_index(
    settings: &ProjectSettings,
    nozzle: ProcessNozzle,
) -> usize {
    let extruder = settings
        .printer
        .gcode
        .extruder_type
        .0
        .get(nozzle.physical)
        .or_else(|| settings.printer.gcode.extruder_type.0.first())
        .expect("validated physical extruder type");
    let extruder = match extruder {
        ExtruderType::DirectDrive => "Direct Drive",
        ExtruderType::Bowden => "Bowden",
    };
    let variant = match nozzle.volume {
        0 => format!("{extruder} Standard"),
        1 => format!("{extruder} High Flow"),
        3 => format!("{extruder} TPU High Flow"),
        5 => format!("{extruder} E3D High Flow"),
        6 => format!("{extruder} Extra High Flow"),
        _ => String::new(),
    };
    let region = &settings.process.region;
    find_variant_index(
        &variant,
        i32::try_from(nozzle.physical).expect("validated physical extruder index") + 1,
        &region.print_extruder_variant.0,
        &region.print_extruder_id.0,
    )
}

fn find_variant_index(
    variant: &str,
    id: i32,
    variants: &[String],
    ids: &[crate::OrcaInt],
) -> usize {
    let same_id = |index: usize| {
        id < 0 || ids.is_empty() || ids.get(index).is_some_and(|value| value.0 == id)
    };
    variants
        .iter()
        .enumerate()
        .find_map(|(index, value)| (value == variant && same_id(index)).then_some(index))
        .or_else(|| {
            let count = if variants.is_empty() {
                ids.len()
            } else {
                variants.len()
            };
            (0..count).find(|&index| same_id(index))
        })
        .unwrap_or(0)
}
