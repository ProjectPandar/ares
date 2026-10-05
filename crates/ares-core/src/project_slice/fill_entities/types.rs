/// Coordinate adapter for `Fill::extended_object_bounding_box`
/// (`FillBase.cpp:1569-1578`) and its X bounds in `FillRectilinear.cpp:2785`.
pub(super) struct ObjectFillGeometry {
    pub(super) scale: crate::geometry::CoordinateScale,
    pub(super) extended_object_min_x: i64,
    pub(super) extended_object_max_x: i64,
}

use crate::{
    ExtrusionRole,
    geometry::Polyline,
    project_slice::perimeters::classic::{
        entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
        materialize::FittedMove,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct FillExtrusionPath {
    pub(in crate::project_slice) polyline: Polyline,
    pub(in crate::project_slice) fitting: Vec<FittedMove>,
    pub(in crate::project_slice) role: ExtrusionRole,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) enum FillExtrusionEntity {
    Path(FillExtrusionPath),
    /// `ExtrusionPathOriented` (`ExtrusionEntity.hpp:364-372`): never reversed.
    Oriented(FillExtrusionPath),
    VariableWidth(GapFillEntity),
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct FillExtrusionCollection {
    pub(in crate::project_slice) entities: Vec<FillExtrusionEntity>,
    pub(in crate::project_slice) no_sort: bool,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct LayerFillEntities {
    pub(in crate::project_slice) perimeters: Vec<ExtrusionEntityCollection>,
    pub(in crate::project_slice) perimeter_ranges: Vec<std::ops::Range<usize>>,
    pub(in crate::project_slice) collections: Vec<FillExtrusionCollection>,
    pub(in crate::project_slice) collection_ranges: Vec<(usize, std::ops::Range<usize>)>,
    pub(in crate::project_slice) thin_fills: Vec<GapFillEntity>,
    pub(in crate::project_slice) thin_fill_ranges: Vec<std::ops::Range<usize>>,
}

impl LayerFillEntities {
    pub(in crate::project_slice) fn region_collections(
        &self,
        region_index: usize,
    ) -> impl Iterator<Item = &FillExtrusionCollection> {
        self.collection_ranges
            .iter()
            .filter(move |(region, _)| *region == region_index)
            .flat_map(|(_, range)| &self.collections[range.clone()])
    }

    pub(in crate::project_slice) fn region_perimeters(
        &self,
        region_index: usize,
    ) -> &[ExtrusionEntityCollection] {
        &self.perimeters[self.perimeter_ranges[region_index].clone()]
    }
}
