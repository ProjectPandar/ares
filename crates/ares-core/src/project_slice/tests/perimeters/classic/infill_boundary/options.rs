use crate::project_slice::perimeters::{
    classic::{
        PreparedPostClassicInfillBoundary,
        gap_extrusion::PreparedPostClassicGapExtrusion,
        infill_boundary::{self, GeometryStep},
        traversal::PendingPathBranch,
    },
    prepare_post_classic_gap_extrusion, prepare_post_classic_infill_boundary,
};

use super::super::super::super::support::KsrArchive;

const CONFIG: &str = "Metadata/project_settings.config";
const NORMAL_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"256x0\",\r\n",
    "\t\t\"256x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);
const LARGE_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"2148x0\",\r\n",
    "\t\t\"2148x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);

#[test]
fn task22o15_overlap_options_come_from_typed_3mf_records() {
    let baseline =
        overlaps(&prepare_post_classic_infill_boundary(&KsrArchive::new().bytes()).unwrap());

    let mut ordinary = KsrArchive::new();
    ordinary.replace_unique(
        CONFIG,
        "\"infill_wall_overlap\": \"15%\"",
        "\"infill_wall_overlap\": \"5%\"",
    );
    let ordinary = overlaps(&prepare_post_classic_infill_boundary(&ordinary.bytes()).unwrap());
    assert_ne!(baseline, ordinary);

    let mut top = KsrArchive::new();
    top.replace_unique(
        CONFIG,
        "\"top_bottom_infill_wall_overlap\": \"25%\"",
        "\"top_bottom_infill_wall_overlap\": \"5%\"",
    );
    let top = overlaps(&prepare_post_classic_infill_boundary(&top.bytes()).unwrap());
    assert_ne!(baseline, top);
}

#[test]
fn task22o15_true_extra_perimeter_option_stays_inactive_for_aligned_false_operands() {
    for (from, to) in [
        (
            "\"detect_overhang_wall\": \"1\"",
            "\"detect_overhang_wall\": \"0\"",
        ),
        ("\"wall_loops\": \"2\"", "\"wall_loops\": \"0\""),
    ] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            CONFIG,
            "\"extra_perimeters_on_overhangs\": \"0\"",
            "\"extra_perimeters_on_overhangs\": \"1\"",
        );
        archive.replace_unique(CONFIG, from, to);
        infill_boundary::reset_geometry_hooks();
        prepare_post_classic_infill_boundary(&archive.bytes()).unwrap();
        assert_guard_event_order(&infill_boundary::geometry_events());
    }
}

#[test]
fn task22o15_aligned_guard_accepts_missing_lower_and_layer_at_raft_independently() {
    for falsifier in [GuardFalsifier::MissingLower, GuardFalsifier::LayerAtRaft] {
        let mut source = source_with_true_extra_perimeters_and_false_detection();
        align_single_guard_record(&mut source, falsifier);
        infill_boundary::reset_geometry_hooks();
        infill_boundary::finish(source).unwrap();
        assert_guard_event_order(&infill_boundary::geometry_events());
    }
}

#[test]
fn task22o15_negative_and_fractional_overlap_options_reach_typed_stage() {
    for value in ["-5%", "12.345%"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            CONFIG,
            "\"infill_wall_overlap\": \"15%\"",
            &format!("\"infill_wall_overlap\": \"{value}\""),
        );
        let output = prepare_post_classic_infill_boundary(&archive.bytes()).unwrap();
        let values = overlaps(&output);
        assert!(values.iter().any(|value| value.1 != 0));
        if value.starts_with('-') {
            assert!(values.iter().any(|value| value.1 < 0));
        }
    }
}

#[test]
fn task22o15_raw_resolution_ignores_arc_fitting_adjustment() {
    let arc = prepare_post_classic_infill_boundary(&KsrArchive::new().bytes()).unwrap();
    let mut no_arc = KsrArchive::new();
    no_arc.replace_unique(
        CONFIG,
        "\"enable_arc_fitting\": \"1\"",
        "\"enable_arc_fitting\": \"0\"",
    );
    let no_arc = prepare_post_classic_infill_boundary(&no_arc.bytes()).unwrap();

    assert_eq!(scaled_resolutions(&arc), scaled_resolutions(&no_arc));
    assert!(
        scaled_resolutions(&arc)
            .iter()
            .all(|&value| value == 12_000.0)
    );
    assert_eq!(first_prelude_resolution(&arc), 2_400.0);
    assert_eq!(first_prelude_resolution(&no_arc), 12_000.0);
}

