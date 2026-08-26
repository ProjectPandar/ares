use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloat, OrcaFloats, OrcaInt, ProjectSettings, RegionOptions,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{
            PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface, RegionSurfaceKind,
            VolumeSlices,
        },
        slice_ordering::{make_single_region_slices, order_expolygons},
    },
};

use crate::project_slice::compensation::{PostCompensationPrintObject, apply_project_compensation};

use super::{
    region_slices::complex::synthetic_outputs,
    support::{region, resolved},
};

#[test]
fn task22m_slice_ordering_uses_contour_first_points_and_preserves_holes() {
    let mut input = markers(&[(10, 0), (0, 0), (20, 0), (0, 10), (10, 10)]);
    input[3] = marker_with_rotated_contour_and_hole(0, 10);
    let expected = [
        input[1].clone(),
        input[3].clone(),
        input[4].clone(),
        input[0].clone(),
        input[2].clone(),
    ];

    assert_eq!(order_expolygons(input), expected);
    assert_eq!(expected[1].holes().len(), 1);
}

#[test]
fn task22m_slice_ordering_keeps_exact_layer_count_for_empty_and_multiple_layers() {
    let input = markers(&[(10, 0), (0, 0), (20, 0), (0, 10), (10, 10)]);
    let object = object(vec![Vec::new(), input.clone()]);
    let output = make_single_region_slices(&object);

    assert_eq!(output.len(), 2);
    assert!(output[0].is_empty());
    assert_eq!(
        output[1],
        [
            input[1].clone(),
            input[3].clone(),
            input[4].clone(),
            input[0].clone(),
            input[2].clone(),
        ]
    );

    let empty = object_with_regions(2, Vec::new());
    assert_eq!(make_single_region_slices(&empty), [Vec::new(), Vec::new()]);
}

#[test]
fn task22m_slice_ordering_chains_uncompensated_backup_independently() {
    let backup = markers(&[(10, 0), (0, 0), (20, 0), (0, 10), (10, 10)]);
    let current = markers(&[(0, 0), (100, 0), (10, 0), (20, 0)]);

    let ordered_current = order_expolygons(current.clone());
    let ordered_backup = order_expolygons(backup.clone());

    assert_eq!(
        first_points(&ordered_current),
        [(100, 0), (20, 0), (10, 0), (0, 0)]
    );
    assert_eq!(
        first_points(&ordered_backup),
        [(0, 0), (0, 10), (10, 10), (10, 0), (20, 0)]
    );
    assert_eq!(first_points(&current), [(0, 0), (100, 0), (10, 0), (20, 0)]);
}

#[test]
#[rustfmt::skip]
fn task22m_apply_wraps_exact_one_two_layer_ramps_and_short_object_clamp() {
    let raw = rectangle(0, 0, 20_000_000, 12_000_000);
    let objects = vec![
        seeded_object(11, 0, 3, vec![vec![raw.clone()]]),
        seeded_object(5, 0, 7, vec![vec![raw.clone()]; 3]),
        seeded_object(20, 0, 9, vec![vec![raw.clone()]; 2]),
    ];
    let plans = objects.iter().map(|object| object.plan.clone()).collect::<Vec<_>>();
    let region_options = objects.iter().map(|object| object.regions[0].options.clone()).collect::<Vec<_>>();
    let resolved = [
        resolved(11, compensation_options(0.15, 1, 0), vec![region()]),
        resolved(5, compensation_options(0.2, 2, 0), vec![region()]),
        resolved(20, compensation_options(0.2, 5, 0), vec![region()]),
    ];
    let output = apply_project_compensation(
        objects, &resolved, FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
        crate::geometry::CoordinateScale::Normal,
    ).unwrap();

    assert_eq!(output.len(), 3);
    assert_compensated_object(&output[0], (&plans[0], 3, &region_options[0]), &raw, &[Some(150_000)]);
    assert_compensated_object(&output[1], (&plans[1], 7, &region_options[1]), &raw, &[Some(200_000), Some(100_000), None]);
    assert_compensated_object(&output[2], (&plans[2], 9, &region_options[2]), &raw, &[Some(200_000), Some(160_000)]);
}

