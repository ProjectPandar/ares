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

pub(super) fn make_single_region_slices(object: &PostRegionPrintObject) -> Vec<Vec<ExPolygon>> {
    match object.regions.as_slice() {
        [] => (0..object.plan.layers.len()).map(|_| Vec::new()).collect(),
        [region] => region
            .layers
            .iter()
            .map(|layer| {
                order_expolygons(
                    layer
                        .surfaces()
                        .iter()
                        .map(|surface| surface.as_parts().1.clone())
                        .collect(),
                )
            })
            .collect(),
        _ => unreachable!("Task 22M make_slices accepts at most one region"),
    }
}

const _: fn(Vec<ExPolygon>) -> Vec<ExPolygon> = order_expolygons;
const _: fn(&PostRegionPrintObject) -> Vec<Vec<ExPolygon>> = make_single_region_slices;
