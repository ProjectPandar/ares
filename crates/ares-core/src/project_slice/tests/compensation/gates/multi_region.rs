use super::*;

#[test]
fn multi_region_elephant_foot_keeps_the_shared_region_boundary() {
    let mut input = post_object(0, 1, 2, None);
    for (index, region) in input.regions.iter_mut().enumerate() {
        let min_x = index as i64 * 10_000_000;
        region.layers[0].surfaces = vec![RegionSurface::internal(expolygon(&[
            (min_x, 0),
            (min_x + 10_000_000, 0),
            (min_x + 10_000_000, 12_000_000),
            (min_x, 12_000_000),
        ]))];
    }
    let output = apply(
        vec![input],
        vec![resolved(0, task22m_options(0.15, 1, 0))],
        &[0.4],
    )
    .unwrap();
    let (regions, backup) = output[0].as_parts();

    assert_eq!(backup[0].len(), 1);
    assert_eq!(backup[0][0].area(), 240_000_000_000_000.0);
    assert_eq!(regions.regions.len(), 2);
    for (index, region) in regions.regions.iter().enumerate() {
        assert_eq!(region.id, index);
        assert_eq!(region.layers[0].surfaces.len(), 1);
        let polygon = region.layers[0].surfaces[0].as_parts().1;
        assert_eq!(polygon.area(), 115_245_000_000_000.0);
        let points = polygon.contour().points();
        let min_x = points.iter().map(|point| point.x()).min().unwrap();
        let max_x = points.iter().map(|point| point.x()).max().unwrap();
        assert_eq!(
            (min_x, max_x),
            if index == 0 {
                (150_000, 10_000_000)
            } else {
                (10_000_000, 19_850_000)
            }
        );
    }
}

#[test]
fn positive_growth_assigns_overlap_to_the_first_region() {
    let mut input = post_object(0, 1, 2, None);
    for (index, region) in input.regions.iter_mut().enumerate() {
        let min_x = index as i64 * 10_000_000;
        region.layers[0].surfaces = vec![RegionSurface::internal(expolygon(&[
            (min_x, 0),
            (min_x + 10_000_000, 0),
            (min_x + 10_000_000, 12_000_000),
            (min_x, 12_000_000),
        ]))];
    }
    let mut options = task22m_options(0.0, 1, 0);
    options.xy_contour_compensation = OrcaFloat(0.2);
    let output = apply(vec![input], vec![resolved(0, options)], &[0.4]).unwrap();
    let (regions, _) = output[0].as_parts();
    for (index, region) in regions.regions.iter().enumerate() {
        assert_eq!(region.layers[0].surfaces.len(), 1);
        let polygon = region.layers[0].surfaces[0].as_parts().1;
        assert_eq!(
            polygon.area(),
            [128_960_000_000_000.0, 124_000_000_000_000.0][index]
        );
        let points = polygon.contour().points();
        assert_eq!(
            (
                points.iter().map(|p| p.x()).min().unwrap(),
                points.iter().map(|p| p.x()).max().unwrap()
            ),
            [(-200_000, 10_200_000), (10_200_000, 20_200_000)][index]
        );
    }
}

#[test]
fn negative_growth_excludes_regions_with_no_extrusion() {
    let mut input = post_object(0, 1, 2, None);
    for (index, region) in input.regions.iter_mut().enumerate() {
        let min_x = index as i64 * 10_000_000;
        region.layers[0].surfaces = vec![RegionSurface::internal(expolygon(&[
            (min_x, 0),
            (min_x + 10_000_000, 0),
            (min_x + 10_000_000, 12_000_000),
            (min_x, 12_000_000),
        ]))];
    }
    let subtractor = &mut input.regions[1].options;
    subtractor.bottom_shell_layers = OrcaInt(0);
    subtractor.top_shell_layers = OrcaInt(0);
    subtractor.wall_loops = OrcaInt(0);
    subtractor.sparse_infill_density = Percent(0.0);
    let mut options = task22m_options(0.0, 1, 0);
    options.xy_contour_compensation = OrcaFloat(-0.2);
    let output = apply(vec![input], vec![resolved(0, options)], &[0.4]).unwrap();
    let (regions, _) = output[0].as_parts();
    assert!(regions.regions[1].layers[0].surfaces.is_empty());
    assert_eq!(regions.regions[0].layers[0].surfaces.len(), 1);
    let points = regions.regions[0].layers[0].surfaces[0]
        .as_parts()
        .1
        .contour()
        .points();
    assert_eq!(
        (
            points.iter().map(|p| p.x()).min().unwrap(),
            points.iter().map(|p| p.x()).max().unwrap()
        ),
        (199_900, 9_800_100)
    );
}

