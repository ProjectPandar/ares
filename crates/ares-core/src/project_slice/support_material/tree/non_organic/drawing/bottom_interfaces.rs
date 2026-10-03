//! True contact bands from `TreeSupport.cpp:2215-2332` and surface helper80-102.

use super::super::heights::LayerHeightData;
use super::areas::LayerAreas;
use crate::{
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Polygon, difference_ex,
        difference_ex_polygons, intersection_ex, intersection_ex_polygons, offset_expolygons,
        union_ex,
    },
    project_slice::layers::PlannedLayer,
};

pub(super) struct BottomInterfaces<'a> {
    pub(super) object_layers: &'a [PlannedLayer],
    pub(super) tops: &'a [Vec<Polygon>],
    pub(super) bottoms: &'a [Vec<Polygon>],
    pub(super) heights: &'a [LayerHeightData],
    pub(super) gap: f64,
    pub(super) nominal_height: f64,
    pub(super) interface_layers: usize,
    pub(super) base_interface_layers: usize,
    pub(super) support_width: f64,
    pub(super) scale: CoordinateScale,
}

struct ContactBand {
    z: f64,
    surfaces: Vec<Polygon>,
}

struct ComponentAreas<'a> {
    base: Vec<ExPolygon>,
    floor: &'a mut Vec<ExPolygon>,
    interface_id_set: bool,
    floor_as_base: bool,
}

impl BottomInterfaces<'_> {
    pub(super) fn apply(
        &self,
        areas: &mut LayerAreas,
        layer: usize,
        object_layer: usize,
    ) -> Result<(), ClipperError> {
        let mut bases = Vec::new();
        let mut floors = Vec::new();
        let mut floor_as_base = areas.floor_interface_as_base;
        for component in &areas.base {
            let bands = self.contact_bands(component, object_layer)?;
            let mut current = ComponentAreas {
                base: vec![component.clone()],
                floor: &mut floors,
                interface_id_set: false,
                floor_as_base,
            };
            for band in bands {
                self.apply_band(&mut current, areas, layer, &band)?;
            }
            bases.extend(current.base);
            floor_as_base = current.floor_as_base;
        }
        areas.base = bases;
        areas.floor = floors;
        areas.floor_interface_as_base = floor_as_base;
        Ok(())
    }

    fn apply_band(
        &self,
        current: &mut ComponentAreas<'_>,
        areas: &LayerAreas,
        layer: usize,
        band: &ContactBand,
    ) -> Result<(), ClipperError> {
        let band_applies = areas.print_z >= band.z - 0.0001;
        if !band_applies {
            return Ok(());
        }
        let start = band.z + self.gap;
        let bottom_z = areas.print_z - areas.height;
        if self.gap > 0.0001 && bottom_z < start - 0.0001 {
            current.base = difference_ex_polygons(&current.base, &band.surfaces)?;
        }
        let interface_height = self.interface_layers as f64 * self.nominal_height;
        if !(self.interface_layers > 0
            && bottom_z >= start - 0.0001
            && bottom_z < start + interface_height - 0.0001)
        {
            return Ok(());
        }
        if !current.interface_id_set {
            self.assign_base_interface(&mut current.floor_as_base, layer, start);
            current.interface_id_set = true;
        }
        let interface = self.interface_component(&current.base, &band.surfaces)?;
        if !interface.is_empty() {
            let separated = offset_expolygons(&interface, 10.0, JoinType::Miter, 3.0)?;
            current.floor.extend(interface);
            current.base = difference_ex(&current.base, &separated)?;
        }
        Ok(())
    }

    fn contact_bands(
        &self,
        component: &ExPolygon,
        object_layer: usize,
    ) -> Result<Vec<ContactBand>, ClipperError> {
        let mut bands = Vec::new();
        for layer in (0..=object_layer).rev() {
            // Source collects every region's stTop first, then every stBottom.
            let mut surfaces =
                Vec::with_capacity(self.tops[layer].len() + self.bottoms[layer].len());
            surfaces.extend_from_slice(&self.tops[layer]);
            surfaces.extend_from_slice(&self.bottoms[layer]);
            if surfaces.is_empty() {
                continue;
            }
            if !intersection_ex_polygons(std::slice::from_ref(component), &surfaces)?.is_empty() {
                bands.push(ContactBand {
                    z: self.object_layers[layer].print_z,
                    surfaces,
                });
            }
        }
        bands.sort_unstable_by(|first, second| first.z.total_cmp(&second.z));
        Ok(bands)
    }

    fn assign_base_interface(&self, floor_as_base: &mut bool, layer: usize, start: f64) {
        let mut first = layer;
        while first > 0 {
            if self.heights[first - 1].print_z <= start + 0.0001 {
                break;
            }
            first -= 1;
        }
        if self.base_interface_layers > 0 {
            let index = layer - first;
            let base_start = self
                .interface_layers
                .saturating_sub(self.base_interface_layers);
            *floor_as_base = index >= base_start;
        }
    }

    fn interface_component(
        &self,
        component: &[ExPolygon],
        surfaces: &[Polygon],
    ) -> Result<Vec<ExPolygon>, ClipperError> {
        let mut band = union_ex(surfaces, FillRule::NonZero)?;
        if !band.is_empty() {
            let margin = offset_expolygons(
                component,
                self.scale.scaled_delta(self.support_width) as f32,
                JoinType::Miter,
                3.0,
            )?;
            band = intersection_ex(&band, &margin)?;
        }
        if band.is_empty() {
            Ok(Vec::new())
        } else {
            intersection_ex(component, &band)
        }
    }
}