#[test]
#[rustfmt::skip]
fn task22m_apply_signed_raft_and_disabled_objects_preserve_complete_current_state() {
    let mut available = synthetic_outputs();
    let mut objects = vec![available.remove(8), available.remove(0), available.remove(6)];
    for object in &mut objects {
        seed_first_surface(object);
    }
    let plans = objects.iter().map(|object| object.plan.clone()).collect::<Vec<_>>();
    let sidecars = objects.iter().map(sidecar_snapshot).collect::<Vec<_>>();
    let regions = objects.iter().map(region_snapshot).collect::<Vec<_>>();
    let surfaces = objects.iter().map(surface_snapshot).collect::<Vec<_>>();
    let lslices = objects.iter().map(make_single_region_slices).collect::<Vec<_>>();
    let resolved = [
        resolved(8, compensation_options(0.15, 1, -1), vec![region()]),
        resolved(0, compensation_options(0.0, 1, 0), vec![region()]),
        resolved(7, compensation_options(0.15, 1, 1), vec![region()]),
    ];
    let output = apply_project_compensation(
        objects, &resolved, FloatOrPercent::Float(0.5), &OrcaFloats(Vec::new()),
        crate::geometry::CoordinateScale::Normal,
    ).unwrap();

    assert_eq!(output.len(), 3);
    for (index, wrapper) in output.iter().enumerate() {
        let (post_regions, actual_lslices) = wrapper.as_parts();
        assert_eq!(post_regions.as_parts().0, &plans[index]);
        assert_eq!(sidecar_snapshot(post_regions), sidecars[index]);
        assert_eq!(region_snapshot(post_regions), regions[index]);
        assert_eq!(surface_snapshot(post_regions), surfaces[index]);
        assert_eq!(actual_lslices, lslices[index]);
    }
}

#[test]
#[rustfmt::skip]
fn task22m_apply_zero_region_object_has_one_empty_slice_vector_per_planned_layer() {
    let object = object_with_context(33, 0, 3, Vec::new());
    let expected_plan = object.plan.clone();
    let mut output = apply_project_compensation(
        vec![object], &[resolved(33, compensation_options(0.15, 1, 0), Vec::new())],
        FloatOrPercent::Float(0.5), &OrcaFloats(Vec::new()),
        crate::geometry::CoordinateScale::Normal,
    ).unwrap();

    let (post_regions, lslices) = output.pop().unwrap().into_parts();
    assert_eq!(post_regions.plan, expected_plan);
    assert!(post_regions.volume_slices.is_empty());
    assert!(post_regions.regions.is_empty());
    assert_eq!(lslices, [Vec::new(), Vec::new(), Vec::new()]);
}

fn object(layers: Vec<Vec<ExPolygon>>) -> PostRegionPrintObject {
    let layer_count = layers.len();
    let region_layers = layers
        .into_iter()
        .map(|expolygons| RegionLayer {
            surfaces: expolygons
                .into_iter()
                .map(RegionSurface::internal)
                .collect(),
        })
        .collect();
    object_with_regions(
        layer_count,
        vec![PostRegion {
            id: 0,
            options: region(),
            layers: region_layers,
        }],
    )
}

fn object_with_regions(layer_count: usize, regions: Vec<PostRegion>) -> PostRegionPrintObject {
    object_with_context(0, 0, layer_count, regions)
}

fn object_with_context(
    source_object_index: usize,
    transform_index: usize,
    layer_count: usize,
    regions: Vec<PostRegion>,
) -> PostRegionPrintObject {
    PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index,
            transform_index,
            layers: (0..layer_count)
                .map(|id| PlannedLayer {
                    id,
                    height: 0.2,
                    print_z: (id + 1) as f64 * 0.2,
                    slice_z: id as f64 * 0.2 + 0.1,
                })
                .collect(),
        },
        volume_slices: Vec::<VolumeSlices>::new(),
        regions,
    }
}

#[rustfmt::skip]
fn seeded_object(
    source_object_index: usize,
    transform_index: usize,
    region_id: usize,
    layers: Vec<Vec<ExPolygon>>,
) -> PostRegionPrintObject {
    let layer_count = layers.len();
    let layers = layers.into_iter().map(|expolygons| RegionLayer {
        surfaces: expolygons.into_iter()
            .map(|expolygon| RegionSurface::internal_with_metadata(expolygon, 2.5, 3, 0.75, 4))
            .collect(),
    }).collect();
    object_with_context(
        source_object_index, transform_index, layer_count,
        vec![PostRegion { id: region_id, options: region(), layers }],
    )
}

fn compensation_options(compensation: f64, layers: i32, raft_layers: i32) -> ObjectOptions {
    let mut options = ObjectOptions::from_base(&ProjectSettings::default().process.object);
    options.elefant_foot_compensation = OrcaFloat(compensation);
    options.elefant_foot_compensation_layers = OrcaInt(layers);
    options.raft_layers = OrcaInt(raft_layers);
    options.line_width = FloatOrPercent::Float(0.42);
    options
}

