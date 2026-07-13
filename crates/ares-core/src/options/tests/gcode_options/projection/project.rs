use super::*;

#[test]
fn gcode_options_projection_preserves_each_project_field() {
    assert_project_projection!(
        deretraction_speed,
        OrcaFloats(vec![OrcaFloat(9401.01), OrcaFloat(9402.02)])
    );
    assert_project_projection!(
        filament_ids,
        OrcaStrings(vec!["project-filament-a".into(), "project-filament-b".into()])
    );
    assert_project_projection!(filament_map_mode, ProjectFilamentMapMode::Manual);
    assert_project_projection!(filament_map, OrcaInts(vec![OrcaInt(9403), OrcaInt(9404)]));
    assert_project_projection!(
        retract_before_wipe,
        OrcaPercents(vec![Percent(9405.05), Percent(9406.06)])
    );
    assert_project_projection!(
        retraction_length,
        OrcaFloats(vec![OrcaFloat(9407.07), OrcaFloat(9408.08)])
    );
    assert_project_projection!(
        retract_length_toolchange,
        OrcaFloats(vec![OrcaFloat(9409.09), OrcaFloat(9410.10)])
    );
    assert_project_projection!(
        z_hop,
        OrcaFloats(vec![OrcaFloat(9411.11), OrcaFloat(9412.12)])
    );
    assert_project_projection!(
        retract_lift_above,
        OrcaFloats(vec![OrcaFloat(9413.13), OrcaFloat(9414.14)])
    );
    assert_project_projection!(
        retract_lift_below,
        OrcaFloats(vec![OrcaFloat(9415.15), OrcaFloat(9416.16)])
    );
    assert_project_projection!(
        retract_restart_extra,
        OrcaFloats(vec![OrcaFloat(9417.17), OrcaFloat(9418.18)])
    );
    assert_project_projection!(
        retract_restart_extra_toolchange,
        OrcaFloats(vec![OrcaFloat(9419.19), OrcaFloat(9420.20)])
    );
    assert_project_projection!(
        retraction_speed,
        OrcaFloats(vec![OrcaFloat(9421.21), OrcaFloat(9422.22)])
    );
    assert_project_projection!(
        nozzle_volume_type,
        NozzleVolumeTypes(vec![NozzleVolumeType::HighFlow])
    );
    assert_project_projection!(
        extruder_ams_count,
        AmsCounts(vec!["project-ams-a".into(), "project-ams-b".into()])
    );
    assert_project_projection!(bbl_calib_mark_logo, OrcaBool(false));
    assert_project_projection!(has_scarf_joint_seam, OrcaBool(true));
}
