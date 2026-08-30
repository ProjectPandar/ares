use super::{FuzzySkinInput, apply};
use crate::{
    OrcaBool, OrcaFloat, OrcaInt, Percent, ProcessFuzzySkinMode, ProcessFuzzySkinType,
    ProcessNoiseType, ProcessRegionSourceOptions, RegionOptions,
    geometry::{CoordinateScale, Point, Polygon},
    perimeters::FuzzySkinConfig,
};

fn scale() -> CoordinateScale {
    CoordinateScale::from_printable_area(&crate::Point2dList(vec![
        crate::Point2d::new(0.0, 0.0),
        crate::Point2d::new(256.0, 256.0),
    ]))
}

fn polygon() -> Polygon {
    let scale = scale();
    Polygon::new(
        [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]
            .into_iter()
            .map(|(x, y)| {
                Point::new(
                    scale.checked_scale(x).unwrap(),
                    scale.checked_scale(y).unwrap(),
                )
            })
            .collect(),
    )
}

fn config(kind: ProcessFuzzySkinType, mode: ProcessFuzzySkinMode) -> FuzzySkinConfig {
    let mut region = RegionOptions::from_base(&ProcessRegionSourceOptions::default());
    region.fuzzy_skin = kind;
    region.fuzzy_skin_first_layer = OrcaBool(true);
    region.fuzzy_skin_noise_type = ProcessNoiseType::Ripple;
    region.fuzzy_skin_mode = mode;
    region.fuzzy_skin_thickness = OrcaFloat(0.2);
    region.fuzzy_skin_point_distance = OrcaFloat(1.0);
    region.fuzzy_skin_ripples_per_layer = OrcaInt(1);
    region.fuzzy_skin_ripple_offset = Percent(0.0);
    region.fuzzy_skin_layers_between_ripple_offset = OrcaInt(1);
    FuzzySkinConfig::from_region(&region)
}

#[test]
fn source_fuzzy_type_matrix_covers_contours_holes_and_inset_depth() {
    let cases = [
        (ProcessFuzzySkinType::None, [false; 4]),
        (ProcessFuzzySkinType::Disabled, [false; 4]),
        (ProcessFuzzySkinType::External, [true, false, false, false]),
        (ProcessFuzzySkinType::Hole, [false, true, false, false]),
        (ProcessFuzzySkinType::All, [true, true, false, false]),
        (ProcessFuzzySkinType::AllWalls, [true, true, true, true]),
    ];
    for (kind, expected) in cases {
        let config = config(kind, ProcessFuzzySkinMode::Displacement);
        assert_eq!(
            [
                config.should_fuzzify(1, 0, true),
                config.should_fuzzify(1, 0, false),
                config.should_fuzzify(1, 1, true),
                config.should_fuzzify(1, 1, false),
            ],
            expected,
            "{kind:?}"
        );
    }
}

#[test]
fn classic_polygon_fuzzy_runs_after_polygon_generation_for_all_modes() {
    let source = polygon();
    let outputs = [
        ProcessFuzzySkinMode::Displacement,
        ProcessFuzzySkinMode::Extrusion,
        ProcessFuzzySkinMode::Combined,
    ]
    .map(|mode| {
        apply(
            &source,
            FuzzySkinInput {
                config: config(ProcessFuzzySkinType::External, mode),
                layer_id: 0,
                slice_z: 0.1,
                loop_index: 0,
                is_contour: true,
                scale: scale(),
            },
        )
        .unwrap()
    });

    assert!(outputs.iter().all(|output| output != &source));
    assert!(outputs.iter().all(|output| output.points().len() > 4));
    assert_eq!(outputs[0], outputs[1]);
    assert_eq!(outputs[1], outputs[2]);
}
