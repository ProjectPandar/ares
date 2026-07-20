use crate::{
    FloatOrPercent, OrcaFloat, OrcaFloats, OrcaInt, Percent,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        compensation::{PostCompensationPrintObject, apply_project_compensation},
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface},
        task22m_oracle::encode,
    },
};

use super::super::{
    region_fixture::checkpoint::sha256,
    support::{identity_resolved, object_options, region},
};
use super::fixture::parse_m_object_count;

const EXPECTED_LEN: usize = 10_351;
const EXPECTED_SHA256: &str = "c112246ff48b280eb803082749d74315e771d073b0407e45afde536e37fcf46d";
const DEFAULT_NOZZLES: &[f64] = &[0.4, 0.4];
const MIXED_NOZZLES: &[f64] = &[0.4, 0.6];

#[derive(Clone, Copy)]
struct CaseOptions {
    compensation: f64,
    compensation_layers: i32,
    raft_layers: i32,
    initial_width: FloatOrPercent,
    outer_width: FloatOrPercent,
    object_width: FloatOrPercent,
    selector: i32,
    nozzles: &'static [f64],
    scale: CoordinateScale,
}

struct InputLayer {
    height: f64,
    expolygons: Vec<ExPolygon>,
}

struct SyntheticCase {
    source_object_index: usize,
    options: CaseOptions,
    layers: Vec<InputLayer>,
}

#[test]
fn task22m_synthetic_aggregate_is_exact_complete_and_repeatable() {
    let first = synthetic_frame();
    assert_eq!(&first[..8], b"ARES22M\0");
    assert_eq!(parse_m_object_count(&first), 19);
    assert_eq!(first.len(), EXPECTED_LEN);
    assert_eq!(sha256(&first), EXPECTED_SHA256);
    assert_eq!(synthetic_frame(), first);
}

fn synthetic_frame() -> Vec<u8> {
    let objects = cases().into_iter().map(apply_case).collect::<Vec<_>>();
    assert_eq!(objects.len(), 19);
    encode(&objects)
}

fn cases() -> Vec<SyntheticCase> {
    let enabled = CaseOptions {
        compensation: 0.15,
        compensation_layers: 1,
        raft_layers: 0,
        initial_width: FloatOrPercent::Float(0.5),
        outer_width: FloatOrPercent::Float(0.42),
        object_width: FloatOrPercent::Float(0.42),
        selector: 0,
        nozzles: DEFAULT_NOZZLES,
        scale: CoordinateScale::Normal,
    };
    let disabled = CaseOptions {
        compensation: 0.0,
        ..enabled
    };
    let raft = CaseOptions {
        raft_layers: 1,
        ..enabled
    };
    let ramp = CaseOptions {
        compensation: 0.2,
        compensation_layers: 2,
        ..enabled
    };
    let clamp = CaseOptions {
        compensation_layers: 5,
        ..ramp
    };
    let varied_height = CaseOptions {
        initial_width: FloatOrPercent::Float(0.42),
        ..ramp
    };
    let initial_override = CaseOptions {
        initial_width: FloatOrPercent::Float(0.6),
        ..enabled
    };
    let initial_to_outer = CaseOptions {
        initial_width: FloatOrPercent::Float(0.0),
        outer_width: FloatOrPercent::Float(0.38),
        ..enabled
    };
    let outer_to_object = CaseOptions {
        outer_width: FloatOrPercent::Float(0.0),
        object_width: FloatOrPercent::Float(0.52),
        ..initial_to_outer
    };
    let auto_width = CaseOptions {
        object_width: FloatOrPercent::Float(0.0),
        ..outer_to_object
    };
    let negative_auto = CaseOptions {
        outer_width: FloatOrPercent::Float(-1.0),
        ..initial_to_outer
    };
    let percent_second_nozzle = CaseOptions {
        initial_width: FloatOrPercent::Percent(Percent(125.0)),
        selector: 2,
        nozzles: MIXED_NOZZLES,
        ..enabled
    };
    let selector_fallback = CaseOptions {
        selector: 3,
        ..percent_second_nozzle
    };
    let large_bed = CaseOptions {
        scale: CoordinateScale::LargeBed,
        ..enabled
    };
    let large = rectangle(0, 0, 20_000_000, 12_000_000);

    vec![
        one_layer_case(0, enabled, large.clone()),
        one_layer_case(1, enabled, narrow_neck()),
        one_layer_case(2, enabled, rectangle(0, 0, 1_000_000, 1_000_000)),
        one_layer_case(3, disabled, large.clone()),
        one_layer_case(4, raft, large.clone()),
        SyntheticCase {
            source_object_index: 5,
            options: ramp,
            layers: vec![
                layer(flow_sensitive_neck(850_000)),
                layer(flow_sensitive_neck(850_000)),
                layer(large.clone()),
            ],
        },
        SyntheticCase {
            source_object_index: 6,
            options: clamp,
            layers: vec![
                layer(flow_sensitive_neck(850_000)),
                layer(flow_sensitive_neck(850_000)),
            ],
        },
        SyntheticCase {
            source_object_index: 7,
            options: enabled,
            layers: Vec::new(),
        },
        SyntheticCase {
            source_object_index: 8,
            options: enabled,
            layers: vec![InputLayer {
                height: 0.2,
                expolygons: Vec::new(),
            }],
        },
        SyntheticCase {
            source_object_index: 9,
            options: varied_height,
            layers: vec![
                layer(flow_sensitive_neck(790_000)),
                InputLayer {
                    height: 0.3,
                    expolygons: vec![flow_sensitive_neck(790_000)],
                },
            ],
        },
        one_layer_case(10, initial_override, flow_sensitive_neck(1_050_000)),
        one_layer_case(11, initial_to_outer, flow_sensitive_neck(850_000)),
        one_layer_case(12, outer_to_object, flow_sensitive_neck(850_000)),
        one_layer_case(13, auto_width, flow_sensitive_neck(850_000)),
        one_layer_case(14, negative_auto, flow_sensitive_neck(850_000)),
        one_layer_case(15, percent_second_nozzle, flow_sensitive_neck(1_200_000)),
        one_layer_case(16, selector_fallback, flow_sensitive_neck(850_000)),
        one_layer_case(17, large_bed, rectangle(0, 0, 2_000_000, 1_200_000)),
        SyntheticCase {
            source_object_index: 18,
            options: enabled,
            layers: vec![InputLayer {
                height: 0.2,
                expolygons: two_pass_union_discriminant(),
            }],
        },
    ]
}

