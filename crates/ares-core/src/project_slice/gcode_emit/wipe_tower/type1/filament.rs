//! `WipeTower.cpp:1541–1610`: Type1 filament state (no Type2 ramming setup).
use super::super::filament::at;
use crate::ProjectSettings;

pub(super) struct Filament {
    pub(super) material: String,
    pub(super) category: i32,
    pub(super) soluble: bool,
    pub(super) temperature: i32,
    pub(super) first_temperature: i32,
    pub(super) interface_temperature: i32,
    pub(super) interface_distance: f32,
    pub(super) interface_length: f32,
    pub(super) ironing_area: f32,
    pub(super) interface_purge: f32,
    pub(super) area: f32,
    pub(super) max_e_speed: f32,
    pub(super) retract_length: f32,
    pub(super) retract_speed: f32,
    pub(super) wipe_distance: f32,
}

impl Filament {
    pub(super) fn new(
        settings: &ProjectSettings,
        index: usize,
        (retract_length, retract_speed, wipe_distance): (f32, f32, f32),
    ) -> Self {
        let g = &settings.filament.gcode;
        let p = &settings.filament.print;
        let chosen = settings.process.print.wipe_tower_filament.0;
        let diameter = at(&g.filament_diameter.0, index).0;
        let area = (std::f64::consts::FRAC_PI_4 * diameter.powi(2)) as f32;
        let first_diameter = g.filament_diameter.0[0].0;
        let first_area = (std::f64::consts::FRAC_PI_4 * first_diameter.powi(2)) as f32;
        let maximum = at(&g.filament_max_volumetric_speed.0, index).0 as f32;
        let interface = at(&g.filament_tower_interface_print_temp.0, index).0;
        Self {
            material: at(&g.filament_type.0, index).clone(),
            category: at(&g.filament_adhesiveness_category.0, index).0,
            soluble: if chosen == 0 {
                at(&g.filament_soluble.0, index).0
            } else {
                index != (chosen - 1) as usize
            },
            temperature: at(&p.nozzle_temperature.0, index).0,
            first_temperature: at(&p.nozzle_temperature_initial_layer.0, index).0,
            interface_temperature: if interface == -1 {
                at(&p.nozzle_temperature_range_high.0, index).0
            } else {
                interface
            },
            interface_distance: at(&g.filament_tower_interface_pre_extrusion_dist.0, index).0
                as f32,
            interface_length: at(&g.filament_tower_interface_pre_extrusion_length.0, index).0
                as f32,
            ironing_area: at(&g.filament_tower_ironing_area.0, index).0 as f32,
            interface_purge: at(&g.filament_tower_interface_purge_volume.0, index).0 as f32,
            area,
            max_e_speed: if maximum == 0.0 {
                f32::MAX
            } else {
                maximum / first_area
            },
            retract_length,
            retract_speed,
            wipe_distance,
        }
    }
}
