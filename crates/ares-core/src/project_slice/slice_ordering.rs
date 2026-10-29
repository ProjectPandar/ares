use crate::geometry::{ClipperError, ExPolygon, chain_points, union_safety_offset_ex};

use super::region_slices::PostRegionPrintObject;

pub(super) fn order_expolygons(expolygons: Vec<ExPolygon>) -> Vec<ExPolygon> {
    let ordering_points = expolygons
        .iter()
        .map(|expolygon| {
            *expolygon
                .contour()
                .points()
                .first()
                .expect("a region surface ExPolygon contour must be nonempty")
        })
        .collect::<Vec<_>>();
    let order = chain_points(&ordering_points);
    let mut source = expolygons.into_iter().map(Some).collect::<Vec<_>>();
    order
        .into_iter()
        .map(|index| {
            source[index]
                .take()
                .expect("chain_points must return each source index once")
        })
        .collect()
}

/// `Layer.cpp:38-67::make_slices` chains a copy into physical-layer `lslices`.
/// Region-owned surface ordering remains unchanged for perimeter generation.
pub(super) fn make_slices(
    object: &mut PostRegionPrintObject,
) -> Result<Vec<Vec<ExPolygon>>, ClipperError> {
    Ok(match object.regions.as_mut_slice() {
        [] => (0..object.plan.layers.len()).map(|_| Vec::new()).collect(),
        [region] => region
            .layers
            .iter()
            .map(|layer| {
                order_expolygons(
                    layer
                        .surfaces
                        .iter()
                        .map(|surface| surface.as_parts().1.clone())
                        .collect(),
                )
            })
            .collect(),
        regions => {
            return (0..object.plan.layers.len())
                .map(|layer_index| {
                    let paths = regions
                        .iter()
                        .flat_map(|region| &region.layers[layer_index].surfaces)
                        .flat_map(|surface| {
                            let polygon = surface.as_parts().1;
                            std::iter::once(polygon.contour()).chain(polygon.holes())
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    union_safety_offset_ex(&paths).map(order_expolygons)
                })
                .collect();
        }
    })
}

const _: fn(Vec<ExPolygon>) -> Vec<ExPolygon> = order_expolygons;
type MakeSlices = fn(&mut PostRegionPrintObject) -> Result<Vec<Vec<ExPolygon>>, ClipperError>;
const _: MakeSlices = make_slices;