fn one_layer_case(
    source_object_index: usize,
    options: CaseOptions,
    expolygon: ExPolygon,
) -> SyntheticCase {
    SyntheticCase {
        source_object_index,
        options,
        layers: vec![layer(expolygon)],
    }
}

fn layer(expolygon: ExPolygon) -> InputLayer {
    InputLayer {
        height: 0.2,
        expolygons: vec![expolygon],
    }
}

fn apply_case(case: SyntheticCase) -> PostCompensationPrintObject {
    let mut resolved = identity_resolved(case.source_object_index);
    let mut object_options = object_options();
    object_options.elefant_foot_compensation = OrcaFloat(case.options.compensation);
    object_options.elefant_foot_compensation_layers = OrcaInt(case.options.compensation_layers);
    object_options.raft_layers = OrcaInt(case.options.raft_layers);
    object_options.line_width = case.options.object_width;
    resolved.object = object_options;

    let mut region_options = region();
    region_options.outer_wall_line_width = case.options.outer_width;
    region_options.outer_wall_filament_id = OrcaInt(case.options.selector);
    let object = post_region_object(case.source_object_index, region_options, case.layers);
    let nozzles = OrcaFloats(
        case.options
            .nozzles
            .iter()
            .copied()
            .map(OrcaFloat)
            .collect(),
    );
    let mut output = apply_project_compensation(
        vec![object],
        std::slice::from_ref(&resolved),
        case.options.initial_width,
        &nozzles,
        case.options.scale,
    )
    .unwrap();
    assert_eq!(output.len(), 1);
    output.pop().unwrap()
}

fn post_region_object(
    source_object_index: usize,
    options: crate::RegionOptions,
    layers: Vec<InputLayer>,
) -> PostRegionPrintObject {
    let mut print_z = 0.0;
    let mut planned_layers = Vec::with_capacity(layers.len());
    let mut region_layers = Vec::with_capacity(layers.len());
    for (id, layer) in layers.into_iter().enumerate() {
        let slice_z = print_z + 0.5 * layer.height;
        print_z += layer.height;
        planned_layers.push(PlannedLayer {
            id,
            height: layer.height,
            print_z,
            slice_z,
        });
        region_layers.push(RegionLayer {
            surfaces: layer
                .expolygons
                .into_iter()
                .map(RegionSurface::internal)
                .collect(),
        });
    }
    PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index,
            transform_index: 0,
            layers: planned_layers,
        },
        volume_slices: Vec::new(),
        regions: vec![PostRegion {
            id: 0,
            options,
            layers: region_layers,
        }],
    }
}

fn narrow_neck() -> ExPolygon {
    let hole = [
        (500_000, 500_000),
        (500_000, 2_000_000),
        (2_000_000, 2_000_000),
        (2_000_000, 500_000),
    ];
    expolygon(
        &[
            (0, 0),
            (8_000_000, 0),
            (8_000_000, 4_000_000),
            (4_350_000, 4_000_000),
            (4_350_000, 9_000_000),
            (3_650_000, 9_000_000),
            (3_650_000, 4_000_000),
            (0, 4_000_000),
        ],
        &[&hole],
    )
}

fn flow_sensitive_neck(width: i64) -> ExPolygon {
    let left = 4_000_000 - width / 2;
    let right = left + width;
    expolygon(
        &[
            (0, 0),
            (8_000_000, 0),
            (8_000_000, 4_000_000),
            (right, 4_000_000),
            (right, 9_000_000),
            (left, 9_000_000),
            (left, 4_000_000),
            (0, 4_000_000),
        ],
        &[],
    )
}

fn two_pass_union_discriminant() -> Vec<ExPolygon> {
    let left_hole = [(10, 10), (10, 50), (50, 50), (50, 10)];
    let right_hole = [(110, 10), (110, 50), (150, 50), (150, 10)];
    vec![
        expolygon(&[(0, 0), (60, 0), (60, 60), (0, 60)], &[&left_hole]),
        rectangle(20, 20, 40, 40),
        expolygon(&[(100, 0), (160, 0), (160, 60), (100, 60)], &[&right_hole]),
    ]
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    expolygon(
        &[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ],
        &[],
    )
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
