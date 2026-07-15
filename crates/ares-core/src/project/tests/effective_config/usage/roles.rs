use crate::{OrcaFloat, OrcaInt, Percent, ProcessBrimType, ProjectVolumeType};

use super::{
    base_region, collect, layer_ranges, object_options, printable_source, resolve_candidates,
    resolved_object, settings, source_object, volume, z_translation,
};

#[test]
fn each_region_role_predicate_uses_its_distinct_selector() {
    let cases = [
        (0, 0.0, 0, 0, vec![]),
        (1, 0.0, 0, 0, vec![0]),
        (2, 0.0, 0, 0, vec![0, 1]),
        (0, 1.0, 0, 0, vec![2, 3]),
        (0, 0.0, 1, 0, vec![3, 4]),
        (0, 0.0, 0, 1, vec![3, 5]),
    ];

    for (wall_loops, density, top, bottom, expected) in cases {
        let settings = settings(6);
        let mut region = base_region(&settings);
        region.wall_loops = OrcaInt(wall_loops);
        region.sparse_infill_density = Percent(density);
        region.top_shell_layers = OrcaInt(top);
        region.bottom_shell_layers = OrcaInt(bottom);
        region.outer_wall_filament_id = OrcaInt(1);
        region.inner_wall_filament_id = OrcaInt(2);
        region.sparse_infill_filament_id = OrcaInt(3);
        region.internal_solid_filament_id = OrcaInt(4);
        region.top_surface_filament_id = OrcaInt(5);
        region.bottom_surface_filament_id = OrcaInt(6);
        let objects = [printable_source()];
        let resolved = [resolved_object(object_options(&settings), vec![region])];

        assert_eq!(
            collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
            expected
        );
    }
}

#[test]
fn supported_brim_respects_type_width_and_same_object_raft() {
    let cases = [
        (ProcessBrimType::AutoBrim, 0.0, 0, vec![5]),
        (ProcessBrimType::BrimEars, 1.0, 0, vec![5]),
        (ProcessBrimType::Painted, 1.0, 0, vec![5]),
        (ProcessBrimType::OuterOnly, 1.0, 0, vec![5]),
        (ProcessBrimType::InnerOnly, 1.0, 0, vec![5]),
        (ProcessBrimType::OuterAndInner, 1.0, 0, vec![5]),
        (ProcessBrimType::OuterOnly, 0.0, 0, vec![]),
        (ProcessBrimType::NoBrim, 1.0, 0, vec![]),
        (ProcessBrimType::AutoBrim, 0.0, -1, vec![5]),
        (ProcessBrimType::AutoBrim, 0.0, 1, vec![]),
    ];

    for (brim_type, brim_width, raft_layers, expected) in cases {
        let settings = settings(6);
        let mut options = object_options(&settings);
        options.brim_type = brim_type;
        options.brim_width = OrcaFloat(brim_width);
        options.raft_layers = OrcaInt(raft_layers);
        let mut region = base_region(&settings);
        region.outer_wall_filament_id = OrcaInt(6);
        let objects = [printable_source()];
        let resolved = [resolved_object(options, vec![region])];

        assert_eq!(
            collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
            expected
        );
    }
}

#[test]
fn valid_brim_on_another_object_makes_the_flag_print_wide() {
    let settings = settings(3);
    let mut rafted = object_options(&settings);
    rafted.brim_type = ProcessBrimType::AutoBrim;
    rafted.raft_layers = OrcaInt(1);
    let mut valid = object_options(&settings);
    valid.brim_type = ProcessBrimType::OuterOnly;
    valid.brim_width = OrcaFloat(1.0);
    let mut first_region = base_region(&settings);
    first_region.outer_wall_filament_id = OrcaInt(1);
    let mut second_region = base_region(&settings);
    second_region.outer_wall_filament_id = OrcaInt(2);
    let objects = [printable_source(), printable_source()];
    let resolved = [
        resolved_object(rafted, vec![first_region]),
        resolved_object(valid, vec![second_region]),
    ];

    assert_eq!(
        collect(&settings, &settings, &objects, &resolved).supported_used_filaments,
        vec![0, 1]
    );
}

#[test]
fn reverse_input_usage_keeps_only_the_first_sorted_group_feature_selector() {
    let mut settings = settings(3);
    settings.process.region.wall_loops = OrcaInt(1);
    settings.process.region.sparse_infill_density = Percent(0.0);
    settings.process.region.top_shell_layers = OrcaInt(0);
    settings.process.region.bottom_shell_layers = OrcaInt(0);
    let objects = [source_object(
        Default::default(),
        Default::default(),
        vec![volume(ProjectVolumeType::ModelPart, None, 0.0, true)],
        vec![
            z_translation(100.0),
            crate::project::transform::Transform3d::IDENTITY,
        ],
        layer_ranges(
            r#"<range min_z="0" max_z="1"><option opt_key="outer_wall_filament_id">2</option></range><range min_z="100" max_z="101"><option opt_key="outer_wall_filament_id">3</option></range>"#,
        ),
    )];
    let resolved = resolve_candidates(&settings, 3, &objects);
    let usage = collect(&settings, &settings, &objects, &resolved);

    assert_eq!(usage.supported_used_filaments, vec![0, 1]);
    assert!(!usage.supported_used_filaments.contains(&2));
}
