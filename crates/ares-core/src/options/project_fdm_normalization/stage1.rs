use super::super::{Nullable, OrcaBool, OrcaFloat, OrcaInt, Percent};
use super::ProjectSettings;

pub(super) fn normalize(settings: &mut ProjectSettings) {
    {
        let region = &mut settings.process.region;
        if region.sparse_infill_filament_id.0 > 0 && region.internal_solid_filament_id.0 == 0 {
            region.internal_solid_filament_id = region.sparse_infill_filament_id;
        }

        let internal_solid = region.internal_solid_filament_id;
        let top_surface = region.top_surface_filament_id;
        let bottom_surface = region.bottom_surface_filament_id;

        if internal_solid.0 == 0 && top_surface.0 > 0 {
            region.internal_solid_filament_id = top_surface;
        }
        if internal_solid.0 == 0 && bottom_surface.0 > 0 {
            region.internal_solid_filament_id = bottom_surface;
        }
        if top_surface.0 == 0 && internal_solid.0 > 0 {
            region.top_surface_filament_id = internal_solid;
        }
        if bottom_surface.0 == 0 && internal_solid.0 > 0 {
            region.bottom_surface_filament_id = internal_solid;
        }
    }

    if settings.process.print.spiral_mode.0 {
        for value in &mut settings.project.print.retract_when_changing_layer.0 {
            *value = OrcaBool(false);
        }
        for value in &mut settings
            .filament
            .retract_overrides
            .filament_retract_when_changing_layer
        {
            *value = Nullable::Value(OrcaBool(false));
        }

        settings.process.region.wall_loops = OrcaInt(1);
        settings.process.region.alternate_extra_wall = OrcaBool(false);
        settings.process.region.top_shell_layers = OrcaInt(0);
        settings.process.region.sparse_infill_density = Percent(0.0);
    }

    settings.process.print.resolution = OrcaFloat(settings.process.print.resolution.0.max(0.001));
}
