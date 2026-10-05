use super::*;
use crate::{GCodeExtrusionRole as Role, MoveType as Type};

/// Layers 0-2: travel / extrusions / seam / retract / wipe sequence covering
/// every branch of `update_view_full_range`.
fn viewer() -> ViewerImpl {
    let moves = [
        (0, Type::Travel, Role::None),
        (0, Type::Extrude, Role::Perimeter),
        (0, Type::Extrude, Role::ExternalPerimeter),
        (0, Type::Seam, Role::None),
        (0, Type::Travel, Role::None),
        (1, Type::Travel, Role::None),
        (1, Type::Extrude, Role::Perimeter),
        (1, Type::Extrude, Role::ExternalPerimeter),
        (1, Type::Retract, Role::None),
        (1, Type::Travel, Role::None),
        (2, Type::Wipe, Role::None),
        (2, Type::Wipe, Role::None),
        (2, Type::Extrude, Role::Perimeter),
        (2, Type::Travel, Role::None),
    ];
    let vertices = moves
        .into_iter()
        .map(|(layer_id, move_type, role)| PathVertex {
            layer_id,
            position: [0.0, 0.0, 0.2 * (layer_id + 1) as f32],
            move_type,
            role,
            ..PathVertex::default()
        })
        .collect();
    let mut viewer = ViewerImpl::default();
    viewer.load(GCodeInputData {
        vertices,
        ..GCodeInputData::default()
    });
    viewer
}

fn ranges(viewer: &ViewerImpl) -> [Interval; 3] {
    [
        viewer.get_view_full_range(),
        viewer.get_view_enabled_range(),
        viewer.get_view_visible_range(),
    ]
}

#[test]
fn full_range_starts_one_step_before_the_first_extrusion_and_drops_hidden_trailing_moves() {
    let mut viewer = viewer();
    viewer.set_layers_view_range(0, 2);
    assert_eq!(ranges(&viewer), [[0, 12]; 3]);
    assert_eq!(viewer.get_current_vertex_id(), 12);
}

#[test]
fn layer_range_change_moves_the_current_vertex() {
    let mut viewer = viewer();
    // Settings.hpp: seams are visible by default, so the seam ends layer 0.
    viewer.set_layers_view_range(0, 0);
    assert_eq!(viewer.get_current_vertex_id(), 3);
    viewer.toggle_option_visibility(OptionType::Seams);
    viewer.set_layers_view_range(0, 0);
    assert_eq!(viewer.get_current_vertex_id(), 2);
    viewer.set_layers_view_range(0, 1);
    assert_eq!(viewer.get_current_vertex_id(), 7);
    // set_layers_view_range clamps to the last layer.
    viewer.set_layers_view_range(1, 9);
    assert_eq!(viewer.get_layers_view_range(), [1, 2]);
    assert_eq!(ranges(&viewer), [[5, 12]; 3]);
}

#[test]
fn visible_travels_and_wipes_extend_both_ends() {
    let mut viewer = viewer();
    viewer.toggle_option_visibility(OptionType::Travels);
    viewer.set_layers_view_range(1, 1);
    assert_eq!(ranges(&viewer), [[3, 9]; 3]);

    let mut viewer = super::tests::viewer();
    viewer.toggle_option_visibility(OptionType::Seams);
    viewer.toggle_option_visibility(OptionType::Wipes);
    viewer.set_layers_view_range(2, 2);
    assert_eq!(ranges(&viewer), [[9, 12]; 3]);
}

#[test]
fn nothing_visible_gives_an_empty_range() {
    let mut viewer = viewer();
    viewer.toggle_option_visibility(OptionType::Seams);
    viewer.toggle_extrusion_role_visibility(Role::Perimeter);
    viewer.toggle_extrusion_role_visibility(Role::ExternalPerimeter);
    viewer.set_layers_view_range(0, 2);
    assert_eq!(ranges(&viewer), [[0, 0]; 3]);
    assert_eq!(viewer.get_current_vertex_id(), 0);
}

#[test]
fn top_layer_only_enables_the_top_layer_from_one_step_before_it() {
    let mut viewer = viewer();
    viewer.toggle_top_layer_only_view_range();
    viewer.set_layers_view_range(0, 2);
    assert_eq!(ranges(&viewer), [[0, 12], [11, 12], [11, 12]]);
}

#[test]
fn load_builds_layers_from_the_vertices() {
    let viewer = viewer();
    assert_eq!(viewer.layers().count(), 3);
    assert_eq!(viewer.vertices().len(), 14);
    assert_eq!(viewer.get_layers_view_range(), [0, 2]);
}
