//! `Print.hpp::WipeTowerData` and the `Print.cpp:5509-5597` Type2 generation lifecycle.
use super::super::tool_ordering::LayerTools;
use super::{generator::Generator, planning::Planned, result::ToolChangeResult};
use crate::{
    ProjectSettings,
    geometry::{BoundingBox, ClipperError, CoordinateScale, Polyline},
};

pub(in crate::project_slice::gcode_emit) struct Data {
    pub(super) type1: bool,
    pub(super) rib_offset: [f32; 2],
    pub(super) priming: Vec<ToolChangeResult>,
    pub(super) changes: Vec<Vec<ToolChangeResult>>,
    pub(super) final_purge: Option<ToolChangeResult>,
    pub(super) width: f32,
    pub(super) depth: f32,
    pub(super) height: f32,
    pub(super) brim_width: f32,
    pub(super) rib_width: f32,
    pub(super) rib_length: f32,
    pub(super) depth_profile: Vec<(f32, f32)>,
    pub(super) first_layer_bounds: Option<BoundingBox>,
    pub(super) outer_walls: Vec<(f32, Vec<Polyline>)>,
    pub(super) used_filament: Vec<f32>,
    pub(super) tool_changes: usize,
}

impl Planned {
    pub(in crate::project_slice::gcode_emit) fn generate(
        self,
        settings: &ProjectSettings,
        scale: CoordinateScale,
        (tools, last): (&[usize], &LayerTools),
        normal_height: f64,
    ) -> Result<Data, ClipperError> {
        let mut generator = Generator::new(
            settings,
            self.filaments,
            (scale, settings.printer.gcode.gcode_flavor),
            self.first_tool,
            self.matrix,
        );
        // Prime even when front-edge output is disabled: native generation
        // retains the resulting tool, temperature and wiping state.
        let priming = generator.prime(
            settings.process.print.initial_layer_print_height.0 as f32,
            tools,
        );
        generator.plan = self.plan;
        let changes = generator.generate()?;
        // `Print.cpp:5575-5594`: a tower below the last print layer gets no final purge.
        let final_purge = (last.wipe_tower_partitions > 0).then(|| {
            if generator.layer_finished {
                generator.set_layer((last.print_z + normal_height) as f32, normal_height as f32);
            }
            generator.tool_change(None)
        });
        // `WipeTower2::get_z_and_depth_pairs`, cpp 2448–2459.
        let mut depth_profile = vec![(0.0, generator.plan.depth)];
        for layer in &generator.plan.layers {
            if layer.depth < depth_profile.last().unwrap().1 - 1e-3 {
                depth_profile.push((layer.z, layer.depth));
            }
        }
        if depth_profile.last().unwrap().0 < generator.plan.height - 1e-3 {
            depth_profile.push((generator.plan.height, 0.0));
        }
        Ok(Data {
            type1: false,
            rib_offset: generator.rib_offset,
            priming,
            changes,
            final_purge,
            width: generator.plan.width,
            depth: generator.plan.depth,
            height: generator.plan.height,
            brim_width: generator.brim_width,
            rib_width: generator.rib_width,
            rib_length: generator.rib_length,
            depth_profile,
            first_layer_bounds: generator.first_layer_bounds,
            outer_walls: Vec::new(),
            used_filament: generator.used_filament,
            tool_changes: generator.tool_changes,
        })
    }
}
