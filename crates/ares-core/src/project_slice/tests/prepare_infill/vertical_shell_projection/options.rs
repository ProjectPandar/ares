use crate::project_slice::{
    prepare_infill::vertical_shell_projection::{self, GeometryStep},
    tests::support::KsrArchive,
};

use super::fixture;

#[test]
fn task22o20_real_3mf_inactive_modes_have_empty_projection_and_zero_geometry() {
    for mode in ["none", "ensure_critical_only", "ensure_moderate"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
            &format!("\"ensure_vertical_shell_thickness\": \"{mode}\""),
        );
        vertical_shell_projection::reset_geometry_hooks();
        let output = fixture::prepare(archive.bytes());
        assert!(output.projections.iter().all(|object| {
            object
                .records
                .iter()
                .flatten()
                .all(|projection| projection.shell.is_empty() && projection.holes.is_empty())
        }));
        assert!(vertical_shell_projection::geometry_events().is_empty());
    }
}

#[test]
fn task22o20_real_3mf_count_one_zero_thickness_activates_exact_anchor_sites() {
    let mut top = KsrArchive::new();
    top.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_layers\": \"5\"",
        "\"top_shell_layers\": \"1\"",
    );
    top.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_thickness\": \"1\"",
        "\"top_shell_thickness\": \"0\"",
    );
    vertical_shell_projection::reset_geometry_hooks();
    let _ = fixture::prepare(top.bytes());
    assert_eq!(count(GeometryStep::TopAnchorOffset), 459);
    assert_eq!(count(GeometryStep::TopAnchorIntersection), 459);

    let mut bottom = KsrArchive::new();
    bottom.replace_unique(
        "Metadata/project_settings.config",
        "\"bottom_shell_layers\": \"3\"",
        "\"bottom_shell_layers\": \"1\"",
    );
    vertical_shell_projection::reset_geometry_hooks();
    let _ = fixture::prepare(bottom.bytes());
    assert_eq!(count(GeometryStep::BottomAnchorOffset), 459);
    assert_eq!(count(GeometryStep::BottomAnchorIntersection), 459);
}

#[test]
fn task22o20_bottom_thickness_literal_expands_the_real_window() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"bottom_shell_thickness\": \"0\"",
        "\"bottom_shell_thickness\": \"1\"",
    );
    vertical_shell_projection::reset_geometry_hooks();
    let _ = fixture::prepare(archive.bytes());
    assert_eq!(count(GeometryStep::TopVisit), 1_830);
    assert_eq!(count(GeometryStep::BottomVisit), 1_830);
}

#[test]
fn task22o20_model_part_override_beats_inactive_global_mode() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"none\"",
    );
    archive.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"ensure_vertical_shell_thickness\" value=\"ensure_all\"/>",
    );
    vertical_shell_projection::reset_geometry_hooks();
    let output = fixture::prepare(archive.bytes());
    assert!(
        output.projections[0]
            .records
            .iter()
            .flatten()
            .any(|projection| !projection.shell.is_empty())
    );
    assert!(!vertical_shell_projection::geometry_events().is_empty());
}

#[test]
fn task22o20_model_part_shell_overrides_drive_anchor_and_thickness_windows() {
    let mut top = KsrArchive::new();
    top.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"top_shell_layers\" value=\"1\"/>\n      <metadata key=\"top_shell_thickness\" value=\"0\"/>",
    );
    vertical_shell_projection::reset_geometry_hooks();
    let _ = fixture::prepare(top.bytes());
    assert_eq!(count(GeometryStep::TopAnchorOffset), 459);

    let mut bottom_anchor = KsrArchive::new();
    bottom_anchor.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"bottom_shell_layers\" value=\"1\"/>",
    );
    vertical_shell_projection::reset_geometry_hooks();
    let _ = fixture::prepare(bottom_anchor.bytes());
    assert_eq!(count(GeometryStep::BottomAnchorOffset), 459);

    let mut bottom = KsrArchive::new();
    bottom.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"bottom_shell_thickness\" value=\"1\"/>",
    );
    vertical_shell_projection::reset_geometry_hooks();
    let _ = fixture::prepare(bottom.bytes());
    assert_eq!(count(GeometryStep::BottomVisit), 1_830);
}

#[test]
fn task22o20_stage_uses_current_spacing_not_stopped_neighbor_spacing() {
    let (first_index, first) = projection_with_spacings(10_000, 500_000);
    let (same_index, stopped_changed) = projection_with_spacings(10_000, 1_000_000);
    let (changed_index, current_changed) = projection_with_spacings(200_000, 500_000);

    assert_eq!(same_index, first_index);
    assert_eq!(changed_index, first_index);
    assert_eq!(stopped_changed, first);
    assert_ne!(current_changed, first);
}

fn projection_with_spacings(
    current_spacing: i64,
    stopped_spacing: i64,
) -> (usize, Vec<Vec<(i64, i64)>>) {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_layers\": \"5\"",
        "\"top_shell_layers\": \"1\"",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_thickness\": \"1\"",
        "\"top_shell_thickness\": \"0\"",
    );
    let mut input = fixture::prepare_o19(archive.bytes());
    let index = {
        let prelude = &input.predecessor.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let lslices = prelude.object.as_parts().0.as_parts().1;
        input.caches[0]
            .records
            .iter()
            .enumerate()
            .take(input.caches[0].records.len() - 1)
            .find_map(|(index, cache)| {
                cache
                    .as_ref()
                    .filter(|cache| {
                        !cache.top_surfaces.is_empty() && !lslices[index + 1].is_empty()
                    })
                    .map(|_| index)
            })
            .expect("KSR must contain an anchored top path with a stopped neighbor")
    };
    let records = &mut input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records;
    records[index].as_mut().unwrap().external_spacing = current_spacing;
    records[index + 1].as_mut().unwrap().external_spacing = stopped_spacing;
    let output = vertical_shell_projection::prepare(input).unwrap();
    let projection = output.projections[0].records[index].as_ref().unwrap();
    let snapshot = projection
        .shell
        .iter()
        .map(|path| {
            path.points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect();
    (index, snapshot)
}

fn count(step: GeometryStep) -> usize {
    vertical_shell_projection::geometry_events()
        .into_iter()
        .filter(|event| *event == step)
        .count()
}
