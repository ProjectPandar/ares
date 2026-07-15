use crate::{
    OrcaBool, OrcaInt, Point2d, Point2dList, ProcessTimelapseType, ProjectObject, ProjectSettings,
    ProjectVolumeType,
    project::{effective_config::usage::has_wipe_tower, transform::Transform3d},
};

use super::{collect, object_options, resolved_object, settings, source_object, volume};

#[test]
fn explicit_wipe_predicate_covers_wrapping_timelapse_spiral_and_selector_routes() {
    let cases = [
        (
            true,
            true,
            3,
            ProcessTimelapseType::Traditional,
            true,
            3,
            true,
        ),
        (
            true,
            true,
            2,
            ProcessTimelapseType::Traditional,
            true,
            3,
            false,
        ),
        (true, false, 0, ProcessTimelapseType::Smooth, true, 3, true),
        (
            true,
            false,
            0,
            ProcessTimelapseType::Traditional,
            false,
            3,
            true,
        ),
        (
            true,
            false,
            0,
            ProcessTimelapseType::Traditional,
            true,
            3,
            false,
        ),
        (
            false,
            true,
            3,
            ProcessTimelapseType::Smooth,
            false,
            3,
            false,
        ),
        (true, false, 0, ProcessTimelapseType::Smooth, true, 0, false),
    ];

    for (tower, wrapping, points, timelapse, spiral, selector, adds_wipe) in cases {
        let source_settings = settings(3);
        let wipe_settings = wipe_settings(
            settings(3),
            (tower, wrapping, points, timelapse, spiral, selector),
        );
        let objects = [two_raw_filaments()];
        let resolved = [resolved_object(
            object_options(&source_settings),
            Vec::new(),
        )];
        let usage = collect(&source_settings, &wipe_settings, &objects, &resolved);
        let expected = if adds_wipe { vec![0, 1, 2] } else { vec![0, 1] };

        assert_eq!(usage.supported_used_filaments, expected);
    }
}

#[test]
fn wipe_len_gate_observes_cross_vector_duplicates_before_final_dedup() {
    let source_settings = settings(2);
    let wipe_settings = wipe_settings(
        settings(2),
        (true, false, 0, ProcessTimelapseType::Smooth, true, 2),
    );
    let objects = [one_raw_filament()];
    let mut options = object_options(&source_settings);
    options.enable_support = OrcaBool(true);
    options.support_filament = OrcaInt(1);
    options.support_interface_filament = OrcaInt(1);
    let resolved = [resolved_object(options, Vec::new())];

    assert_eq!(
        collect(&source_settings, &wipe_settings, &objects, &resolved).supported_used_filaments,
        vec![0, 1]
    );
}

#[test]
fn current_support_appends_the_object_vector_before_the_wipe_len_gate() {
    let source_settings = settings(2);
    let wipe_settings = wipe_settings(
        settings(2),
        (true, false, 0, ProcessTimelapseType::Smooth, true, 2),
    );
    let objects = [one_raw_filament()];
    let mut options = object_options(&source_settings);
    options.enable_support = OrcaBool(true);
    options.support_filament = OrcaInt(0);
    options.support_interface_filament = OrcaInt(0);
    let resolved = [resolved_object(options, Vec::new())];

    assert_eq!(
        collect(&source_settings, &wipe_settings, &objects, &resolved).supported_used_filaments,
        vec![0, 1]
    );
}

#[test]
fn support_vector_is_deduplicated_before_the_wipe_len_gate() {
    let source_settings = settings(2);
    let wipe_settings = wipe_settings(
        settings(2),
        (true, false, 0, ProcessTimelapseType::Smooth, true, 2),
    );
    let objects = [source_object(
        Default::default(),
        Default::default(),
        Vec::new(),
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let mut options = object_options(&source_settings);
    options.enable_support = OrcaBool(true);
    options.support_filament = OrcaInt(1);
    options.support_interface_filament = OrcaInt(1);
    let resolved = [resolved_object(options, Vec::new())];

    assert_eq!(
        collect(&source_settings, &wipe_settings, &objects, &resolved).supported_used_filaments,
        vec![0]
    );
}

#[test]
fn wipe_predicate_and_selector_read_the_separately_supplied_phase_snapshot() {
    let mut source_settings = settings(3);
    source_settings.process.print.enable_prime_tower = OrcaBool(false);
    source_settings.process.print.wipe_tower_filament = OrcaInt(0);
    source_settings.process.print.spiral_mode = OrcaBool(true);
    let wipe_settings = wipe_settings(
        settings(3),
        (true, false, 0, ProcessTimelapseType::Smooth, true, 3),
    );
    let objects = [two_raw_filaments()];
    let resolved = [resolved_object(
        object_options(&source_settings),
        Vec::new(),
    )];

    assert_eq!(
        collect(&source_settings, &wipe_settings, &objects, &resolved).supported_used_filaments,
        vec![0, 1, 2]
    );
}

#[test]
fn non_spiral_wipe_requires_more_than_one_logical_filament() {
    let one_filament = wipe_settings(
        settings(1),
        (true, false, 0, ProcessTimelapseType::Traditional, false, 0),
    );
    let two_filaments = wipe_settings(
        settings(2),
        (true, false, 0, ProcessTimelapseType::Traditional, false, 0),
    );

    assert!(!has_wipe_tower(&one_filament, 1));
    assert!(has_wipe_tower(&two_filaments, 2));
}

#[test]
fn object_vector_is_deduplicated_before_the_wipe_len_gate() {
    let source_settings = settings(2);
    let wipe_settings = wipe_settings(
        settings(2),
        (true, false, 0, ProcessTimelapseType::Smooth, true, 2),
    );
    let objects = [source_object(
        Default::default(),
        Default::default(),
        vec![
            volume(ProjectVolumeType::ModelPart, Some(1), 0.0, true),
            volume(ProjectVolumeType::ParameterModifier, Some(1), 0.0, true),
        ],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )];
    let resolved = [resolved_object(
        object_options(&source_settings),
        Vec::new(),
    )];

    assert_eq!(
        collect(&source_settings, &wipe_settings, &objects, &resolved).supported_used_filaments,
        vec![0]
    );
}

fn wipe_settings(
    mut settings: ProjectSettings,
    (tower, wrapping, points, timelapse, spiral, selector): (
        bool,
        bool,
        usize,
        ProcessTimelapseType,
        bool,
        i32,
    ),
) -> ProjectSettings {
    settings.process.print.enable_prime_tower = OrcaBool(tower);
    settings.process.gcode.enable_wrapping_detection = OrcaBool(wrapping);
    settings.printer.gcode.wrapping_exclude_area = Point2dList(
        (0..points)
            .map(|point| Point2d::new(point as f64, point as f64))
            .collect(),
    );
    settings.process.print.timelapse_type = timelapse;
    settings.process.print.spiral_mode = OrcaBool(spiral);
    settings.process.print.wipe_tower_filament = OrcaInt(selector);
    settings
}

fn two_raw_filaments() -> ProjectObject {
    source_object(
        Default::default(),
        Default::default(),
        vec![
            volume(ProjectVolumeType::ModelPart, Some(1), 0.0, true),
            volume(ProjectVolumeType::ParameterModifier, Some(2), 0.0, true),
        ],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )
}

fn one_raw_filament() -> ProjectObject {
    source_object(
        Default::default(),
        Default::default(),
        vec![volume(ProjectVolumeType::ModelPart, Some(1), 0.0, true)],
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )
}
