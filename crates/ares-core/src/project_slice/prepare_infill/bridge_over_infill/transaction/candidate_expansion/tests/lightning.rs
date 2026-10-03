use super::*;
use crate::{
    FloatOrPercent, ProcessInfillPattern,
    geometry::{ExPolygon, Point, Polygon, Polyline},
    project_slice::region_slices::RegionSurface,
};

#[test]
fn empty_later_lightning_regions_width_does_not_change_generated_tree_lines() {
    let baseline = generated_lines(0.42, 15.0, 0).unwrap();
    assert!(!baseline.is_empty());
    assert_eq!(generated_lines(0.84, 15.0, 0).unwrap(), baseline);
}

#[test]
fn zero_density_later_lightning_region_does_not_activate_generator() {
    assert!(generated_lines(0.42, 0.0, 0).is_none());
}

#[test]
fn moving_internal_geometry_to_later_region_preserves_generated_tree_lines() {
    let baseline = generated_lines(0.42, 15.0, 0).unwrap();
    assert!(!baseline.is_empty());
    assert_eq!(generated_lines(0.42, 15.0, 1).unwrap(), baseline);
}

fn generated_lines(
    later_width: f64,
    later_density: f64,
    geometry_region: usize,
) -> Option<Vec<Polyline>> {
    let mut raw = prepare(KsrArchive::new());
    let horizontal = &mut raw.predecessor.predecessor;
    let prelude = &mut horizontal.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let layer_count = prelude.object.object.as_parts().0.plan.layers.len();
    let regions = &mut prelude.object.object.as_parts_mut().0.regions;
    let mut options = regions[0].options.clone();
    options.sparse_infill_pattern = ProcessInfillPattern::Lightning;
    options.sparse_infill_line_width = FloatOrPercent::Float(later_width);
    options.sparse_infill_density.0 = later_density;
    regions.push(PostRegion {
        id: regions[0].id + 1,
        options,
        layers: (0..layer_count)
            .map(|_| RegionLayer {
                surfaces: Vec::new(),
            })
            .collect(),
    });
    prelude.object.records.resize_with(2 * layer_count, || None);
    horizontal.objects[0]
        .records
        .resize_with(2 * layer_count, || None);
    horizontal.objects[0]
        .records
        .iter_mut()
        .flatten()
        .for_each(|record| record.fill_surfaces.clear());
    let square = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(20_000_000, 0),
        Point::new(20_000_000, 20_000_000),
        Point::new(0, 20_000_000),
    ]);
    for layer in 0..3 {
        horizontal.objects[0].records[layer]
            .as_mut()
            .unwrap()
            .fill_surfaces = vec![RegionSurface::internal(ExPolygon::new(
            square.clone(),
            Vec::new(),
        ))];
        if geometry_region == 1 {
            let mut input = prelude.object.records[layer].take().unwrap();
            input.current.region_index = 1;
            input.region_id = regions[1].id;
            prelude.object.records[layer_count + layer] = Some(input);
            horizontal.objects[0].records[layer_count + layer] =
                horizontal.objects[0].records[layer].take();
        }
    }
    raw.objects[0].has_lightning_infill = true;
    raw.objects[0].surfaces_by_layer.clear();

    super::super::prepare(&raw.predecessor, &mut raw.objects).unwrap();

    let lines = raw.objects[0]
        .lightning_generator
        .as_ref()
        .map(|generator| generator.convert_to_lines(2, &[square], 0).unwrap());
    bridge_over_infill::dispose(raw);
    lines
}
