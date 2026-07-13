use std::collections::BTreeSet;

use super::{gcode_rows, inventory};

mod load;
mod projection;
mod shapes;
mod templates;

#[test]
fn gcode_options_fixture_loads_all_sources_and_projects_all_fields() {
    let fixture = load::load_fixture();
    let inventory = inventory();
    let rows = gcode_rows(&inventory);
    let gcode_keys = rows
        .iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let non_gcode_keys = inventory
        .iter()
        .filter(|row| {
            !row
                .effective_projections
                .iter()
                .any(|projection| projection == "g_code")
        })
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(gcode_keys.len(), 149);
    assert!(fixture.raw.len() > gcode_keys.len());
    assert!(gcode_keys.is_disjoint(&non_gcode_keys));
    projection::assert_all_fields(&fixture);
}

#[test]
fn gcode_options_fixture_preserves_exact_raw_array_shapes() {
    shapes::assert_raw_shapes(&load::load_fixture());
}

#[test]
fn gcode_options_fixture_preserves_all_template_bytes() {
    templates::assert_template_bytes(&load::load_fixture());
}
