use crate::{
    ProjectVolumeType, SliceError,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        region_slices::{
            PendingRegionSlices, PostRegionPrintObject, complex::compose_complex_region_slices,
            prepare_region_slices,
        },
        task22j_oracle,
        volume_regions::{VolumeRegion, VolumeRegionGraph},
    },
};

use super::super::region_fixture;
use super::{VolumeCase, bounded, compose, region, volume_case};

#[test]
fn task22j_complex_later_source_order_model_wins_despite_occurrence_sort() {
    let base = rect(0, 0, 1_000, 1_000);
    let later = rect(500, 0, 1_500, 1_000);
    let pending = compose(
        &[(0, 0.2, 0.1)],
        vec![
            volume_case(
                90,
                ProjectVolumeType::ModelPart,
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                vec![vec![base]],
            ),
            volume_case(
                10,
                ProjectVolumeType::ModelPart,
                [0.5, 0.0, 0.0, 1.5, 1.0, 1.0],
                vec![vec![later.clone()]],
            ),
        ],
        2,
        &[
            (0, ProjectVolumeType::ModelPart, Some(0)),
            (1, ProjectVolumeType::ModelPart, Some(1)),
        ],
    );

    let output = compose_complex_region_slices(pending, CoordinateScale::Normal).unwrap();
    assert_eq!(occurrences(&output), vec![10, 90]);
    assert_eq!(
        geometry(&output, 0, 0),
        vec![polygon(&[(500, 1_000), (0, 1_000), (0, 0), (500, 0)])]
    );
    assert_eq!(geometry(&output, 1, 0), vec![later]);
}

#[test]
#[rustfmt::skip]
fn task22j_complex_negative_subtracts_only_preceding_model_parts() {
    let band = rect(400, -100, 600, 1_100);
    let after = compose(
        &[(0, 0.2, 0.1)],
        vec![
            volume_case(100, ProjectVolumeType::ModelPart, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![vec![rect(0, 0, 1_000, 1_000)]]),
            volume_case(50, ProjectVolumeType::NegativeVolume, [0.4, -0.1, 0.0, 0.6, 1.1, 1.0], vec![vec![band.clone()]]),
        ],
        1,
        &[(0, ProjectVolumeType::ModelPart, Some(0)), (1, ProjectVolumeType::NegativeVolume, None)],
    );
    let after = compose_complex_region_slices(after, CoordinateScale::Normal).unwrap();
    assert_eq!(geometry(&after, 0, 0), vec![
        polygon(&[(1_000,1_000),(600,1_000),(600,0),(1_000,0)]),
        polygon(&[(400,1_000),(0,1_000),(0,0),(400,0)]),
    ]);

    let before = compose(
        &[(0, 0.2, 0.1)],
        vec![
            volume_case(50, ProjectVolumeType::NegativeVolume, [0.4, -0.1, 0.0, 0.6, 1.1, 1.0], vec![vec![band]]),
            volume_case(100, ProjectVolumeType::ModelPart, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![vec![rect(0, 0, 1_000, 1_000)]]),
            volume_case(150, ProjectVolumeType::ParameterModifier, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![vec![]]),
        ],
        1,
        &[(0, ProjectVolumeType::NegativeVolume, None), (1, ProjectVolumeType::ModelPart, Some(0)), (2, ProjectVolumeType::ParameterModifier, Some(0))],
    );
    let before = compose_complex_region_slices(before, CoordinateScale::Normal).unwrap();
    assert_eq!(geometry(&before, 0, 0), vec![rect(0, 0, 1_000, 1_000)]);
}

