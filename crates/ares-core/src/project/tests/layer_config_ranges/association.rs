use crate::load_project;

use super::LayerProject;

#[test]
fn one_based_ordinals_follow_final_build_order_not_source_ids() {
    let mut project = LayerProject::with_build_order(&[42, 7]);
    project.insert_ranges(
        "Metadata/layer_config_ranges.xml",
        r#"<objects>
          <object id="1"><range min_z="0" max_z="1"><option opt_key="wall_loops">4</option></range></object>
          <object id="2"><range min_z="1" max_z="2"><option opt_key="extruder">3</option></range></object>
        </objects>"#,
    );

    let loaded = load_project(project.bytes()).unwrap();

    assert_eq!(loaded.objects()[0].id(), 42);
    assert_eq!(loaded.objects()[1].id(), 7);
    assert_eq!(
        loaded.objects()[0].layer_config_ranges()[0]
            .region_overrides()
            .wall_loops,
        Some(crate::OrcaInt(4))
    );
    assert_eq!(
        loaded.objects()[1].layer_config_ranges()[0]
            .region_overrides()
            .extruder,
        Some(crate::OrcaInt(3))
    );
}

#[test]
fn ranges_sort_lexicographically_without_normalizing_raw_shapes() {
    let mut project = LayerProject::one_object();
    project.insert_ranges(
        "Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1">
          <range min_z="10" max_z="5"><option opt_key="wall_loops">1</option></range>
          <range min_z="7" max_z="8"><option opt_key="wall_loops">2</option></range>
          <range min_z="-2" max_z="1"><option opt_key="wall_loops">3</option></range>
          <range min_z="0.5" max_z="3"><option opt_key="wall_loops">4</option></range>
        </object></objects>"#,
    );

    let loaded = load_project(project.bytes()).unwrap();
    let ranges = loaded.objects()[0].layer_config_ranges();
    let bounds = ranges
        .iter()
        .map(|range| (range.min_z(), range.max_z()))
        .collect::<Vec<_>>();

    assert_eq!(bounds, [(-2.0, 1.0), (0.5, 3.0), (7.0, 8.0), (10.0, 5.0)]);
}

#[test]
fn task22a_range_layer_height_is_typed_separate_and_last_write_wins() {
    let mut project = LayerProject::one_object();
    project.insert_ranges(
        "Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1"><range min_z="0" max_z="1">
          <option opt_key="wall_loops">2</option>
          <option opt_key="layer_height">0.18</option>
          <option opt_key="wall_loops">9</option>
          <option opt_key="layer_height">0.24</option>
        </range></object></objects>"#,
    );

    let loaded = load_project(project.bytes()).unwrap();
    let range = &loaded.objects()[0].layer_config_ranges()[0];
    let overrides = range.region_overrides();

    assert_eq!(range.layer_height(), Some(crate::OrcaFloat(0.24)));
    assert_eq!(overrides.wall_loops, Some(crate::OrcaInt(9)));
    assert_eq!(overrides.present_keys(), ["wall_loops"]);
}

#[test]
fn task22a_range_duplicate_replacement_clears_prior_layer_height() {
    let mut project = LayerProject::one_object();
    project.insert_ranges(
        "Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1">
          <range min_z="-0.0" max_z="2"><option opt_key="layer_height">0.18</option><option opt_key="wall_loops">8</option><option opt_key="sparse_infill_density">31%</option></range>
          <range min_z="0.0" max_z="2"><option opt_key="extruder">5</option></range>
        </object></objects>"#,
    );

    let loaded = load_project(project.bytes()).unwrap();
    let ranges = loaded.objects()[0].layer_config_ranges();

    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].min_z().to_bits(), 0.0_f64.to_bits());
    assert_eq!(ranges[0].max_z(), 2.0);
    assert_eq!(ranges[0].layer_height(), None);
    assert_eq!(ranges[0].region_overrides().present_keys(), ["extruder"]);
    assert_eq!(
        ranges[0].region_overrides().extruder,
        Some(crate::OrcaInt(5))
    );
}

#[test]
fn empty_object_groups_add_no_range_state() {
    let mut project = LayerProject::one_object();
    project.insert_ranges(
        "Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1"/></objects>"#,
    );

    let loaded = load_project(project.bytes()).unwrap();

    assert!(loaded.objects()[0].layer_config_ranges().is_empty());
}

#[test]
fn bounded_range_without_options_retains_an_empty_typed_config() {
    let mut project = LayerProject::one_object();
    project.insert_ranges(
        "Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1"><range min_z="-1" max_z="-2"/></object></objects>"#,
    );

    let loaded = load_project(project.bytes()).unwrap();
    let ranges = loaded.objects()[0].layer_config_ranges();

    assert_eq!(ranges.len(), 1);
    assert_eq!((ranges[0].min_z(), ranges[0].max_z()), (-1.0, -2.0));
    assert!(ranges[0].region_overrides().present_keys().is_empty());
}
