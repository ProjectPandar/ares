use crate::{
    ObjectOptions, OrcaFloat, OrcaInt, Point3d, ProjectObject, ProjectVolume, ProjectVolumeType,
    SliceError, Transform3d,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    load_project,
    mesh_slicer::SlicingMode,
    project::effective_config::{resolve_bounded_project_config, types::ResolvedProjectObject},
};

use super::{
    super::{
        chained_intersections::chain_project_intersections,
        closing::{
            ClosingDeltas, PostClosingPrintObject, apply_project_closing, close_expolygons,
            closing_deltas,
        },
        layers::PlannedPrintObject,
        looped_intersections::loop_project_intersections,
        pre_closing_unions::{PreClosingPrintObject, apply_project_pre_closing_unions},
        slicing_mode_intersections::apply_project_slicing_modes,
        task22g_oracle::encode,
    },
    raw_support::{intersections, mesh_volume, planned_layers},
    support::{KsrArchive, object, region, resolved},
};

const NORMAL_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"256x0\",\r\n",
    "\t\t\"256x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);
const LARGE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"3000x0\",\r\n",
    "\t\t\"3000x3000\",\r\n",
    "\t\t\"0x3000\"\r\n",
    "\t]",
);

#[test]
fn task22g_closing_deltas_freeze_exact_float_scale_chain() {
    for (radius, scale, expected, forbidden) in [
        (0.049, CoordinateScale::Normal, 0x473f_6800, None),
        (0.049, CoordinateScale::LargeBed, 0x4599_2000, None),
        (
            0.049_000_4,
            CoordinateScale::Normal,
            0x473f_6867,
            Some(0x473f_6866),
        ),
        (
            0.333_333_333_333_333_3,
            CoordinateScale::LargeBed,
            0x4702_3556,
            Some(0x4702_3555),
        ),
    ] {
        let deltas = closing_deltas(radius, scale).unwrap().unwrap();
        assert_eq!(deltas.outward.to_bits(), expected);
        assert_eq!(deltas.inward.to_bits(), expected | 0x8000_0000);
        if let Some(forbidden) = forbidden {
            assert_ne!(deltas.outward.to_bits(), forbidden);
        }
    }
}

#[test]
fn task22g_closing_zero_and_float_underflow_skip_offset_generation() {
    for radius in [0.0, -0.0, f64::MIN_POSITIVE] {
        assert_eq!(
            closing_deltas(radius, CoordinateScale::Normal).unwrap(),
            None
        );
    }
    let input = vec![expolygon(&[(40, 40), (0, 40), (0, 0), (40, 0)])];
    assert_eq!(close_expolygons(input.clone(), None).unwrap(), input);
}

#[test]
fn task22g_invalid_radius_and_scaled_overflow_share_exact_option_error() {
    let expected = Err(invalid_radius());
    for radius in [-0.001, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(closing_deltas(radius, CoordinateScale::Normal), expected);
    }
    let finite = 3.402_823_5e32;
    assert!((finite as f32).is_finite());
    assert_eq!(
        closing_deltas(finite, CoordinateScale::Normal),
        Err(invalid_radius())
    );
    let large = closing_deltas(finite, CoordinateScale::LargeBed)
        .unwrap()
        .unwrap();
    assert_eq!(large.outward.to_bits(), 0x7dcc_cccd);
    assert_eq!(large.inward.to_bits(), 0xfdcc_cccd);
}

#[test]
fn task22g_clipper_range_error_maps_once_at_project_boundary() {
    const HIGH: i64 = 0x3fff_ffff_ffff_ffff;
    let input = vec![expolygon(&[
        (HIGH - 16_384, 0),
        (HIGH - 8_192, 0),
        (HIGH - 8_192, 8_192),
        (HIGH - 16_384, 8_192),
    ])];
    assert_eq!(
        close_expolygons(
            input,
            Some(ClosingDeltas {
                outward: 16_384.0,
                inward: -16_384.0,
            }),
        ),
        Err(SliceError::InvalidInput(
            "project closing polygon coordinate is outside the supported Clipper range".to_owned()
        ))
    );
}

#[test]
fn task22g_real_3mf_options_keep_exact_normal_large_and_noninteger_float_chain() {
    for (radius, large_bed, expected_bits) in [
        ("0.049", false, 0x473f_6800),
        ("0.049", true, 0x4599_2000),
        ("0.0490004", false, 0x473f_6867),
        ("0.3333333333333333", true, 0x4702_3556),
    ] {
        let (options, scale) = resolved_3mf_options(radius, None, large_bed);
        assert_eq!(
            options.slice_closing_radius.0,
            radius.parse::<f64>().unwrap()
        );
        let deltas = closing_deltas(options.slice_closing_radius.0, scale)
            .unwrap()
            .unwrap();
        assert_eq!(deltas.outward.to_bits(), expected_bits);
        assert_eq!(deltas.inward.to_bits(), expected_bits | 0x8000_0000);
    }
}

