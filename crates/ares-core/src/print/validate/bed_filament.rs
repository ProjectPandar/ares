//! Print.cpp:1751-1781: current-bed compatibility of used logical filaments.
use crate::{ProjectSettings, SliceError, options::ProjectBedType};

pub(super) fn validate(
    settings: &ProjectSettings,
    bbl: bool,
    used: &[usize],
    plate_number: u32,
) -> Result<(), SliceError> {
    let temperatures = &settings.filament.print;
    let (bed, values) = match settings.project.print.curr_bed_type {
        ProjectBedType::SupertackPlate => ("Supertack Plate", &temperatures.supertack_plate_temp),
        ProjectBedType::CoolPlate => ("Cool Plate", &temperatures.cool_plate_temp),
        ProjectBedType::TexturedCoolPlate => (
            "Textured Cool Plate",
            &temperatures.textured_cool_plate_temp,
        ),
        ProjectBedType::EngineeringPlate => ("Engineering Plate", &temperatures.eng_plate_temp),
        ProjectBedType::HighTempPlate => ("High Temp Plate", &temperatures.hot_plate_temp),
        ProjectBedType::TexturedPeiPlate => {
            ("Textured PEI Plate", &temperatures.textured_plate_temp)
        }
        // Accepted deviation: preserve the existing public diagnostic, not native UB.
        ProjectBedType::DefaultPlate => {
            return Err(SliceError::InvalidInput(
                "curr_bed_type Default Plate has no first-layer temperature".to_owned(),
            ));
        }
    };
    if !bbl && !settings.printer.gcode.support_multi_bed_types.0 {
        return Ok(());
    }
    // Print::extruders() supplies sorted/deduplicated logical indices. No map.
    for &filament in used {
        if values.0.get(filament).unwrap_or(&values.0[0]).0 == 0 {
            return Err(SliceError::InvalidInput(format!(
                "Plate {plate_number}: {bed} does not support filament {}\n",
                filament + 1,
            )));
        }
    }
    Ok(())
}
