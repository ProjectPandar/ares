use super::{super::OrcaInt, RegionOptions};
use crate::options::{Percent, ProcessFuzzySkinType};

pub(super) fn normalize(options: &mut RegionOptions, num_extruders: usize) {
    clamp_feature(&mut options.sparse_infill_filament_id, num_extruders);
    clamp_feature(&mut options.internal_solid_filament_id, num_extruders);
    clamp_feature(&mut options.top_surface_filament_id, num_extruders);
    clamp_feature(&mut options.bottom_surface_filament_id, num_extruders);
    clamp_feature(&mut options.outer_wall_filament_id, num_extruders);
    clamp_feature(&mut options.inner_wall_filament_id, num_extruders);

    if options.sparse_infill_density.0 < f64::from(0.00011_f32) {
        options.sparse_infill_density = Percent(0.0);
    } else if options.sparse_infill_density.0 > 100.0 {
        options.sparse_infill_density = Percent(100.0);
    }

    if options.fuzzy_skin != ProcessFuzzySkinType::None
        && (options.fuzzy_skin_point_distance.0 < 0.01 || options.fuzzy_skin_thickness.0 < 0.001)
    {
        options.fuzzy_skin = ProcessFuzzySkinType::None;
    }
}

fn clamp_feature(value: &mut OrcaInt, num_extruders: usize) {
    if value.0 <= 0 || value.0 as usize > num_extruders {
        *value = OrcaInt(1);
    }
}