#[test]
fn negative_growth_initializes_closing_radius_before_later_objects() {
    let first = post_object(0, 1, 2, None);
    let mut first_options = task22m_options(0.0, 1, 0);
    first_options.xy_contour_compensation = OrcaFloat(-0.1);
    first_options.slice_closing_radius = OrcaFloat(0.0);
    let mut second = post_object(1, 1, 2, None);
    for (index, region) in second.regions.iter_mut().enumerate() {
        let min_x = index as i64 * 10_050_000;
        region.layers[0].surfaces = vec![RegionSurface::internal(expolygon(&[
            (min_x, 0),
            (min_x + 10_000_000, 0),
            (min_x + 10_000_000, 12_000_000),
            (min_x, 12_000_000),
        ]))];
    }
    let output = apply(
        vec![first, second],
        vec![
            resolved(0, first_options),
            resolved(1, task22m_options(0.15, 1, 0)),
        ],
        &[0.4],
    )
    .unwrap();
    let (_, backup) = output[1].as_parts();
    // Zero closing radius retains the 0.05 mm gap instead of closing it.
    assert_eq!(backup[0].len(), 2);
    assert_eq!(backup[0][0].area(), 120_000_000_000_000.0);
    assert_eq!(backup[0][1].area(), 120_000_000_000_000.0);
}

#[test]
fn mixed_growth_clips_to_regions_without_filling_existing_holes() {
    let mut input = post_object(0, 1, 2, None);
    for (index, region) in input.regions.iter_mut().enumerate() {
        let min_x = index as i64 * 10_000_000;
        let holes = if index == 0 {
            vec![Polygon::new(vec![
                Point::new(2_000_000, 4_000_000),
                Point::new(2_000_000, 6_000_000),
                Point::new(4_000_000, 6_000_000),
                Point::new(4_000_000, 4_000_000),
            ])]
        } else {
            Vec::new()
        };
        let contour = expolygon(&[
            (min_x, 0),
            (min_x + 10_000_000, 0),
            (min_x + 10_000_000, 12_000_000),
            (min_x, 12_000_000),
        ])
        .into_parts()
        .0;
        region.layers[0].surfaces = vec![RegionSurface::internal(ExPolygon::new(contour, holes))];
    }
    let mut options = task22m_options(0.0, 1, 0);
    options.xy_contour_compensation = OrcaFloat(0.2);
    options.xy_hole_compensation = OrcaFloat(-0.1);
    let output = apply(vec![input], vec![resolved(0, options)], &[0.4]).unwrap();
    let (regions, _) = output[0].as_parts();
    assert_eq!(regions.regions[0].layers[0].surfaces.len(), 1);
    assert_eq!(regions.regions[1].layers[0].surfaces.len(), 1);
    let left = regions.regions[0].layers[0].surfaces[0].as_parts().1;
    let right = regions.regions[1].layers[0].surfaces[0].as_parts().1;
    assert_eq!(left.area(), 124_960_799_960_000.0);
    assert_eq!(right.area(), 124_000_000_000_000.0);
    assert_eq!(left.holes().len(), 1);
    let hole = left.holes()[0].points();
    // CW offset shrinks the trimming hole; intersection cannot fill the
    // larger hole retained from the first merged-outline stage (100 units).
    assert_eq!(hole.iter().map(|p| p.x()).min().unwrap(), 2_000_100);
    assert_eq!(hole.iter().map(|p| p.x()).max().unwrap(), 3_999_900);
    assert_eq!(hole.iter().map(|p| p.y()).min().unwrap(), 4_000_100);
    assert_eq!(hole.iter().map(|p| p.y()).max().unwrap(), 5_999_900);
    assert_eq!(
        left.contour().points().iter().map(|p| p.x()).max().unwrap(),
        10_200_000
    );
    assert_eq!(
        right
            .contour()
            .points()
            .iter()
            .map(|p| p.x())
            .min()
            .unwrap(),
        10_200_000
    );
}