#[test]
#[rustfmt::skip]
fn task22j_complex_modifiers_partition_empty_chain_and_forward_one_source() {
    let empty_parent = pending(
        vec![
            volume_case(70,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,0.4,1.0,1.0],vec![vec![]]),
            volume_case(30,ProjectVolumeType::ModelPart,[0.6,0.0,0.0,1.0,1.0,1.0],vec![vec![rect(600,0,1_000,1_000)]]),
            volume_case(50,ProjectVolumeType::ParameterModifier,[0.2,0.0,0.0,0.8,1.0,1.0],vec![vec![rect(200,0,800,1_000)]]),
        ], 3, &[(0,ProjectVolumeType::ModelPart,None,Some(0)),(1,ProjectVolumeType::ModelPart,None,Some(1)),(2,ProjectVolumeType::ParameterModifier,Some(0),Some(2)),(2,ProjectVolumeType::ParameterModifier,Some(1),Some(2))]);
    let empty_parent = compose_complex_region_slices(empty_parent, CoordinateScale::Normal).unwrap();
    assert!(geometry(&empty_parent, 0, 0).is_empty());
    assert_eq!(geometry(&empty_parent, 1, 0), vec![polygon(&[(1_000,1_000),(800,1_000),(800,0),(1_000,0)])]);
    assert_eq!(geometry(&empty_parent, 2, 0), vec![polygon(&[(800,1_000),(600,1_000),(600,0),(800,0)])]);

    let chain = pending(
        vec![
            volume_case(101,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,1.2,1.2,1.0],vec![vec![rect(0,0,1_200,1_200)]]),
            volume_case(202,ProjectVolumeType::ParameterModifier,[0.3,0.0,0.0,0.9,1.2,1.0],vec![vec![rect(300,0,900,1_200)]]),
            volume_case(303,ProjectVolumeType::ParameterModifier,[0.0,0.4,0.0,1.2,0.8,1.0],vec![vec![rect(0,400,1_200,800)]]),
        ], 3, &[(0,ProjectVolumeType::ModelPart,None,Some(0)),(1,ProjectVolumeType::ParameterModifier,Some(0),Some(1)),(2,ProjectVolumeType::ParameterModifier,Some(1),Some(2))]);
    let chain = compose_complex_region_slices(chain, CoordinateScale::Normal).unwrap();
    assert_eq!(geometry(&chain, 2, 0), vec![polygon(&[(900,800),(300,800),(300,400),(900,400)])]);

}

#[test]
#[rustfmt::skip]
fn task22j_complex_stable_sort_uses_only_validity_region_and_occurrence() {
    let pending = pending(
        vec![
            volume_case(90,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,0.4,1.0,1.0],vec![vec![rect(0,0,400,1_000)]]),
            volume_case(10,ProjectVolumeType::ModelPart,[0.6,0.0,0.0,1.0,1.0,1.0],vec![vec![rect(600,0,1_000,1_000)]]),
            volume_case(50,ProjectVolumeType::ParameterModifier,[0.2,0.0,0.0,0.8,1.0,1.0],vec![vec![rect(200,0,800,1_000)]]),
            volume_case(5,ProjectVolumeType::NegativeVolume,[2.0,0.0,0.0,2.1,1.0,1.0],vec![vec![rect(2_000,0,2_100,1_000)]]),
        ], 2, &[(0,ProjectVolumeType::ModelPart,None,Some(0)),(1,ProjectVolumeType::ModelPart,None,Some(0)),(2,ProjectVolumeType::ParameterModifier,Some(1),Some(1)),(2,ProjectVolumeType::ParameterModifier,Some(0),Some(1)),(3,ProjectVolumeType::NegativeVolume,None,None)]);
    let output = compose_complex_region_slices(pending, CoordinateScale::LargeBed).unwrap();
    assert_eq!(occurrences(&output), vec![5,10,50,90]);
    assert_eq!(geometry(&output, 0, 0), vec![
        polygon(&[(1_000,1_000),(800,1_000),(800,0),(1_000,0)]),
        polygon(&[(200,1_000),(0,1_000),(0,0),(200,0)]),
    ]);
    assert_eq!(geometry(&output, 1, 0), vec![
        polygon(&[(800,1_000),(600,1_000),(600,0),(800,0)]),
        polygon(&[(400,1_000),(200,1_000),(200,0),(400,0)]),
    ]);
}

