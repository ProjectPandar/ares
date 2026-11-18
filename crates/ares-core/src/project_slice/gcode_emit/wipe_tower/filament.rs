//! `WipeTower2::set_extruder` and `FilamentParameters` (cpp 1346–1411).

use crate::ProjectSettings;

pub(super) fn at<T>(values: &[T], index: usize) -> &T {
    values.get(index).unwrap_or(&values[0])
}

pub(super) struct Filament {
    pub(super) material: String,
    pub(super) soluble: bool,
    pub(super) support: bool,
    pub(super) temperature: i32,
    pub(super) first_temperature: i32,
    pub(super) interface_temperature: i32,
    pub(super) minimal_purge: f32,
    pub(super) loading_speed: f32,
    pub(super) loading_start: f32,
    pub(super) unloading_speed: f32,
    pub(super) unloading_start: f32,
    pub(super) delay: f32,
    pub(super) cooling_moves: i32,
    pub(super) cooling_initial: f32,
    pub(super) cooling_final: f32,
    pub(super) stamping_speed: f32,
    pub(super) stamping_distance: f32,
    pub(super) area: f32,
    pub(super) nozzle: f32,
    pub(super) max_e_speed: f32,
    pub(super) ramming_width: f32,
    pub(super) ramming_step: f32,
    pub(super) ramming_speeds: Vec<f32>,
    pub(super) multitool_ramming: bool,
    pub(super) multitool_ramming_time: f32,
    pub(super) retract_length: f32,
    pub(super) retract_speed: f32,
    pub(super) interface_distance: f32,
    pub(super) interface_length: f32,
    pub(super) ironing_area: f32,
    pub(super) interface_purge: f32,
}

impl Filament {
    pub(super) fn from_config(
        settings: &ProjectSettings,
        index: usize,
        (retract_length, retract_speed): (f32, f32),
    ) -> Self {
        let g = &settings.filament.gcode;
        let p = &settings.filament.print;
        let semm = settings.printer.gcode.single_extruder_multi_material.0;
        let chosen = settings.process.print.wipe_tower_filament.0;
        let diameter = at(&g.filament_diameter.0, index).0;
        let first_diameter = g.filament_diameter.0[0].0;
        let first_area = (std::f64::consts::FRAC_PI_4 * first_diameter.powi(2)) as f32;
        let max_volume = at(&g.filament_max_volumetric_speed.0, index).0 as f32;
        let interface_temperature = at(&g.filament_tower_interface_print_temp.0, index).0;
        let mut filament = Self {
            material: at(&g.filament_type.0, index).clone(),
            soluble: if chosen > 0 {
                index != (chosen - 1) as usize
            } else {
                at(&g.filament_soluble.0, index).0
            },
            support: at(&g.filament_is_support.0, index).0,
            temperature: at(&p.nozzle_temperature.0, index).0,
            first_temperature: at(&p.nozzle_temperature_initial_layer.0, index).0,
            interface_temperature: if interface_temperature == -1 {
                at(&p.nozzle_temperature_range_high.0, index).0
            } else {
                interface_temperature
            },
            minimal_purge: at(&g.filament_minimal_purge_on_wipe_tower.0, index).0 as f32,
            loading_speed: 0.0,
            loading_start: 0.0,
            unloading_speed: 0.0,
            unloading_start: 0.0,
            delay: 0.0,
            cooling_moves: 0,
            cooling_initial: 0.0,
            cooling_final: 0.0,
            stamping_speed: 0.0,
            stamping_distance: 0.0,
            area: (std::f64::consts::FRAC_PI_4 * diameter.powi(2)) as f32,
            nozzle: at(&settings.project.print.nozzle_diameter.0, index).0 as f32,
            max_e_speed: if max_volume == 0.0 {
                f32::MAX
            } else {
                max_volume / first_area
            },
            ramming_width: 1.0,
            ramming_step: 1.0,
            ramming_speeds: Vec::new(),
            multitool_ramming: false,
            multitool_ramming_time: 0.0,
            retract_length,
            retract_speed,
            interface_distance: at(&g.filament_tower_interface_pre_extrusion_dist.0, index).0
                as f32,
            interface_length: at(&g.filament_tower_interface_pre_extrusion_length.0, index).0
                as f32,
            ironing_area: at(&g.filament_tower_ironing_area.0, index).0 as f32,
            interface_purge: at(&g.filament_tower_interface_purge_volume.0, index).0 as f32,
        };
        if semm {
            filament.loading_speed = at(&g.filament_loading_speed.0, index).0 as f32;
            filament.loading_start = at(&g.filament_loading_speed_start.0, index).0 as f32;
            filament.unloading_speed = at(&g.filament_unloading_speed.0, index).0 as f32;
            filament.unloading_start = at(&g.filament_unloading_speed_start.0, index).0 as f32;
            filament.delay = at(&g.filament_toolchange_delay.0, index).0 as f32;
            filament.cooling_moves = at(&g.filament_cooling_moves.0, index).0;
            filament.cooling_initial = at(&g.filament_cooling_initial_speed.0, index).0 as f32;
            filament.cooling_final = at(&g.filament_cooling_final_speed.0, index).0 as f32;
            filament.stamping_speed = at(&g.filament_stamping_loading_speed.0, index).0 as f32;
            filament.stamping_distance = at(&g.filament_stamping_distance.0, index).0 as f32;
            let mut numbers = at(&g.filament_ramming_parameters.0, index)
                .split('|')
                .next()
                .unwrap()
                .split_whitespace()
                .map_while(|word| word.parse::<f32>().ok());
            filament.ramming_width = numbers.next().unwrap_or(1.0) / 100.0;
            filament.ramming_step = numbers.next().unwrap_or(1.0) / 100.0;
            filament.ramming_speeds.extend(numbers);
        } else {
            let volume = at(&g.filament_multitool_ramming_volume.0, index).0 as f32;
            let flow = at(&g.filament_multitool_ramming_flow.0, index).0 as f32;
            filament.multitool_ramming =
                at(&g.filament_multitool_ramming.0, index).0 && volume > 0.0 && flow > 0.0;
            filament.ramming_width = 2.0;
            filament.ramming_speeds.push(flow);
            filament.multitool_ramming_time = if flow > 0.0 { volume / flow } else { 0.0 };
        }
        filament
    }
}