#[test]
fn task22o15_large_bed_uses_raw_scale_specific_resolution() {
    let normal = prepare_post_classic_infill_boundary(&KsrArchive::new().bytes()).unwrap();
    let mut large = KsrArchive::new();
    large.replace_unique(CONFIG, NORMAL_PRINTABLE_AREA, LARGE_PRINTABLE_AREA);
    let large = prepare_post_classic_infill_boundary(&large.bytes()).unwrap();
    assert_eq!(normal.predecessor.scale.factor(), 0.000_001);
    assert_eq!(large.predecessor.scale.factor(), 0.000_01);
    assert!(
        scaled_resolutions(&normal)
            .iter()
            .all(|&value| value == 12_000.0)
    );
    assert!(
        scaled_resolutions(&large)
            .iter()
            .all(|&value| value == 1_200.0)
    );
}

fn overlaps(prepared: &PreparedPostClassicInfillBoundary) -> Vec<(i64, i64, i64)> {
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.overlap)
        .map(|overlap| {
            (
                overlap.inset,
                overlap.infill_peri_overlap,
                overlap.top_infill_peri_overlap,
            )
        })
        .collect()
}

fn scaled_resolutions(prepared: &PreparedPostClassicInfillBoundary) -> Vec<f64> {
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.overlap)
        .map(|overlap| overlap.scaled_resolution)
        .collect()
}

fn first_prelude_resolution(prepared: &PreparedPostClassicInfillBoundary) -> f64 {
    prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records
        .iter()
        .flatten()
        .next()
        .unwrap()
        .surface_simplify_resolution
}

#[derive(Clone, Copy)]
enum GuardFalsifier {
    MissingLower,
    LayerAtRaft,
}

fn source_with_true_extra_perimeters_and_false_detection() -> PreparedPostClassicGapExtrusion {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        "\"extra_perimeters_on_overhangs\": \"0\"",
        "\"extra_perimeters_on_overhangs\": \"1\"",
    );
    archive.replace_unique(
        CONFIG,
        "\"detect_overhang_wall\": \"1\"",
        "\"detect_overhang_wall\": \"0\"",
    );
    prepare_post_classic_gap_extrusion(&archive.bytes()).unwrap()
}

fn align_single_guard_record(
    source: &mut PreparedPostClassicGapExtrusion,
    falsifier: GuardFalsifier,
) {
    let object = &mut source.predecessor.objects[0];
    let index = object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records
        .iter()
        .position(|record| {
            record.as_ref().is_some_and(|record| match falsifier {
                GuardFalsifier::MissingLower => {
                    record.layer_id > 0 && record.lower_layer_index.is_some()
                }
                GuardFalsifier::LayerAtRaft => record.layer_id == 0,
            })
        })
        .unwrap();

    let layer_id = {
        let prelude = &object.predecessor.predecessor.predecessor.predecessor;
        let input = prelude.object.records[index].as_ref().unwrap();
        let options = prelude.object.region_options(input);
        assert!(options.extra_perimeters_on_overhangs.0);
        assert!(options.wall_loops.0 > 0);
        assert!(!input.spiral_mode);
        input.layer_id
    };
    {
        let input = object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .records[index]
            .as_mut()
            .unwrap();
        match falsifier {
            GuardFalsifier::MissingLower => input.lower_layer_index = None,
            GuardFalsifier::LayerAtRaft => input.lower_layer_index = Some(0),
        }
    }
    object.records[index].as_mut().unwrap().branch =
        PendingPathBranch::from_operands(true, layer_id, 0);

    let input = object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records[index]
        .as_ref()
        .unwrap();
    assert_guard_is_independently_inactive(
        input,
        object.records[index].as_ref().unwrap().branch,
        falsifier,
    );
}

fn assert_guard_is_independently_inactive(
    input: &crate::project_slice::perimeters::types::PerimeterInputRecord,
    branch: PendingPathBranch,
    falsifier: GuardFalsifier,
) {
    let (detect, layer_id, raft_layers) = match branch {
        PendingPathBranch::OverhangClipping {
            detect_overhang_wall,
            layer_id,
            raft_layers,
        }
        | PendingPathBranch::OrdinaryUnsplit {
            detect_overhang_wall,
            layer_id,
            raft_layers,
        } => (detect_overhang_wall, layer_id, raft_layers),
    };
    assert!(detect);
    assert_eq!(input.layer_id, layer_id);
    match falsifier {
        GuardFalsifier::MissingLower => {
            assert!(input.lower_layer_index.is_none());
            assert!(i32::try_from(layer_id).unwrap() > raft_layers);
        }
        GuardFalsifier::LayerAtRaft => {
            assert!(input.lower_layer_index.is_some());
            assert!(i32::try_from(layer_id).unwrap() <= raft_layers);
        }
    }
}

fn assert_guard_event_order(events: &[GeometryStep]) {
    assert!(events.windows(2).any(|window| {
        window
            == [
                GeometryStep::SurfaceAppend,
                GeometryStep::ExtraPerimeterGuard,
            ]
    }));
    assert!(events.windows(2).any(|window| {
        window[0] == GeometryStep::ExtraPerimeterGuard
            && matches!(
                window[1],
                GeometryStep::NoOverlapOne | GeometryStep::NoOverlapTwo
            )
    }));
}