#[test]
#[rustfmt::skip]
fn task22j_complex_closing_requires_an_actual_same_region_append_and_uses_scale() {
    let normal = pending_layers(
        &[(0,0.2,0.1),(1,0.4,0.3)],
        vec![
            volume_case(900,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,1.0,1.0,1.0],vec![vec![rect(0,0,1_000,1_000)],vec![rect(0,0,1_000,1_000)]]),
            volume_case(17,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,1.0,1.0,1.0],vec![vec![rect(1_150,0,2_150,1_000)],vec![rect(1_250,0,2_250,1_000)]]),
        ], 1, &[(0,ProjectVolumeType::ModelPart,None,Some(0)),(1,ProjectVolumeType::ModelPart,None,Some(0))]);
    let normal = compose_complex_region_slices(normal, CoordinateScale::Normal).unwrap();
    assert_eq!(geometry(&normal,0,0),vec![polygon(&[(2_150,1_000),(0,1_000),(0,0),(2_150,0)])]);
    assert_eq!(geometry(&normal,0,1),vec![
        polygon(&[(2_250,1_000),(1_250,1_000),(1_250,0),(2_250,0)]),
        polygon(&[(1_000,1_000),(0,1_000),(0,0),(1_000,0)]),
    ]);

    let large = pending_layers(
        &[(0,0.2,0.1),(1,0.4,0.3)],
        vec![
            volume_case(900,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,1.0,1.0,1.0],vec![vec![rect(0,0,1_000,1_000)],vec![rect(0,0,1_000,1_000)]]),
            volume_case(17,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,1.0,1.0,1.0],vec![vec![rect(1_015,0,2_015,1_000)],vec![rect(1_025,0,2_025,1_000)]]),
        ], 1, &[(0,ProjectVolumeType::ModelPart,None,Some(0)),(1,ProjectVolumeType::ModelPart,None,Some(0))]);
    let large = compose_complex_region_slices(large, CoordinateScale::LargeBed).unwrap();
    assert_eq!(geometry(&large,0,0),vec![polygon(&[(2_015,1_000),(0,1_000),(0,0),(2_015,0)])]);
    assert_eq!(geometry(&large,0,1),vec![
        polygon(&[(2_025,1_000),(1_025,1_000),(1_025,0),(2_025,0)]),
        polygon(&[(1_000,1_000),(0,1_000),(0,0),(1_000,0)]),
    ]);

    let left = rect(0,0,1_000,1_000); let right = rect(1_050,0,2_050,1_000);
    let single = pending(vec![
        volume_case(1,ProjectVolumeType::ModelPart,[0.0,0.0,0.0,1.0,1.0,1.0],vec![vec![left.clone(),right.clone()]]),
        volume_case(2,ProjectVolumeType::NegativeVolume,[0.0,0.0,0.0,1.0,1.0,1.0],vec![vec![]]),
    ],1,&[(0,ProjectVolumeType::ModelPart,None,Some(0)),(1,ProjectVolumeType::NegativeVolume,None,None)]);
    let single = compose_complex_region_slices(single,CoordinateScale::Normal).unwrap();
    assert_eq!(geometry(&single,0,0),vec![left,right]);
}

#[test]
fn task22j_complex_closing_maps_generated_clipper_range_error_exactly() {
    let hi = 0x3fff_ffff_ffff_ffff;
    let pending = pending(
        vec![
            volume_case(
                1,
                ProjectVolumeType::ModelPart,
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                vec![vec![rect(hi - 2_048, 0, hi, 1_000)]],
            ),
            volume_case(
                2,
                ProjectVolumeType::ModelPart,
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                vec![vec![rect(0, 0, 1_000, 1_000)]],
            ),
        ],
        1,
        &[
            (0, ProjectVolumeType::ModelPart, None, Some(0)),
            (1, ProjectVolumeType::ModelPart, None, Some(0)),
        ],
    );
    let Err(error) = compose_complex_region_slices(pending, CoordinateScale::Normal) else {
        panic!("closing must reject its generated out-of-range coordinate");
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "project region composition polygon coordinate is outside the supported Clipper range"
                .to_owned()
        )
    );
}