#[test]
fn task22g_process_base_object_override_and_reversed_association_are_consumed() {
    let (base, _) = resolved_3mf_options("0.04", None, false);
    let (overridden, _) = resolved_3mf_options("0.04", Some("0.06"), false);
    assert_eq!(base.slice_closing_radius, OrcaFloat(0.04));
    assert_eq!(overridden.slice_closing_radius, OrcaFloat(0.06));

    let resolved_objects = vec![
        synthetic_resolved(0, base),
        synthetic_resolved(1, overridden),
    ];
    let sources = vec![separated_boxes_object(0), separated_boxes_object(1)];
    let plans = vec![
        planned_layers(0, 0, &[(1.2, 1.0)]),
        planned_layers(1, 0, &[(1.2, 1.0)]),
    ];
    let pre_closing = prepare_pre_closing(&sources, &resolved_objects, plans, false);
    assert_eq!(first_layer_counts_pre(&pre_closing), [2, 2]);

    let reversed = [resolved_objects[1].clone(), resolved_objects[0].clone()];
    let output = apply_project_closing(pre_closing, &reversed, CoordinateScale::Normal).unwrap();
    assert_eq!(
        output
            .iter()
            .map(|object| object.plan().source_object_index)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(first_layer_counts_post(&output), [2, 1]);
}

#[test]
fn task22g_zero_radius_preserves_owned_records_points_modes_and_empty_slots() {
    let (options, _) = resolved_3mf_options("0", None, false);
    let mut spiral_region = region();
    spiral_region.bottom_shell_layers = OrcaInt(0);
    spiral_region.bottom_shell_thickness = OrcaFloat(0.0);
    let resolved_objects = vec![resolved(0, options, vec![spiral_region])];
    let sources = vec![separated_boxes_object(0)];
    let plans = vec![planned_layers(0, 0, &[(1.2, 1.0), (3.2, 3.0)])];
    let pre_closing = prepare_pre_closing(&sources, &resolved_objects, plans, true);
    let before = pre_facts(&pre_closing);
    assert_eq!(
        before[0].volumes[0].layers[0].0,
        SlicingMode::PositiveLargestContour
    );
    assert_eq!(before[0].volumes[0].layers[0].1.len(), 2);
    assert!(before[0].volumes[0].layers[1].1.is_empty());

    let output =
        apply_project_closing(pre_closing, &resolved_objects, CoordinateScale::Normal).unwrap();
    assert_eq!(post_facts(&output), before);
    assert!(encode(&output).starts_with(b"ARES22G\0"));
}

#[test]
#[should_panic(expected = "pre-closing object must have resolved configuration")]
fn task22g_missing_resolved_object_is_an_internal_invariant() {
    let (options, _) = resolved_3mf_options("0", None, false);
    let resolved_objects = vec![synthetic_resolved(0, options)];
    let sources = vec![separated_boxes_object(0)];
    let plans = vec![planned_layers(0, 0, &[(1.2, 1.0)])];
    let pre_closing = prepare_pre_closing(&sources, &resolved_objects, plans, false);
    let _ = apply_project_closing(pre_closing, &[], CoordinateScale::Normal);
}

#[derive(Debug, PartialEq)]
struct ObjectFacts {
    plan: PlannedPrintObject,
    volumes: Vec<VolumeFacts>,
}

#[derive(Debug, PartialEq)]
struct VolumeFacts {
    source_volume_index: usize,
    ordinal: u32,
    volume_type: ProjectVolumeType,
    layers: Vec<(SlicingMode, Vec<ExPolygon>)>,
}

fn resolved_3mf_options(
    process_radius: &str,
    object_radius: Option<&str>,
    large_bed: bool,
) -> (ObjectOptions, CoordinateScale) {
    let mut archive = KsrArchive::new();
    if process_radius != "0.049" {
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"slice_closing_radius\": \"0.049\"",
            &format!("\"slice_closing_radius\": \"{process_radius}\""),
        );
    }
    if large_bed {
        archive.replace_unique("Metadata/project_settings.config", NORMAL_AREA, LARGE_AREA);
    }
    if let Some(radius) = object_radius {
        archive.replace(
            "Metadata/model_settings.config",
            r#"<object id="2">"#,
            &format!(r#"<object id="2"><metadata key="slice_closing_radius" value="{radius}"/>"#),
        );
    }
    let project = load_project(archive.bytes()).unwrap();
    let resolved = resolve_bounded_project_config(&project).unwrap();
    let scale =
        CoordinateScale::from_printable_area(&resolved.views.full.printer.remaining.printable_area);
    let options = resolved.objects.into_iter().next().unwrap().object;
    (options, scale)
}

fn synthetic_resolved(source_object_index: usize, options: ObjectOptions) -> ResolvedProjectObject {
    resolved(source_object_index, options, vec![region()])
}

fn prepare_pre_closing(
    sources: &[ProjectObject],
    resolved_objects: &[ResolvedProjectObject],
    plans: Vec<PlannedPrintObject>,
    spiral_mode: bool,
) -> Vec<PreClosingPrintObject> {
    let raw = intersections(sources, resolved_objects, plans).unwrap();
    let chained = chain_project_intersections(raw);
    let looped = loop_project_intersections(chained, 2_000_000);
    let modes = apply_project_slicing_modes(looped, resolved_objects, spiral_mode).unwrap();
    apply_project_pre_closing_unions(modes).unwrap()
}

fn separated_boxes_object(index: usize) -> ProjectObject {
    let path = format!("closing-{index}.model");
    object(
        &path,
        100 + u32::try_from(index).unwrap(),
        vec![separated_boxes_volume(200 + u32::try_from(index).unwrap())],
        &[Transform3d::IDENTITY],
    )
}

fn separated_boxes_volume(id: u32) -> ProjectVolume {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for x in [0.0, 1.1] {
        let base = u32::try_from(vertices.len()).unwrap();
        vertices.extend([
            Point3d::new(x, 0.0, 0.0),
            Point3d::new(x + 1.0, 0.0, 0.0),
            Point3d::new(x + 1.0, 1.0, 0.0),
            Point3d::new(x, 1.0, 0.0),
            Point3d::new(x, 0.0, 2.0),
            Point3d::new(x + 1.0, 0.0, 2.0),
            Point3d::new(x + 1.0, 1.0, 2.0),
            Point3d::new(x, 1.0, 2.0),
        ]);
        triangles.extend(
            [
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [1, 2, 6],
                [1, 6, 5],
                [2, 3, 7],
                [2, 7, 6],
                [3, 0, 4],
                [3, 4, 7],
            ]
            .map(|face| face.map(|vertex| base + vertex)),
        );
    }
    mesh_volume(
        id,
        ProjectVolumeType::ModelPart,
        vertices,
        triangles,
        Transform3d::IDENTITY,
    )
}

fn first_layer_counts_pre(objects: &[PreClosingPrintObject]) -> Vec<usize> {
    objects
        .iter()
        .map(|object| object.volumes()[0].layers()[0].expolygons().len())
        .collect()
}

fn first_layer_counts_post(objects: &[PostClosingPrintObject]) -> Vec<usize> {
    objects
        .iter()
        .map(|object| object.volumes()[0].layers()[0].expolygons().len())
        .collect()
}

fn pre_facts(objects: &[PreClosingPrintObject]) -> Vec<ObjectFacts> {
    objects
        .iter()
        .map(|object| ObjectFacts {
            plan: object.plan().clone(),
            volumes: object
                .volumes()
                .iter()
                .map(|volume| VolumeFacts {
                    source_volume_index: volume.source_volume_index(),
                    ordinal: volume.ordinal(),
                    volume_type: volume.volume_type(),
                    layers: volume
                        .layers()
                        .iter()
                        .map(|layer| (layer.mode(), layer.expolygons().to_vec()))
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

fn post_facts(objects: &[PostClosingPrintObject]) -> Vec<ObjectFacts> {
    objects
        .iter()
        .map(|object| ObjectFacts {
            plan: object.plan().clone(),
            volumes: object
                .volumes()
                .iter()
                .map(|volume| VolumeFacts {
                    source_volume_index: volume.source_volume_index(),
                    ordinal: volume.ordinal(),
                    volume_type: volume.volume_type(),
                    layers: volume
                        .layers()
                        .iter()
                        .map(|layer| (layer.mode(), layer.expolygons().to_vec()))
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

fn invalid_radius() -> SliceError {
    SliceError::InvalidInput("invalid Orca option slice_closing_radius".to_owned())
}

fn expolygon(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()),
        Vec::new(),
    )
}
