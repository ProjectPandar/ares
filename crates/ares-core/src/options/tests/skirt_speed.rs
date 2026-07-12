use super::super::*;
use crate::{PrintPathRole, ToolpathMoveKind};
use serde_json::json;

#[test]
fn speed_options_apply_default_skirt_speed() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::Skirt),
        50.0
    );
}

#[test]
fn speed_options_apply_positive_skirt_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_speed": 35,
        "outer_wall_speed": 60
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::Skirt),
        35.0
    );
}

#[test]
fn speed_options_zero_skirt_speed_uses_external_perimeter_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_speed": 0,
        "outer_wall_speed": 40
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_role(ToolpathMoveKind::Print, PrintPathRole::Skirt),
        40.0
    );
}
