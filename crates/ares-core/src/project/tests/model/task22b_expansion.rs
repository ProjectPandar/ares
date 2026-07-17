use crate::{Project, SliceError, load_project};

use super::fixture::ProjectParts;

const CYCLE_ERROR: &str = "invalid project model graph: component graph contains a cycle";

fn mesh(id: u32) -> String {
    format!(
        r#"<object id="{id}" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object>"#
    )
}

fn group(id: u32, children: impl IntoIterator<Item = u32>) -> String {
    let components = children
        .into_iter()
        .map(|child| format!(r#"<component objectid="{child}"/>"#))
        .collect::<String>();
    format!(r#"<object id="{id}" type="model"><components>{components}</components></object>"#)
}

fn model(resources: &str, build_ids: &[u32]) -> String {
    let build = build_ids
        .iter()
        .map(|id| format!(r#"<item objectid="{id}"/>"#))
        .collect::<String>();
    format!(
        r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>{resources}</resources><build>{build}</build></model>"#
    )
}

fn project(
    resources: &str,
    build_ids: &[u32],
    metadata_ids: &[u32],
) -> Result<Project, SliceError> {
    let mut parts = ProjectParts::valid();
    parts.make_single_model(&model(resources, build_ids));
    parts.set_model_settings_objects("", metadata_ids);
    load_project(parts.bytes())
}

#[test]
fn task22b_component_cycle_preflight_is_iterative_build_reachable_and_precedes_materialization() {
    for (resources, build_ids) in [
        (group(1, [1]), vec![1]),
        (format!("{}{}", group(1, [2]), group(2, [1])), vec![1]),
    ] {
        assert_eq!(
            project(&resources, &build_ids, &build_ids)
                .unwrap_err()
                .to_string(),
            CYCLE_ERROR
        );
    }

    let unreachable = format!("{}{}", mesh(1), group(2, [2]));
    assert_eq!(
        project(&unreachable, &[1], &[1]).unwrap().objects().len(),
        1
    );

    let cycle_after_unmaterialized_mesh = format!("{}{}", mesh(1), group(2, [2]));
    assert_eq!(
        project(&cycle_after_unmaterialized_mesh, &[1, 2], &[2])
            .unwrap_err()
            .to_string(),
        CYCLE_ERROR
    );
}

#[test]
fn task22b_component_expansion_is_breadth_first_not_depth_first_or_source_id_sorted() {
    let resources = format!(
        "{}{}{}{}{}",
        mesh(1),
        mesh(2),
        mesh(3),
        group(20, [1, 2]),
        group(10, [20, 3])
    );
    let loaded = project(&resources, &[10], &[10]).unwrap();

    assert_eq!(
        loaded.objects()[0]
            .volumes()
            .iter()
            .map(|volume| volume.id())
            .collect::<Vec<_>>(),
        [3, 1, 2]
    );
}
