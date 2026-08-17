use std::panic::catch_unwind;

use super::super::super::super::support::metadata;
use super::*;
use crate::{OrcaBool, OrcaFloat, OrcaFloats, Percent, SliceError, slice_project};

#[test]
fn task22n_flow_option_pairs_cover_bridge_branches() {
    bridge_pair(
        "bridge-auto",
        (
            &[process(r#""bridge_flow": "1""#, r#""bridge_flow": "0.8""#)],
            process(
                r#""bridge_line_width": "100%""#,
                r#""bridge_line_width": "0""#,
            ),
        ),
        Key::BridgeWidth,
        [percent(100.0), absolute(0.0)],
        change(
            BOTH_LAYERS,
            OVERHANG_ROLE,
            NONTHICK_SHRINK,
            NONTHICK_AUTO_WIDTH,
        ),
    );
    bridge_pair(
        "bridge-grow",
        (
            &[],
            process(r#""bridge_flow": "1""#, r#""bridge_flow": "1.4""#),
        ),
        Key::BridgeFlow,
        [float(1.0), float(1.4)],
        change(
            BOTH_LAYERS,
            OVERHANG_ROLE,
            PERCENT_100_NOZZLE_04,
            NONTHICK_GROW,
        ),
    );
    bridge_pair(
        "bridge-shrink",
        (
            &[],
            process(r#""bridge_flow": "1""#, r#""bridge_flow": "0.8""#),
        ),
        Key::BridgeFlow,
        [float(1.0), float(0.8)],
        change(
            BOTH_LAYERS,
            OVERHANG_ROLE,
            PERCENT_100_NOZZLE_04,
            NONTHICK_SHRINK,
        ),
    );
    bridge_pair(
        "bridge-round",
        (
            &[],
            process(r#""bridge_flow": "1""#, r#""bridge_flow": "0.2""#),
        ),
        Key::BridgeFlow,
        [float(1.0), float(0.2)],
        change(
            BOTH_LAYERS,
            OVERHANG_ROLE,
            PERCENT_100_NOZZLE_04,
            NONTHICK_ROUND,
        ),
    );
    run(OptionPair {
        name: "bridge-epsilon",
        setup: &[INITIAL_ZERO],
        delta: process(r#""bridge_flow": "1""#, r#""bridge_flow": "1.0005""#),
        key: Key::BridgeFlow,
        raw: [float(1.0), float(1.0005)],
        effective: [float(1.0), float(1.0005)],
        changes: &[],
    });
    bridge_pair(
        "thick-configured",
        (
            &[
                process(
                    r#""bridge_line_width": "100%""#,
                    r#""bridge_line_width": "120%""#,
                ),
                process(r#""bridge_flow": "1""#, r#""bridge_flow": "1.44""#),
            ],
            process(r#""thick_bridges": "0""#, r#""thick_bridges": "1""#),
        ),
        Key::Thick,
        [Value::Bool(false), Value::Bool(true)],
        change(
            BOTH_LAYERS,
            OVERHANG_ROLE,
            NONTHICK_PERCENT_120_RATIO_144,
            THICK_PERCENT_120_RATIO_144,
        ),
    );
    bridge_pair(
        "thick-auto",
        (
            &[
                process(
                    r#""bridge_line_width": "100%""#,
                    r#""bridge_line_width": "0""#,
                ),
                process(r#""bridge_flow": "1""#, r#""bridge_flow": "0.64""#),
            ],
            process(r#""thick_bridges": "0""#, r#""thick_bridges": "1""#),
        ),
        Key::Thick,
        [Value::Bool(false), Value::Bool(true)],
        change(
            BOTH_LAYERS,
            OVERHANG_ROLE,
            NONTHICK_AUTO_RATIO_064,
            THICK_AUTO_RATIO_064,
        ),
    );
}

#[test]
fn task22n_canonical_increase_else_is_reached_from_one_3mf_option_delta() {
    let mut setup = ArchiveBuilder::new();
    setup.replace_all("3D/Objects/task22n_box.model", r#"z="0.4""#, r#"z="18.5""#);
    for (from, to) in [
        (r#""layer_height": "0.2""#, r#""layer_height": "9.2289915""#),
        (
            r#""initial_layer_print_height": "0.2""#,
            r#""initial_layer_print_height": "9.2289915""#,
        ),
        (
            r#""initial_layer_line_width": "0.5""#,
            r#""initial_layer_line_width": "1000%""#,
        ),
        (r#""line_width": "0.42""#, r#""line_width": "1000%""#),
        (
            r#""inner_wall_line_width": "0.45""#,
            r#""inner_wall_line_width": "1000%""#,
        ),
        (
            r#""outer_wall_line_width": "0.42""#,
            r#""outer_wall_line_width": "1000%""#,
        ),
        (
            r#""internal_solid_infill_line_width": "0.42""#,
            r#""internal_solid_infill_line_width": "1000%""#,
        ),
        (
            r#""bridge_line_width": "100%""#,
            r#""bridge_line_width": "0""#,
        ),
    ] {
        setup.replace_unique(PROCESS, from, to);
    }
    setup.replace_unique(
        PROCESS,
        "\t\"nozzle_diameter\": [\r\n\t\t\"0.4\",\r\n\t\t\"0.4\"\r\n\t]",
        "\t\"nozzle_diameter\": [\r\n\t\t\"52.83409\",\r\n\t\t\"52.83409\"\r\n\t]",
    );

    let before_archive = setup.clone().bytes();
    let mut after_setup = setup;
    let delta = process(r#""bridge_flow": "1""#, r#""bridge_flow": "1.0000001""#);
    after_setup.replace_unique(delta.path, delta.from, delta.to);
    let after_archive = after_setup.bytes();
    assert_single_entry_replacement(
        &before_archive,
        &after_archive,
        delta.path,
        delta.from,
        delta.to,
    );
    assert_ne!(
        semantic_identity(&before_archive),
        semantic_identity(&after_archive)
    );
    assert_eq!(
        [
            loaded(&before_archive, Key::BridgeFlow),
            loaded(&after_archive, Key::BridgeFlow),
        ],
        [
            (float(1.0), float(1.0)),
            (float(1.0000001), float(1.0000001)),
        ]
    );

    assert_eq!(
        task22n_browser_input_oracle(&before_archive).unwrap(),
        task22n_browser_input_oracle(&after_archive).unwrap()
    );
    let before_output = task22n_browser_oracle(&before_archive).unwrap();
    let after_output = task22n_browser_oracle(&after_archive).unwrap();
    let before_frame = parse_n(&before_output).unwrap();
    let after_frame = parse_n(&after_output).unwrap();
    let ([before_object], [after_object]) = (
        before_frame.objects.as_slice(),
        after_frame.objects.as_slice(),
    ) else {
        panic!("one object")
    };
    assert_eq!(
        (before_object.slots.len(), after_object.slots.len()),
        (2, 2)
    );
    for (before, after) in before_object.slots.iter().zip(&after_object.slots) {
        let (Some(before), Some(after)) = (before, after) else {
            panic!("two populated slots")
        };
        assert_context(before, after, "canonical increase-else");
        for role in [0, 1, 3] {
            assert_eq!(flow(before.flows[role]), flow(after.flows[role]));
        }
        assert_eq!(
            flow(before.flows[2]),
            bits(
                [0x440415d2, 0x4113a9f3, 0x44039711, 0x4253561c],
                false,
                f64::from(f32::from_bits(0x4597ce34)).to_bits(),
            )
        );
        assert_eq!(
            flow(after.flows[2]),
            bits(
                [0x440415d1, 0x4113a9f3, 0x44039710, 0x4253561c],
                false,
                0x40b2f9c660000000,
            )
        );
    }
}

#[tokio::test]
async fn task22n_real_3mf_release_decrease_rounding_is_transactional_without_panic() {
    let archive = release_decrease_rounding_archive();
    let project = crate::load_project(&archive).unwrap();
    let raw = project.settings();
    assert_eq!(
        [
            raw.process.object.layer_height,
            raw.process.print.initial_layer_print_height,
        ],
        [OrcaFloat(2e-7); 2]
    );
    assert_eq!(
        [
            raw.process.print.initial_layer_line_width,
            raw.process.region.inner_wall_line_width,
        ],
        [FloatOrPercent::Percent(Percent(500.0)); 2]
    );
    assert_eq!(
        (
            raw.process.region.bridge_line_width,
            raw.process.region.bridge_flow,
            raw.process.object.thick_bridges,
            &raw.project.print.nozzle_diameter,
        ),
        (
            FloatOrPercent::Float(0.0),
            OrcaFloat(f64::MIN_POSITIVE),
            OrcaBool(false),
            &OrcaFloats(vec![OrcaFloat(100.0); 2]),
        )
    );
    let resolved = resolve_bounded_project_config(&project).unwrap();
    let [object] = resolved.objects.as_slice() else {
        panic!("one resolved object")
    };
    let [candidate] = object.layer_candidates.as_slice() else {
        panic!("one layer candidate")
    };
    let [part] = candidate.model_parts.as_slice() else {
        panic!("one model part")
    };
    assert_eq!(
        [
            object.object.layer_height,
            resolved.views.full.process.print.initial_layer_print_height,
        ],
        [OrcaFloat(2e-7); 2]
    );
    assert_eq!(object.object.thick_bridges, OrcaBool(false));
    assert_eq!(
        resolved.views.full.process.print.initial_layer_line_width,
        FloatOrPercent::Percent(Percent(500.0))
    );
    assert_eq!(
        part.region.inner_wall_line_width,
        FloatOrPercent::Percent(Percent(500.0))
    );
    assert_eq!(part.region.bridge_line_width, FloatOrPercent::Float(0.0));
    assert_eq!(part.region.bridge_flow, OrcaFloat(f64::MIN_POSITIVE));
    assert_eq!(
        resolved.views.full.project.print.nozzle_diameter,
        OrcaFloats(vec![OrcaFloat(100.0); 2])
    );

    let expected = SliceError::InvalidInput("invalid Orca option bridge_flow".to_owned());
    let predecessor = task22n_browser_input_oracle(&archive).unwrap();
    let checkpoint = catch_unwind(|| task22n_browser_oracle(&archive));
    let Ok(checkpoint) = checkpoint else {
        panic!("real 3MF Task 22N checkpoint must not panic")
    };
    assert_eq!(checkpoint.unwrap_err(), expected);
    assert_eq!(task22n_browser_input_oracle(&archive).unwrap(), predecessor);

    let public = tokio::spawn(slice_project(archive.clone(), metadata())).await;
    let Ok(public) = public else {
        panic!("public slice_project must not panic")
    };
    assert_eq!(public.unwrap_err(), expected);
    assert_eq!(task22n_browser_input_oracle(&archive).unwrap(), predecessor);
}

fn release_decrease_rounding_archive() -> Vec<u8> {
    let mut archive = ArchiveBuilder::new();
    archive.replace_all("3D/Objects/task22n_box.model", r#"z="0.4""#, r#"z="2e-7""#);
    for (from, to) in [
        (r#""layer_height": "0.2""#, r#""layer_height": "2e-7""#),
        (
            r#""initial_layer_print_height": "0.2""#,
            r#""initial_layer_print_height": "2e-7""#,
        ),
        (
            r#""initial_layer_line_width": "0.5""#,
            r#""initial_layer_line_width": "500%""#,
        ),
        (
            r#""inner_wall_line_width": "0.45""#,
            r#""inner_wall_line_width": "500%""#,
        ),
        (
            r#""bridge_line_width": "100%""#,
            r#""bridge_line_width": "0""#,
        ),
        (
            r#""bridge_flow": "1""#,
            r#""bridge_flow": "2.2250738585072014e-308""#,
        ),
    ] {
        archive.replace_unique(PROCESS, from, to);
    }
    archive.replace_unique(
        PROCESS,
        "\t\"nozzle_diameter\": [\r\n\t\t\"0.4\",\r\n\t\t\"0.4\"\r\n\t]",
        "\t\"nozzle_diameter\": [\r\n\t\t\"100\",\r\n\t\t\"100\"\r\n\t]",
    );
    archive.bytes()
}

fn bridge_pair(
    name: &str,
    (setup, delta): (&[Edit], Edit),
    key: Key,
    values: [Value; 2],
    change: Change,
) {
    let mut common = Vec::with_capacity(setup.len() + 1);
    common.push(INITIAL_ZERO);
    common.extend_from_slice(setup);
    run(OptionPair {
        name,
        setup: &common,
        delta,
        key,
        raw: values,
        effective: values,
        changes: &[change],
    });
}
