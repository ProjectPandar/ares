use crate::{
    FloatOrPercent, OrcaFloat, OrcaFloats, OrcaInt,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        compensation::apply_project_compensation,
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface},
    },
};

use super::super::support::{identity_resolved, object_options, region};

#[test]
fn task22m_two_pass_union_preserves_discriminant_geometry() {
    let mut resolved = identity_resolved(18);
    let mut object_options = object_options();
    object_options.elefant_foot_compensation = OrcaFloat(0.15);
    object_options.elefant_foot_compensation_layers = OrcaInt(1);
    object_options.raft_layers = OrcaInt(0);
    object_options.line_width = FloatOrPercent::Float(0.42);
    resolved.object = object_options;

    let object = PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index: 18,
            transform_index: 0,
            layers: vec![PlannedLayer {
                id: 0,
                height: 0.2,
                print_z: 0.2,
                slice_z: 0.1,
            }],
        },
        volume_slices: Vec::new(),
        regions: vec![PostRegion {
            id: 0,
            options: region(),
            layers: vec![RegionLayer {
                surfaces: two_pass_union_discriminant()
                    .into_iter()
                    .map(RegionSurface::internal)
                    .collect(),
            }],
        }],
    };
    let (post_regions, _) = apply_project_compensation(
        vec![object],
        std::slice::from_ref(&resolved),
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4)]),
        CoordinateScale::Normal,
    )
    .unwrap()
    .pop()
    .unwrap()
    .into_parts();
    let (_, _, regions) = post_regions.into_parts();
    let surfaces = regions
        .into_iter()
        .flat_map(|region| region.into_parts().2)
        .map(|layer| {
            layer
                .into_parts()
                .into_iter()
                .map(|surface| surface.into_parts().1)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        surfaces,
        vec![vec![
            expolygon(
                &[(60, 60), (0, 60), (0, 0), (60, 0)],
                &[&[(10, 10), (10, 50), (50, 50), (50, 10)]],
            ),
            expolygon(&[(40, 40), (20, 40), (20, 20), (40, 20)], &[]),
            expolygon(
                &[(160, 60), (100, 60), (100, 0), (160, 0)],
                &[&[(110, 10), (110, 50), (150, 50), (150, 10)]],
            ),
        ]]
    );
}

fn two_pass_union_discriminant() -> Vec<ExPolygon> {
    let left_hole = [(10, 10), (10, 50), (50, 50), (50, 10)];
    let right_hole = [(110, 10), (110, 50), (150, 50), (150, 10)];
    vec![
        expolygon(&[(0, 0), (60, 0), (60, 60), (0, 60)], &[&left_hole]),
        expolygon(&[(20, 20), (40, 20), (40, 40), (20, 40)], &[]),
        expolygon(&[(100, 0), (160, 0), (160, 60), (100, 60)], &[&right_hole]),
    ]
}

fn expolygon(contour: &[(i64, i64)], holes: &[&[(i64, i64)]]) -> ExPolygon {
    ExPolygon::new(
        polygon(contour),
        holes.iter().map(|points| polygon(points)).collect(),
    )
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
