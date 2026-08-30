use crate::{
    FloatOrPercent,
    gcode_spiral_vase::{
        ProjectSpiralVaseConfig, ProjectSpiralVaseLayer, ProjectSpiralVaseRunState,
    },
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

const SOURCE_EPSILON_MM: f64 = 1e-4;

pub(super) struct SpiralVaseFilter {
    run: ProjectSpiralVaseRunState,
    enabled: bool,
    bottom_shell_layers: usize,
    bottom_shell_thickness: f64,
    skirt_height: usize,
    has_brim: bool,
    infinite_skirt: bool,
    travel_feedrate: f64,
}

impl SpiralVaseFilter {
    pub(super) fn from_traversal(traversal: &PreparedPostClassicTraversal, has_brim: bool) -> Self {
        let full = &traversal.resolved.views.full;
        let print = &full.process.print;
        let region = &full.process.region;
        let nozzle = full
            .project
            .print
            .nozzle_diameter
            .0
            .first()
            .map_or(0.4, |value| value.0);
        let max_xy_smoothing = match print.spiral_mode_max_xy_smoothing {
            FloatOrPercent::Float(value) => value,
            FloatOrPercent::Percent(value) => nozzle * value.0 * 0.01,
        };
        let enabled = print.spiral_mode.0;
        Self {
            run: ProjectSpiralVaseRunState::new(ProjectSpiralVaseConfig {
                enabled,
                smooth_xy: enabled && print.spiral_mode_smooth.0,
                max_xy_smoothing,
                starting_flow_ratio: print.spiral_starting_flow_ratio.0,
                finishing_flow_ratio: print.spiral_finishing_flow_ratio.0,
                resolution: print.resolution.0,
                relative_e: traversal
                    .resolved
                    .views
                    .runtime_gcode
                    .use_relative_e_distances
                    .0,
            }),
            enabled,
            bottom_shell_layers: usize::try_from(region.bottom_shell_layers.0)
                .expect("normalized bottom_shell_layers is non-negative"),
            bottom_shell_thickness: region.bottom_shell_thickness.0,
            skirt_height: usize::try_from(print.skirt_height.0)
                .expect("normalized skirt_height is non-negative"),
            has_brim,
            infinite_skirt: print.draft_shield != crate::ProcessDraftShield::Disabled,
            travel_feedrate: traversal.resolved.views.runtime_gcode.travel_speed.0 * 60.0,
        }
    }

    /// `GCode.cpp:4596-4612` enables the vase filter only after bottom/skirt
    /// layers and only for a layer containing one perimeter and no fill.
    pub(super) fn is_body_layer(
        &self,
        layer: &OrderedExtrusionLayer,
        layer_index: usize,
        layer_z: f64,
    ) -> bool {
        let mut perimeter_count = 0;
        let mut has_fill = false;
        for entity in layer.islands.iter().flat_map(|island| &island.entities) {
            match entity {
                IslandPrintEntity::Perimeter(collection) => {
                    perimeter_count += collection.entities.len();
                }
                IslandPrintEntity::Fill(_)
                | IslandPrintEntity::FillCollection(_)
                | IslandPrintEntity::Thin(_) => has_fill = true,
            }
        }
        self.enabled
            && (layer_index > 0 || !self.has_brim)
            && !self.infinite_skirt
            && layer_index >= self.skirt_height
            && layer_index >= self.bottom_shell_layers
            && layer_z >= self.bottom_shell_thickness - SOURCE_EPSILON_MM
            && perimeter_count <= 1
            && !has_fill
    }

    pub(super) fn append_layer_z(&self, output: &mut Vec<u8>, layer_index: usize, layer_z: f64) {
        if self.enabled && layer_index > 0 {
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}\n",
                    super::format_processor_float(layer_z),
                    super::format_processor_float(self.travel_feedrate)
                )
                .as_bytes(),
            );
        }
    }

    pub(super) fn process_layer(&mut self, output: &mut Vec<u8>, layer: ProjectSpiralVaseLayer) {
        self.run.process_layer(output, layer);
    }
}

pub(super) type Layer = ProjectSpiralVaseLayer;
