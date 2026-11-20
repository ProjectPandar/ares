use super::{DEFAULT_TOOLPATH_HEIGHT, DEFAULT_TOOLPATH_WIDTH, MoveRecorder};
use crate::ExtrusionRole;
use crate::gcode_processor::EMoveType;

impl MoveRecorder<'_> {
    /// Width, height and `mm3_per_mm` of an extrusion (`:3880-3947`).
    pub(super) fn extrusion_geometry(
        &mut self,
        delta: [f64; 4],
        delta_xyz: f32,
        z: f64,
        linear: bool,
    ) -> f32 {
        let diameter = self
            .filament_diameters
            .get(self.filament)
            .or(self.filament_diameters.last())
            .copied()
            .unwrap_or(1.75);
        let radius = 0.5 * diameter;
        let area = std::f32::consts::PI * (radius * radius);
        let volume = (f64::from(area) * delta[3]) as f32;
        // UsedFilaments::increase_model_caches (GCodeProcessor.cpp:1534-1540).
        if !matches!(
            self.role,
            ExtrusionRole::SupportMaterial
                | ExtrusionRole::SupportMaterialInterface
                | ExtrusionRole::SupportTransition
                | ExtrusionRole::WipeTower
        ) {
            self.color_change_volume += f64::from(volume);
        }
        self.mm3_per_mm = volume / delta_xyz;
        if self.forced_height > 0.0 {
            self.height = self.forced_height;
        } else if linear && z > f64::from(self.extruded_last_z) + 1e-4 {
            self.height = (z - f64::from(self.extruded_last_z)) as f32;
        }
        if self.height == 0.0 {
            self.height = DEFAULT_TOOLPATH_HEIGHT;
        }
        if linear {
            self.extruded_last_z = z as f32;
        }
        let section = f64::from(delta_xyz * self.height);
        self.width = if self.forced_width > 0.0 {
            self.forced_width
        } else if self.role == ExtrusionRole::ExternalPerimeter {
            let factor =
                (std::f64::consts::PI * f64::from((1.05 * radius) * (1.05 * radius))) as f32;
            (delta[3] * f64::from(factor) / section) as f32
        } else if matches!(
            self.role,
            ExtrusionRole::BridgeInfill | ExtrusionRole::InternalBridgeInfill | ExtrusionRole::None
        ) {
            (f64::from(diameter) * (delta[3] / f64::from(delta_xyz)).sqrt()) as f32
        } else {
            let factor = (std::f64::consts::PI * f64::from(radius * radius)) as f32;
            let semicircles = (1.0 - 0.25 * std::f64::consts::PI) as f32 * self.height;
            (delta[3] * f64::from(factor) / section + f64::from(semicircles)) as f32
        };
        if self.width == 0.0 {
            self.width = DEFAULT_TOOLPATH_WIDTH;
        }
        self.width = self.width.min(2.0_f32.max(4.0 * self.height));
        volume
    }

    pub(super) fn add_mass(&mut self, start: [f64; 4], end: [f64; 4], volume: f32) {
        if let Some(locator) = self.mass_locator {
            let offset = self.extruder_offsets[self.filament];
            let midpoint = std::array::from_fn(|axis| {
                0.5 * (start[axis] + end[axis])
                    + f64::from([offset[0], offset[1], -self.z_offset][axis])
            });
            let density = crate::gcode_processor::density(
                self.filament_densities
                    .iter()
                    .map(|density| density.0 as f32),
                self.filament,
            );
            locator.add(
                &mut self.masses,
                self.role,
                midpoint,
                (f64::from(volume), density),
            );
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn travel_distance(&self) -> f32 {
        self.total_travel_distance
    }

    /// UsedFilaments::process_color_change_cache (GCodeProcessor.cpp:1549-1555).
    pub(super) fn flush_color_change_volume(&mut self) {
        if self.color_change_volume != 0.0 {
            self.color_change_volumes.push(self.color_change_volume);
            self.color_change_volume = 0.0;
        }
    }
}

/// `process_G1::move_type` (`:3835-3851`).
pub(super) fn move_type(wiping: bool, delta: [f64; 4]) -> EMoveType {
    let [x, y, z, e] = delta;
    if wiping {
        EMoveType::Wipe
    } else if e < 0.0 {
        if x != 0.0 || y != 0.0 || z != 0.0 {
            EMoveType::Travel
        } else {
            EMoveType::Retract
        }
    } else if e > 0.0 {
        if x == 0.0 && y == 0.0 {
            if z == 0.0 {
                EMoveType::Unretract
            } else {
                EMoveType::Travel
            }
        } else {
            EMoveType::Extrude
        }
    } else if x != 0.0 || y != 0.0 || z != 0.0 {
        EMoveType::Travel
    } else {
        EMoveType::Noop
    }
}
