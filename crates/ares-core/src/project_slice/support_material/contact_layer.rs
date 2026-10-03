//! Top-contact subset of `Support/SupportLayer.hpp::SupportGeneratorLayer`.

use crate::geometry::{ClipperError, Polygon, union_polygons_paths};

#[derive(Default)]
pub(super) struct TopContactLayer {
    pub(super) print_z: f64,
    pub(super) bottom_z: f64,
    pub(super) height: f64,
    pub(super) polygons: Vec<Polygon>,
    pub(super) contact_polygons: Option<Vec<Polygon>>,
    pub(super) overhang_polygons: Option<Vec<Polygon>>,
    pub(super) enforcer_polygons: Option<Vec<Polygon>>,
}

impl TopContactLayer {
    pub(super) fn merge(&mut self, mut source: Self) -> Result<(), ClipperError> {
        self.polygons.append(&mut source.polygons);
        self.polygons = union_polygons_paths(&self.polygons)?;
        merge_polygons(&mut self.contact_polygons, source.contact_polygons)?;
        merge_polygons(&mut self.overhang_polygons, source.overhang_polygons)?;
        merge_polygons(&mut self.enforcer_polygons, source.enforcer_polygons)?;
        Ok(())
    }
}

fn merge_polygons(
    destination: &mut Option<Vec<Polygon>>,
    source: Option<Vec<Polygon>>,
) -> Result<(), ClipperError> {
    match destination {
        Some(polygons) if !polygons.is_empty() => {
            if let Some(mut source) = source
                && !source.is_empty()
            {
                polygons.append(&mut source);
                *polygons = union_polygons_paths(polygons)?;
            }
        }
        _ => *destination = source,
    }
    Ok(())
}

/// `SupportMaterial.cpp::merge_contact_layers`: conservative lower-Z merge.
pub(super) fn merge_contact_layers(
    mut layers: Vec<TopContactLayer>,
    first_print_layer_height: f64,
    support_layer_height_min: f64,
) -> Result<Vec<TopContactLayer>, ClipperError> {
    const EPSILON: f64 = 1e-4;
    layers.sort_unstable_by(|left, right| left.print_z.total_cmp(&right.print_z));
    let mut layers = layers.into_iter().peekable();
    let mut merged = Vec::new();
    let first_limit = first_print_layer_height + support_layer_height_min - EPSILON;
    if layers
        .peek()
        .is_some_and(|layer| layer.print_z < first_limit)
    {
        let mut first = layers.next().unwrap();
        while layers
            .peek()
            .is_some_and(|layer| layer.print_z < first_limit)
        {
            first.merge(layers.next().unwrap())?;
        }
        first.print_z = first_print_layer_height;
        first.height = first_print_layer_height;
        first.bottom_z = 0.0;
        merged.push(first);
    }
    while let Some(mut layer) = layers.next() {
        let limit = layer.print_z + support_layer_height_min + EPSILON;
        while layers.peek().is_some_and(|next| next.print_z < limit) {
            layer.merge(layers.next().unwrap())?;
        }
        merged.push(layer);
    }
    Ok(merged)
}
