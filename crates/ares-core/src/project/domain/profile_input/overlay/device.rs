use crate::{SliceError, project::raw_settings::ProjectSettingsRaw};

pub(super) use crate::profiles::is_filament_dev_key as is_key;

// CLI:3815-3834 cannot partition stored device vectors for partial loads.
pub(super) fn apply(
    result: &mut ProjectSettingsRaw,
    filaments: &[ProjectSettingsRaw],
    all_loaded: bool,
) -> Result<(), SliceError> {
    if all_loaded {
        crate::profiles::set_filament_dev_options(result, filaments.iter().map(Some), None)?;
    }
    Ok(())
}
