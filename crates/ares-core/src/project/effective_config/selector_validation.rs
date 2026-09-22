use crate::{OrcaInt, ProjectSettings, SliceError};

use super::{Project, ValidatedMaterializedProject, invalid_option};

// `PrintConfig.cpp:10391-10394`: firmware retraction is incompatible with
// wipe (G-code wipe interleaves with G10/G11 retracts).
const FIRMWARE_RETRACTION_WIPE_MESSAGE: &str =
    "--use-firmware-retraction is not compatible with --wipe";

// `Print.cpp:1699-1708`: the `G92 E0` matchers — `regex_g92e0` is any-case,
// `regex_g92e0_correct` is the exact-uppercase form relative addressing needs.
// The `(?m)` flag makes `^`/`$` per-line anchors — upstream `boost::regex_search`
// matches `G92 E0` on any line of the multi-line gcode template.
const G92E0_ANY: &str = r"(?m)^[ \t]*[gG]92[ \t]*[eE](0(\.0*)?|\.0+)[ \t]*(;.*)?$";
const G92E0_EXACT: &str = r"(?m)^[ \t]*G92[ \t]*E(0(\.0*)?|\.0+)[ \t]*(;.*)?$";

pub(super) fn validate(
    settings: &ProjectSettings,
    project: &Project,
    validated: ValidatedMaterializedProject,
) -> Result<(), SliceError> {
    validate_wipe_tower(settings, validated)?;
    validate_firmware_retraction(settings)?;
    validate_extruder_addressing(settings)?;
    validate_support_selector("support_filament", settings.process.object.support_filament)?;
    validate_support_selector(
        "support_interface_filament",
        settings.process.object.support_interface_filament,
    )?;

    for object in project.objects() {
        if let Some(selector) = object.object_overrides().support_filament {
            validate_support_selector("support_filament", selector)?;
        }
        if let Some(selector) = object.object_overrides().support_interface_filament {
            validate_support_selector("support_interface_filament", selector)?;
        }

        validate_raw_extruder(
            object.region_overrides().extruder,
            validated.logical_filament_count,
        )?;
        for volume in object.volumes() {
            validate_raw_extruder(
                volume.region_overrides().extruder,
                validated.logical_filament_count,
            )?;
        }
        for layer_range in object.layer_config_ranges() {
            validate_raw_extruder(
                layer_range.region_overrides().extruder,
                validated.logical_filament_count,
            )?;
        }
    }
    Ok(())
}

fn validate_wipe_tower(
    settings: &ProjectSettings,
    validated: ValidatedMaterializedProject,
) -> Result<(), SliceError> {
    let selector = settings.process.print.wipe_tower_filament.0;
    if selector == 0 {
        return Ok(());
    }
    if selector < 0 {
        return Err(invalid_option("wipe_tower_filament"));
    }

    // Upstream only consumes the selector when a wipe tower is actually
    // emitted and more than one extruder is in play
    // (`Print.cpp:540`: `has_wipe_tower() && wipe_tower_filament != 0 &&
    // extruders.size() > 1`); on a single-filament no-tower print the value
    // is ignored entirely, so an out-of-range index is not an error there.
    if !super::usage::has_wipe_tower(settings, validated.logical_filament_count)
        || validated.logical_filament_count <= 1
    {
        return Ok(());
    }

    let selector = selector as usize;
    if selector >= validated.physical_extruder_count || selector > validated.logical_filament_count
    {
        return Err(invalid_option("wipe_tower_filament"));
    }
    Ok(())
}

fn validate_support_selector(key: &str, selector: OrcaInt) -> Result<(), SliceError> {
    if selector.0 < 0 {
        return Err(invalid_option(key));
    }
    Ok(())
}

fn validate_raw_extruder(
    extruder: Option<OrcaInt>,
    logical_filament_count: usize,
) -> Result<(), SliceError> {
    if let Some(extruder) = extruder
        && (extruder.0 < 0 || extruder.0 as usize > logical_filament_count)
    {
        return Err(invalid_option("extruder"));
    }
    Ok(())
}

fn validate_firmware_retraction(settings: &ProjectSettings) -> Result<(), SliceError> {
    if settings.printer.gcode.use_firmware_retraction.0
        && settings.project.print.wipe.0.iter().any(|wipe| wipe.0)
    {
        return Err(SliceError::InvalidInput(format!(
            "use_firmware_retraction: {FIRMWARE_RETRACTION_WIPE_MESSAGE}"
        )));
    }
    Ok(())
}

// `Print.cpp:1694-1745`: `G92 E0` handling depends on `use_relative_e_distances`.
fn validate_extruder_addressing(settings: &ProjectSettings) -> Result<(), SliceError> {
    let gcode = &settings.printer.gcode;
    let before = &gcode.before_layer_change_gcode.0;
    let layer = &gcode.layer_change_gcode.0;
    let any = regex::Regex::new(G92E0_ANY).unwrap();
    if gcode.use_relative_e_distances.0 {
        let exact = regex::Regex::new(G92E0_EXACT).unwrap();
        let before_exact = exact.is_match(before);
        let layer_exact = exact.is_match(layer);
        if any.is_match(before) && !before_exact {
            return Err(SliceError::InvalidInput(
                "before_layer_change_gcode: \"G92 E0\" was found in before_layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase \"G92 E0\"."
                    .to_owned(),
            ));
        }
        if any.is_match(layer) && !layer_exact {
            return Err(SliceError::InvalidInput(
                "layer_change_gcode: \"G92 E0\" was found in layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase \"G92 E0\"."
                    .to_owned(),
            ));
        }
    } else {
        if any.is_match(before) {
            return Err(SliceError::InvalidInput(
                "before_layer_change_gcode: \"G92 E0\" was found in before_layer_change_gcode, which is incompatible with absolute extruder addressing."
                    .to_owned(),
            ));
        }
        if any.is_match(layer) {
            return Err(SliceError::InvalidInput(
                "layer_change_gcode: \"G92 E0\" was found in layer_change_gcode, which is incompatible with absolute extruder addressing."
                    .to_owned(),
            ));
        }
    }
    Ok(())
}
