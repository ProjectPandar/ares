use crate::{SliceError, WallSequence};

pub(super) fn parse_wall_sequence(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<WallSequence, SliceError> {
    let Some(value) = values.get("wall_sequence") else {
        return Ok(WallSequence::InnerOuter);
    };
    match value.as_str() {
        Some("inner wall/outer wall") => Ok(WallSequence::InnerOuter),
        Some("outer wall/inner wall") => Ok(WallSequence::OuterInner),
        Some("inner-outer-inner wall") => Ok(WallSequence::InnerOuterInner),
        _ => Err(SliceError::InvalidInput(
            "wall_sequence must be inner wall/outer wall, outer wall/inner wall, or inner-outer-inner wall".to_owned(),
        )),
    }
}
