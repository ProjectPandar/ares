#[derive(Clone, Debug, PartialEq)]
pub struct HardwareOptions {
    pub(super) nozzle_diameters: Vec<f64>,
    pub(super) filament_diameters: Vec<f64>,
    pub(super) min_layer_heights: Vec<f64>,
    pub(super) max_layer_heights: Vec<f64>,
}

impl HardwareOptions {
    pub fn nozzle_diameters(&self) -> &[f64] {
        &self.nozzle_diameters
    }

    pub fn filament_diameters(&self) -> &[f64] {
        &self.filament_diameters
    }

    pub fn min_layer_heights(&self) -> &[f64] {
        &self.min_layer_heights
    }

    pub fn max_layer_heights(&self) -> &[f64] {
        &self.max_layer_heights
    }
}
