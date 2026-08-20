use crate::{
    OrcaBool, ProcessInfillPattern,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        group_fills, prepare_infill::combine_infill, region_slices::RegionSurfaceKind,
    },
};

use super::{
    focused::fixture::{
        external, external_mut, graph, object_mut, options_mut, record_mut, surface,
    },
    oracle::authoritative_geometry,
};

const LAYER: usize = 1;

#[test]
fn task22o74_large_bed_scale_reaches_both_non_line_and_line_splits() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    external_mut(&mut graph).predecessor.predecessor.scale = CoordinateScale::LargeBed;
    record_mut(&mut graph, LAYER)
        .fill_no_overlap_expolygons
        .clear();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::InternalSolid,
        scaled_dog_bone(),
        0,
    )];

    options_mut(&mut graph, LAYER).internal_solid_infill_pattern = ProcessInfillPattern::Grid;
    let non_line = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(non_line.surface_fills.len(), 2);

    options_mut(&mut graph, LAYER).internal_solid_infill_pattern = ProcessInfillPattern::Monotonic;
    let line = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(line.surface_fills.len(), 2);
    assert_ne!(
        authoritative_geometry(&non_line),
        authoritative_geometry(&line)
    );
    combine_infill::dispose(graph);
}

fn scaled_dog_bone() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(400_000, 0),
            Point::new(400_000, 190_000),
            Point::new(600_000, 190_000),
            Point::new(600_000, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 400_000),
            Point::new(600_000, 400_000),
            Point::new(600_000, 210_000),
            Point::new(400_000, 210_000),
            Point::new(400_000, 400_000),
            Point::new(0, 400_000),
        ]),
        Vec::new(),
    )
}
