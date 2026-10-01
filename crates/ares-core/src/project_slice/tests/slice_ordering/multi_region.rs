use super::*;

#[test]
fn layer_make_slices_merges_regions_without_replacing_region_surfaces() {
    let left = rectangle(0, 0, 10_000_000, 10_000_000);
    let right = rectangle(10_000_000, 0, 20_000_000, 10_000_000);
    let mut input = object(vec![vec![left.clone()], Vec::new()]);
    input.regions.push(PostRegion {
        id: 7,
        options: region(),
        layers: vec![
            RegionLayer {
                surfaces: vec![RegionSurface::internal_with_metadata(
                    right.clone(),
                    2.5,
                    3,
                    0.75,
                    4,
                )],
            },
            RegionLayer {
                surfaces: Vec::new(),
            },
        ],
    });
    let before = surface_snapshot(&input);
    let output = make_slices(&mut input).unwrap();

    assert_eq!(output.len(), 2);
    assert_eq!(output[0].len(), 1);
    assert_eq!(output[0][0].area(), 20_000_020.0 * 10_000_020.0);
    assert!(output[0][0].holes().is_empty());
    assert!(output[1].is_empty());
    assert_eq!(surface_snapshot(&input), before);
    assert_eq!(input.regions[1].id, 7);
}

#[test]
fn layer_make_slices_keeps_holes_from_every_region() {
    let outer = rectangle(0, 0, 1_000_000, 1_000_000);
    let holed = ExPolygon::new(
        outer.contour().clone(),
        vec![Polygon::new(vec![
            Point::new(200_000, 200_000),
            Point::new(200_000, 800_000),
            Point::new(800_000, 800_000),
            Point::new(800_000, 200_000),
        ])],
    );
    let remote = rectangle(2_000_000, 0, 3_000_000, 1_000_000);
    let mut input = object(vec![vec![holed.clone()]]);
    input.regions.push(PostRegion {
        id: 1,
        options: region(),
        layers: vec![RegionLayer {
            surfaces: vec![RegionSurface::internal(remote.clone())],
        }],
    });
    let output = make_slices(&mut input).unwrap();

    assert_eq!(output[0].len(), 2);
    assert_eq!(
        output[0]
            .iter()
            .map(|polygon| polygon.holes().len())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        output[0].iter().map(ExPolygon::area).sum::<f64>(),
        2.0 * 1_000_020.0 * 1_000_020.0 - 599_980.0 * 599_980.0
    );
}