#[test]
fn task22j_complex_complete_synthetic_stream_is_exact() {
    region_fixture::assert_synthetic_j(&task22j_oracle::encode(&synthetic_outputs()));
}

#[rustfmt::skip]
fn synthetic_outputs() -> Vec<PostRegionPrintObject> {
    use CoordinateScale::{LargeBed, Normal}; use ProjectVolumeType::{ModelPart as Part, NegativeVolume as Neg, ParameterModifier as Mod};
    vec![
        synthetic((0,Normal),&[(7,0.6,0.5),(42,1.1,1.0),(99,1.6,1.5)],vec![volume_case(42,Part,[0.,0.,0.,0.4,0.4,2.],vec![vec![rect(0,0,100,100)],vec![],vec![rect(0,0,300,100)]])],1,&[(0,Part,None,Some(0))]),
        synthetic((1,Normal),&[(0,0.6,0.5)],vec![volume_case(70,Part,[0.,0.,0.,0.4,0.4,3.],vec![vec![rect(0,0,400,400)]]),volume_case(20,Part,[1.,0.,0.,1.4,0.4,3.],vec![vec![rect(1_000,0,1_400,400)]])],2,&[(0,Part,None,Some(0)),(1,Part,None,Some(1))]),
        synthetic((2,Normal),&[(0,0.6,0.5)],vec![volume_case(90,Part,[0.,0.,0.,1.,1.,1.],vec![vec![rect(0,0,1_000,1_000)]]),volume_case(10,Part,[0.5,0.,0.,1.5,1.,1.],vec![vec![rect(500,0,1_500,1_000)]])],2,&[(0,Part,None,Some(0)),(1,Part,None,Some(1))]),
        synthetic((3,Normal),&[(0,0.6,0.5)],vec![volume_case(100,Part,[0.,0.,0.,1.,1.,1.],vec![vec![rect(0,0,1_000,1_000)]]),volume_case(50,Neg,[0.4,-0.1,0.,0.6,1.1,1.],vec![vec![rect(400,-100,600,1_100)]])],1,&[(0,Part,None,Some(0)),(1,Neg,None,None)]),
        synthetic((4,Normal),&[(0,0.6,0.5)],vec![volume_case(50,Neg,[0.4,-0.1,0.,0.6,1.1,1.],vec![vec![rect(400,-100,600,1_100)]]),volume_case(100,Part,[0.,0.,0.,1.,1.,1.],vec![vec![rect(0,0,1_000,1_000)]])],1,&[(0,Neg,None,None),(1,Part,None,Some(0))]),
        synthetic((5,Normal),&[(0,0.6,0.5)],vec![volume_case(101,Part,[0.,0.,0.,1.2,1.2,1.],vec![vec![rect(0,0,1_200,1_200)]]),volume_case(202,Mod,[0.3,0.,0.,0.9,1.2,1.],vec![vec![rect(300,0,900,1_200)]]),volume_case(303,Mod,[0.,0.4,0.,1.2,0.8,1.],vec![vec![rect(0,400,1_200,800)]])],3,&[(0,Part,None,Some(0)),(1,Mod,Some(0),Some(1)),(2,Mod,Some(1),Some(2))]),
        synthetic((6,LargeBed),&[(0,0.6,0.5)],vec![volume_case(70,Part,[0.,0.,0.,0.4,1.,1.],vec![vec![rect(0,0,400,1_000)]]),volume_case(30,Part,[0.6,0.,0.,1.,1.,1.],vec![vec![rect(600,0,1_000,1_000)]]),volume_case(50,Mod,[0.2,0.,0.,0.8,1.,1.],vec![vec![rect(200,0,800,1_000)]])],2,&[(0,Part,None,Some(0)),(1,Part,None,Some(0)),(2,Mod,Some(1),Some(1)),(2,Mod,Some(0),Some(1))]),
        synthetic((7,Normal),&[(0,0.6,0.5),(1,1.6,1.5)],vec![volume_case(900,Part,[0.,0.,0.,2.25,1.,2.],vec![vec![rect(0,0,1_000,1_000)],vec![rect(0,0,1_000,1_000)]]),volume_case(17,Part,[0.,0.,0.,2.25,1.,2.],vec![vec![rect(1_150,0,2_150,1_000)],vec![rect(1_250,0,2_250,1_000)]])],1,&[(0,Part,None,Some(0)),(1,Part,None,Some(0))]),
        synthetic((8,LargeBed),&[(0,0.6,0.5),(1,1.6,1.5)],vec![volume_case(900,Part,[0.,0.,0.,2.025,1.,2.],vec![vec![rect(0,0,1_000,1_000)],vec![rect(0,0,1_000,1_000)]]),volume_case(17,Part,[0.,0.,0.,2.025,1.,2.],vec![vec![rect(1_015,0,2_015,1_000)],vec![rect(1_025,0,2_025,1_000)]])],1,&[(0,Part,None,Some(0)),(1,Part,None,Some(0))]),
        synthetic((9,Normal),&[(0,0.6,0.5),(1,1.6,1.5)],vec![volume_case(90,Part,[0.,0.,0.,1.,1.,2.],vec![vec![rect(0,0,1_000,1_000)],vec![rect(0,0,1_000,1_000)]]),volume_case(10,Neg,[0.,0.,0.,1.,1.,2.],vec![vec![],vec![rect(0,0,1_000,1_000)]])],1,&[(0,Part,None,Some(0)),(1,Neg,None,None)]),
    ]
}

