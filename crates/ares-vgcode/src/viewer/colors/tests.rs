use crate::{
    ColorRange, GCodeExtrusionRole as Role, GCodeInputData, MoveType as Type, OptionType,
    PathVertex, ViewType, ViewerImpl,
};

/// Perimeter and Custom extrusions plus a fast travel and a wipe.
fn viewer() -> ViewerImpl {
    let vertex = |move_type, role, width, height, feedrate| PathVertex {
        move_type,
        role,
        width,
        height,
        feedrate,
        mm3_per_mm: 0.1,
        ..PathVertex::default()
    };
    let mut viewer = ViewerImpl::default();
    viewer.load(GCodeInputData {
        vertices: vec![
            vertex(Type::Extrude, Role::Perimeter, 0.4, 0.2, 50.0),
            vertex(Type::Extrude, Role::Custom, 0.8, 0.3, 20.0),
            vertex(Type::Travel, Role::None, 0.0, 0.0, 300.0),
            vertex(Type::Wipe, Role::None, 0.0, 0.0, 40.0),
        ],
        ..GCodeInputData::default()
    });
    viewer
}

/// Range bounds, compared past the `round_to_bin` float error.
fn bounds(viewer: &mut ViewerImpl, view_type: ViewType) -> [f32; 2] {
    viewer.colors();
    let range = viewer
        .get_color_range(view_type)
        .map(ColorRange::get_range)
        .unwrap();
    range.map(|value| (value * 1e4).round() / 1e4)
}

/// Hidden Custom extrusions leave width and flow but not height
/// (`ViewerImpl.cpp:1845-1851`); the cached ranges follow the toggle.
#[test]
fn custom_role_visibility_gates_width_and_flow() {
    let mut viewer = viewer();
    assert_eq!(bounds(&mut viewer, ViewType::Width), [0.4, 0.8]);
    viewer.toggle_extrusion_role_visibility(Role::Custom);
    assert_eq!(bounds(&mut viewer, ViewType::Width), [0.4, 0.4]);
    assert_eq!(
        bounds(&mut viewer, ViewType::VolumetricFlowRate),
        [5.0, 5.0]
    );
    assert_eq!(bounds(&mut viewer, ViewType::Height), [0.2, 0.3]);
}

/// Speed covers travels and wipes only while visible (`ViewerImpl.cpp:1858-1866`).
#[test]
fn option_visibility_gates_speed() {
    let mut viewer = viewer();
    assert_eq!(bounds(&mut viewer, ViewType::Speed), [20.0, 50.0]);
    viewer.toggle_option_visibility(OptionType::Travels);
    assert_eq!(bounds(&mut viewer, ViewType::Speed), [20.0, 300.0]);
    viewer.toggle_option_visibility(OptionType::Wipes);
    viewer.toggle_option_visibility(OptionType::Travels);
    assert_eq!(bounds(&mut viewer, ViewType::Speed), [20.0, 50.0]);
}

/// Wipes take the option color except in the speed views; travels outside
/// them too (`ViewerImpl.cpp:1472-1473, 1483-1486`).
#[test]
fn wipe_and_travel_colors_by_view_type() {
    let mut viewer = viewer();
    viewer.set_view_type(ViewType::Height);
    let wipes = super::DEFAULT_OPTIONS_COLORS[OptionType::Wipes.index()];
    let travels = super::DEFAULT_OPTIONS_COLORS[OptionType::Travels.index()];
    assert_eq!(viewer.colors()[2..], [travels, wipes]);
    viewer.set_view_type(ViewType::Speed);
    let speed = viewer.get_color_range(ViewType::Speed).unwrap().clone();
    let colors = viewer.colors().to_vec();
    assert_eq!(
        colors[2..],
        [speed.get_color_at(300.0), speed.get_color_at(40.0)]
    );
}
