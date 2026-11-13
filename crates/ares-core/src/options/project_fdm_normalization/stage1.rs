use super::super::{Nullable, OrcaBool, OrcaFloat, OrcaInt, Percent};
use super::ProjectSettings;

pub(super) fn normalize(settings: &mut ProjectSettings) {
    if settings.process.print.spiral_mode.0 {
        settings
            .project
            .print
            .retract_when_changing_layer
            .0
            .fill(OrcaBool(false));
        settings
            .filament
            .retract_overrides
            .filament_retract_when_changing_layer
            .fill(Nullable::Value(OrcaBool(false)));

        settings.process.region.wall_loops = OrcaInt(1);
        settings.process.region.alternate_extra_wall = OrcaBool(false);
        settings.process.region.top_shell_layers = OrcaInt(0);
        settings.process.region.sparse_infill_density = Percent(0.0);
    }

    settings.process.print.resolution = OrcaFloat(settings.process.print.resolution.0.max(0.001));
}