fn synthetic(
    (source_object_index, scale): (usize, CoordinateScale),
    layers: &[(usize, f64, f64)],
    cases: Vec<VolumeCase>,
    region_count: usize,
    records: &[(usize, ProjectVolumeType, Option<usize>, Option<usize>)],
) -> PostRegionPrintObject {
    let pending = pending_layers(layers, cases, region_count, records);
    let mut output = compose_complex_region_slices(pending, scale).unwrap();
    output.plan.source_object_index = source_object_index;
    output
}

fn occurrences(output: &PostRegionPrintObject) -> Vec<u32> {
    output
        .as_parts()
        .1
        .iter()
        .map(|volume| volume.as_parts().0.get())
        .collect()
}

fn geometry(output: &PostRegionPrintObject, region: usize, layer: usize) -> Vec<ExPolygon> {
    output.as_parts().2[region].as_parts().2[layer]
        .surfaces()
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect()
}

fn pending(
    cases: Vec<VolumeCase>,
    region_count: usize,
    records: &[(usize, ProjectVolumeType, Option<usize>, Option<usize>)],
) -> PendingRegionSlices {
    pending_layers(&[(0, 0.2, 0.1)], cases, region_count, records)
}

fn pending_layers(
    layer_zs: &[(usize, f64, f64)],
    cases: Vec<VolumeCase>,
    region_count: usize,
    records: &[(usize, ProjectVolumeType, Option<usize>, Option<usize>)],
) -> PendingRegionSlices {
    let bounded = bounded(layer_zs, cases);
    let graph = VolumeRegionGraph {
        all_regions: (0..region_count).map(|_| region()).collect(),
        volume_regions: records
            .iter()
            .map(|&(source, kind, parent, region_id)| VolumeRegion {
                source_volume_index: source,
                occurrence_id: bounded
                    .volume_for_source_index(source)
                    .unwrap()
                    .occurrence_id(),
                kind,
                parent,
                region_id,
                bound_index: bounded.bound_index_for_source_index(source).unwrap(),
            })
            .collect(),
    };
    prepare_region_slices(bounded, graph)
}

fn rect(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    polygon(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
}

fn polygon(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()),
        Vec::new(),
    )
}
