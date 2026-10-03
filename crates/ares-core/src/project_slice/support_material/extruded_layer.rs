//! `SupportCommon.cpp:755-830` ordinary `SupportGeneratorLayerExtruded`.

use super::grid_generation::GeneratedSupportGeometry;
use crate::geometry::{ClipperError, Polygon, union_safety_offset_polygons};

#[derive(Clone, Copy)]
pub(super) enum SourceLayer {
    Bottom(usize),
    Top(usize),
    Base(usize),
    Interface(usize),
    BaseInterface(usize),
}

impl SourceLayer {
    pub(super) fn canonical(self, geometry: &GeneratedSupportGeometry) -> Self {
        match self {
            Self::Interface(index) => Self::Interface(geometry.ownership.interface(index, false)),
            Self::BaseInterface(index) => {
                Self::BaseInterface(geometry.ownership.interface(index, true))
            }
            Self::Bottom(_) | Self::Top(_) | Self::Base(_) => self,
        }
    }

    pub(super) fn print_z(self, geometry: &GeneratedSupportGeometry) -> f64 {
        match self.canonical(geometry) {
            Self::Bottom(index) => geometry.bottoms[index].print_z,
            Self::Top(index) => geometry.tops[index].print_z,
            Self::Base(index) => geometry.bases[index].print_z,
            Self::Interface(index) => geometry.interfaces[index].print_z,
            Self::BaseInterface(index) => geometry.base_interfaces[index].print_z,
        }
    }

    pub(super) fn bottom_z(self, geometry: &GeneratedSupportGeometry) -> f64 {
        match self.canonical(geometry) {
            Self::Bottom(index) => geometry.bottoms[index].bottom_z,
            Self::Top(index) => geometry.tops[index].bottom_z,
            Self::Base(index) => geometry.bases[index].bottom_z,
            Self::Interface(index) => geometry.interfaces[index].bottom_z,
            Self::BaseInterface(index) => geometry.base_interfaces[index].bottom_z,
        }
    }

    pub(super) fn polygons(self, geometry: &GeneratedSupportGeometry) -> &[Polygon] {
        match self.canonical(geometry) {
            Self::Bottom(index) => &geometry.bottoms[index].polygons,
            Self::Top(index) => &geometry.tops[index].polygons,
            Self::Base(index) => &geometry.bases[index].polygons,
            Self::Interface(index) => &geometry.interfaces[index].polygons,
            Self::BaseInterface(index) => &geometry.base_interfaces[index].polygons,
        }
    }

    fn polygons_mut(self, geometry: &mut GeneratedSupportGeometry) -> &mut Vec<Polygon> {
        match self.canonical(geometry) {
            Self::Bottom(index) => &mut geometry.bottoms[index].polygons,
            Self::Top(index) => &mut geometry.tops[index].polygons,
            Self::Base(index) => &mut geometry.bases[index].polygons,
            Self::Interface(index) => &mut geometry.interfaces[index].polygons,
            Self::BaseInterface(index) => &mut geometry.base_interfaces[index].polygons,
        }
    }

    pub(super) fn height_and_bridging(self, geometry: &GeneratedSupportGeometry) -> (f64, bool) {
        match self.canonical(geometry) {
            Self::Bottom(index) => (
                geometry.bottoms[index].height,
                geometry.bottoms[index].bridging,
            ),
            Self::Top(index) => (geometry.tops[index].height, false),
            Self::Base(index) => (geometry.bases[index].height, false),
            Self::Interface(index) => (geometry.interfaces[index].height, false),
            Self::BaseInterface(index) => (geometry.base_interfaces[index].height, false),
        }
    }
}

// A missing cache entry is Option<ExtrudedLayer>; populated entries always have a source.
pub(super) struct ExtrudedLayer<E> {
    pub(super) source: SourceLayer,
    pub(super) extrusions: Vec<E>,
    remaining: Option<Vec<Polygon>>,
}

impl<E> ExtrudedLayer<E> {
    pub(super) fn new(source: SourceLayer) -> Self {
        Self {
            source,
            extrusions: Vec::new(),
            remaining: None,
        }
    }

    pub(super) fn empty(&self, geometry: &GeneratedSupportGeometry) -> bool {
        self.source.polygons(geometry).is_empty()
    }

    pub(super) fn set_polygons_to_extrude(&mut self, polygons: Vec<Polygon>) {
        self.remaining = Some(polygons);
    }

    pub(super) fn polygons_to_extrude<'a>(
        &'a self,
        geometry: &'a GeneratedSupportGeometry,
    ) -> &'a [Polygon] {
        self.remaining
            .as_deref()
            .unwrap_or_else(|| self.source.polygons(geometry))
    }

    pub(super) fn could_merge(&self, other: &Self, geometry: &GeneratedSupportGeometry) -> bool {
        let (height, bridging) = self.source.height_and_bridging(geometry);
        let (other_height, other_bridging) = other.source.height_and_bridging(geometry);
        !self.empty(geometry)
            && !other.empty(geometry)
            && (height - other_height).abs() < 1e-4
            && bridging == other_bridging
    }

    pub(super) fn merge(
        &mut self,
        other: &mut Self,
        geometry: &mut GeneratedSupportGeometry,
    ) -> Result<(), ClipperError> {
        if let Some(mut polygons) = other.remaining.take() {
            let remaining = self
                .remaining
                .get_or_insert_with(|| self.source.polygons(geometry).to_vec());
            remaining.append(&mut polygons);
            *remaining = union_safety_offset_polygons(remaining)?;
        } else if let Some(remaining) = &mut self.remaining {
            remaining.extend_from_slice(other.source.polygons(geometry));
            *remaining = union_safety_offset_polygons(remaining)?;
        }
        self.extrusions.append(&mut other.extrusions);
        let mut polygons = std::mem::take(other.source.polygons_mut(geometry));
        let destination = self.source.polygons_mut(geometry);
        destination.append(&mut polygons);
        *destination = union_safety_offset_polygons(destination)?;
        Ok(())
    }

    pub(super) fn polygons_append(
        &self,
        geometry: &GeneratedSupportGeometry,
        destination: &mut Vec<Polygon>,
    ) {
        destination.extend_from_slice(self.source.polygons(geometry));
    }
}
