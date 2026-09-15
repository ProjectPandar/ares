use crate::geometry::{ExPolygon, chain_points};

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

/// `Layer::make_slices` (`Layer.cpp:40-67`): chain the region surfaces by
/// their first contour points into lslices, and — critically — set the
/// region's own slices to the SAME order (`Layer.cpp:93`
/// `m_regions.front()->slices.set(this->lslices, stInternal)`). The
/// perimeter generator's BBS surface reorder
/// (`PerimeterGenerator.cpp:1214-1224` `chain_expolygons` over bbox
/// centers) then chains from the lslices order, and its lowest-index tie
/// breaks make the input order observable in the collection emission
/// order.
pub(super) fn make_single_region_slices(object: &mut PostRegionPrintObject) -> Vec<Vec<ExPolygon>> {
    match object.regions.as_mut_slice() {
        [] => (0..object.plan.layers.len()).map(|_| Vec::new()).collect(),
        [region] => region
            .layers
            .iter_mut()
            .map(|layer| {
                let mut surfaces = std::mem::take(&mut layer.surfaces);
                let ordering_points = surfaces
                    .iter()
                    .map(|surface| {
                        *surface
                            .as_parts()
                            .1
                            .contour()
                            .points()
                            .first()
                            .expect("a region surface ExPolygon contour must be nonempty")
                    })
                    .collect::<Vec<_>>();
                let order = chain_points(&ordering_points);
                let mut source = surfaces.into_iter().map(Some).collect::<Vec<_>>();
                let ordered = order
                    .into_iter()
                    .map(|index| {
                        source[index]
                            .take()
                            .expect("chain_points must return each source index once")
                    })
                    .collect::<Vec<_>>();
                let lslices = ordered
                    .iter()
                    .map(|surface| surface.as_parts().1.clone())
                    .collect();
                layer.surfaces = ordered;
                lslices
            })
            .collect(),
        _ => unreachable!("Task 22M make_slices accepts at most one region"),
    }
}

const _: fn(Vec<ExPolygon>) -> Vec<ExPolygon> = order_expolygons;
const _: fn(&mut PostRegionPrintObject) -> Vec<Vec<ExPolygon>> = make_single_region_slices;
