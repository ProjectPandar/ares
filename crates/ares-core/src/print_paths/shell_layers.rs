use crate::Layer;

use super::PrintPathRole;

const SHELL_THICKNESS_EPSILON_MM: f64 = 1e-6;
const SHELL_THICKNESS_SCALE: f64 = 1_000_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellLayerOptions {
    bottom_shell_layers: usize,
    bottom_shell_thickness_microns: u64,
    top_shell_layers: usize,
    top_shell_thickness_microns: u64,
}

impl ShellLayerOptions {
    pub const fn new(bottom_shell_layers: usize, top_shell_layers: usize) -> Self {
        Self {
            bottom_shell_layers,
            bottom_shell_thickness_microns: 0,
            top_shell_layers,
            top_shell_thickness_microns: 0,
        }
    }

    pub(crate) fn with_thicknesses(
        bottom_shell_layers: usize,
        bottom_shell_thickness_mm: f64,
        top_shell_layers: usize,
        top_shell_thickness_mm: f64,
    ) -> Self {
        Self {
            bottom_shell_layers,
            bottom_shell_thickness_microns: thickness_microns(bottom_shell_thickness_mm),
            top_shell_layers,
            top_shell_thickness_microns: thickness_microns(top_shell_thickness_mm),
        }
    }

    pub const fn bottom_shell_layers(self) -> usize {
        self.bottom_shell_layers
    }

    pub const fn top_shell_layers(self) -> usize {
        self.top_shell_layers
    }

    pub const fn bottom_shell_thickness_mm(self) -> f64 {
        self.bottom_shell_thickness_microns as f64 / SHELL_THICKNESS_SCALE
    }

    pub const fn top_shell_thickness_mm(self) -> f64 {
        self.top_shell_thickness_microns as f64 / SHELL_THICKNESS_SCALE
    }

    pub(crate) fn is_bottom_shell(self, layers: &[Layer], layer_index: usize) -> bool {
        self.bottom_shell_layers > 0
            && (layer_index < self.bottom_shell_layers
                || self.bottom_shell_thickness_mm() > 0.0
                    && layer_bottom_z(&layers[layer_index]) - layer_bottom_z(&layers[0])
                        < self.bottom_shell_thickness_mm() - SHELL_THICKNESS_EPSILON_MM)
    }

    pub(crate) fn is_top_shell(self, layers: &[Layer], layer_index: usize) -> bool {
        self.top_shell_layers > 0
            && (layer_index >= layers.len().saturating_sub(self.top_shell_layers)
                || self.top_shell_thickness_mm() > 0.0
                    && layers[layers.len() - 1].print_z() - layers[layer_index].print_z()
                        < self.top_shell_thickness_mm() - SHELL_THICKNESS_EPSILON_MM)
    }

    pub(crate) fn solid_role(
        self,
        layers: &[Layer],
        layer_index: usize,
        unsupported_bridge: bool,
    ) -> PrintPathRole {
        if self.is_bottom_shell(layers, layer_index) {
            if unsupported_bridge {
                PrintPathRole::Bridge
            } else {
                PrintPathRole::BottomSurface
            }
        } else if self.is_top_shell(layers, layer_index) {
            PrintPathRole::TopSolidInfill
        } else {
            PrintPathRole::SolidInfill
        }
    }
}

impl Default for ShellLayerOptions {
    fn default() -> Self {
        Self::new(3, 4)
    }
}

pub(super) fn solid_print_path_role(
    layer_index: usize,
    layer_count: usize,
    shell_layers: ShellLayerOptions,
    unsupported_bridge: bool,
) -> PrintPathRole {
    if layer_index < shell_layers.bottom_shell_layers() {
        if unsupported_bridge {
            PrintPathRole::Bridge
        } else {
            PrintPathRole::BottomSurface
        }
    } else if shell_layers.top_shell_layers() > 0
        && layer_index >= layer_count.saturating_sub(shell_layers.top_shell_layers())
    {
        PrintPathRole::TopSolidInfill
    } else {
        PrintPathRole::SolidInfill
    }
}

fn thickness_microns(value: f64) -> u64 {
    (value * SHELL_THICKNESS_SCALE).round() as u64
}

fn layer_bottom_z(layer: &Layer) -> f64 {
    layer.print_z() - layer.height()
}