fn assert_compensated_object(
    wrapper: &PostCompensationPrintObject,
    expected: (&PlannedPrintObject, usize, &RegionOptions),
    raw: &ExPolygon,
    insets: &[Option<i64>],
) {
    let (expected_plan, region_id, expected_options) = expected;
    let (post_regions, lslices) = wrapper.as_parts();
    let (plan, sidecars, regions) = post_regions.as_parts();
    assert_eq!(plan, expected_plan);
    assert!(sidecars.is_empty());
    assert_eq!(regions.len(), 1);
    let (actual_id, options, layers) = regions[0].as_parts();
    assert_eq!(actual_id, region_id);
    assert_eq!(options, expected_options);
    assert_eq!(layers.len(), insets.len());
    assert_eq!(lslices.len(), insets.len());

    for (index, inset) in insets.iter().enumerate() {
        let surfaces = layers[index].surfaces();
        assert_eq!(surfaces.len(), 1);
        let (kind, expolygon, thickness, thickness_layers, bridge_angle, extra_perimeters) =
            surfaces[0].as_parts();
        let expected = inset.map_or_else(|| raw.clone(), compensated_rectangle);
        assert_eq!(expolygon, &expected);
        let expected_metadata = if inset.is_some() {
            (RegionSurfaceKind::Internal, -1.0, 1, -1.0, 0)
        } else {
            (RegionSurfaceKind::Internal, 2.5, 3, 0.75, 4)
        };
        assert_eq!(
            (
                kind,
                thickness,
                thickness_layers,
                bridge_angle,
                extra_perimeters
            ),
            expected_metadata,
        );
        assert_eq!(lslices[index].as_slice(), std::slice::from_ref(raw));
    }
}

#[rustfmt::skip]
fn seed_first_surface(object: &mut PostRegionPrintObject) {
    let surface = object.regions.iter_mut().flat_map(|region| &mut region.layers)
        .flat_map(|layer| &mut layer.surfaces).next().unwrap();
    let expolygon = surface.as_parts().1.clone();
    *surface = RegionSurface::internal_with_metadata(expolygon, 2.5, 3, 0.75, 4);
}

#[rustfmt::skip]
fn sidecar_snapshot(object: &PostRegionPrintObject) -> Vec<(u32, Vec<Vec<ExPolygon>>)> {
    object.volume_slices.iter().map(|sidecar| {
        let (occurrence_id, layers) = sidecar.as_parts();
        (occurrence_id.get(), layers.to_vec())
    }).collect()
}

#[rustfmt::skip]
fn region_snapshot(object: &PostRegionPrintObject) -> Vec<(usize, RegionOptions)> {
    object.regions.iter().map(|region| (region.id, region.options.clone())).collect()
}

type SurfaceSnapshot = (RegionSurfaceKind, ExPolygon, u64, u16, u64, u16);

fn surface_snapshot(object: &PostRegionPrintObject) -> Vec<Vec<Vec<SurfaceSnapshot>>> {
    object
        .regions
        .iter()
        .map(|region| {
            region
                .layers
                .iter()
                .map(|layer| layer.surfaces.iter().map(surface_state).collect())
                .collect()
        })
        .collect()
}

#[rustfmt::skip]
fn surface_state(surface: &RegionSurface) -> SurfaceSnapshot {
    let (kind, expolygon, thickness, layers, bridge, extra) = surface.as_parts();
    (kind, expolygon.clone(), thickness.to_bits(), layers, bridge.to_bits(), extra)
}

fn markers(points: &[(i64, i64)]) -> Vec<ExPolygon> {
    points.iter().map(|&(x, y)| marker(x, y)).collect()
}

#[rustfmt::skip]
fn marker(x: i64, y: i64) -> ExPolygon {
    shape(&[(x, y), (x + 1, y), (x, y + 1)])
}

#[rustfmt::skip]
fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    shape(&[(min_x, min_y), (max_x, min_y), (max_x, max_y), (min_x, max_y)])
}

#[rustfmt::skip]
fn compensated_rectangle(inset: i64) -> ExPolygon {
    shape(&[
        (20_000_000 - inset, 12_000_000 - inset), (inset, 12_000_000 - inset),
        (inset, inset), (20_000_000 - inset, inset),
    ])
}

fn shape(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()),
        Vec::new(),
    )
}

fn marker_with_rotated_contour_and_hole(x: i64, y: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, y),
            Point::new(x - 100, y + 100),
            Point::new(x - 100, y - 100),
        ]),
        vec![Polygon::new(vec![
            Point::new(x - 90, y - 10),
            Point::new(x - 90, y - 9),
            Point::new(x - 89, y - 10),
        ])],
    )
}

fn first_points(expolygons: &[ExPolygon]) -> Vec<(i64, i64)> {
    expolygons
        .iter()
        .map(|expolygon| {
            let point = expolygon.contour().points()[0];
            (point.x(), point.y())
        })
        .collect()
}
