use crate::{SliceError, SliceOptions};

pub(crate) fn consume(options: &SliceOptions) -> Result<(), SliceError> {
    options.preheat_options()?.consume_runtime();
    options.timelapse_type_options()?.consume_runtime();
    options.change_filament_gcode()?;
    options.filament_change_options()?.consume_runtime();
    Ok(())
}
