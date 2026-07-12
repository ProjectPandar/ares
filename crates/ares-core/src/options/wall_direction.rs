use crate::{SliceError, WallDirection};

pub(super) fn parse_wall_direction(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<WallDirection, SliceError> {
    let Some(value) = values.get("wall_direction") else {
        return Ok(WallDirection::CounterClockwise);
    };
    match value.as_str() {
        Some("ccw") => Ok(WallDirection::CounterClockwise),
        Some("cw") => Ok(WallDirection::Clockwise),
        _ => Err(SliceError::InvalidInput(
            "wall_direction must be ccw or cw".to_owned(),
        )),
    }
}
