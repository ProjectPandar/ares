//! `TreeSupport.cpp:2444-2507` enabled global Lightning support geometry.

use std::collections::HashMap;

use super::super::contacts::ContactNodes;
use super::areas::LayerAreas;
use crate::{
    fill::lightning::generator::{Generator, SupportSettings},
    geometry::{ClipperError, JoinType, difference_ex, offset_expolygons},
    project_slice::{perimeters::types::Flow, support_material::layer_input::to_polygons},
};

pub(in super::super) struct SupportLightning {
    pub(in super::super) generator: Generator,
    print_z_to_layer: HashMap<u64, usize>,
}

impl SupportLightning {
    pub(in super::super) fn layer_index(&self, print_z: f64) -> Option<usize> {
        self.print_z_to_layer.get(&z_key(print_z)).copied()
    }
}

pub(super) struct LightningInput<'a> {
    pub(super) contacts: &'a ContactNodes,
    pub(super) support_width: f64,
    pub(super) nominal_flow: &'a Flow,
    pub(super) base_pattern_spacing: f64,
    pub(super) settings: &'a SupportSettings,
}

impl LightningInput<'_> {
    pub(super) fn generate(&self, layers: &[LayerAreas]) -> Result<SupportLightning, ClipperError> {
        let mut contours = Vec::new();
        let mut overhangs = Vec::new();
        let mut print_z_to_layer = HashMap::new();
        for (index, layer) in layers.iter().enumerate().skip(1) {
            if self.contacts.layers[index].is_empty()
                || layer.height < 0.0001
                || layer.groups.is_empty()
            {
                continue;
            }
            let Some(lower) = layers[..index]
                .iter()
                .rposition(|area| !area.groups.is_empty())
            else {
                continue;
            };
            if lower == 0 {
                continue;
            }
            let lower = &layers[lower];
            let inset = offset_expolygons(
                &lower.base,
                (-2.0 * self.settings.scale.scaled_delta(self.support_width)) as f32,
                JoinType::Miter,
                3.0,
            )?;
            overhangs.push(to_polygons(&difference_ex(&inset, &layer.base)?));
            contours.push(to_polygons(&lower.base));
            print_z_to_layer.insert(z_key(lower.print_z), overhangs.len() - 1);
        }
        let spacing = f64::from(self.nominal_flow.spacing);
        let density = (spacing / (self.base_pattern_spacing + spacing) * 2.0).min(1.0);
        Ok(SupportLightning {
            generator: Generator::for_support(&contours, overhangs, density as f32, self.settings)?,
            print_z_to_layer,
        })
    }
}

fn z_key(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}
