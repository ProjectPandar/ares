use crate::{SliceError, SliceOptions};

const SINGLE_EXTRUDER_MULTI_MATERIAL: &str = "single_extruder_multi_material";
const MANUAL_FILAMENT_CHANGE: &str = "manual_filament_change";
const SINGLE_EXTRUDER_MULTI_MATERIAL_PRIMING: &str =
    "single_extruder_multi_material_priming";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FilamentChangeOptions {
    single_extruder_multi_material: bool,
    manual_filament_change: bool,
    single_extruder_multi_material_priming: bool,
}

impl FilamentChangeOptions {
    pub(crate) const fn single_extruder_multi_material(&self) -> bool {
        self.single_extruder_multi_material
    }

    pub(crate) const fn manual_filament_change(&self) -> bool {
        self.manual_filament_change
    }

    pub(crate) const fn single_extruder_multi_material_priming(&self) -> bool {
        self.single_extruder_multi_material_priming
    }

    pub(crate) fn consume_runtime(self) {
        let _ = (
            self.single_extruder_multi_material(),
            self.manual_filament_change(),
            self.single_extruder_multi_material_priming(),
        );
    }
}

impl SliceOptions {
    pub(crate) fn filament_change_options(&self) -> Result<FilamentChangeOptions, SliceError> {
        Ok(FilamentChangeOptions {
            single_extruder_multi_material: self
                .bool_option(SINGLE_EXTRUDER_MULTI_MATERIAL, true)?,
            manual_filament_change: self.bool_option(MANUAL_FILAMENT_CHANGE, false)?,
            single_extruder_multi_material_priming: self
                .bool_option(SINGLE_EXTRUDER_MULTI_MATERIAL_PRIMING, false)?,
        })
    }
}
