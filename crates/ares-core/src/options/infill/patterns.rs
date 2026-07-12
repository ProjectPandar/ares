use serde_json::Value;

use crate::SliceError;

use super::InfillPattern;

pub(super) fn parse_sparse_infill_rotate_template(
    value: Option<&Value>,
) -> Result<Vec<f64>, SliceError> {
    parse_infill_rotate_template("sparse_infill_rotate_template", value)
}

pub(crate) fn parse_infill_rotate_template(
    key: &str,
    value: Option<&Value>,
) -> Result<Vec<f64>, SliceError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Some(text) = value.as_str() else {
        return Err(SliceError::InvalidInput(format!("{key} must be a string")));
    };
    let text = text.trim();
    if text.is_empty() {
        return Ok(Vec::new());
    }
    if text.chars().any(|ch| {
        matches!(
            ch,
            '+' | '-'
                | '%'
                | '*'
                | '@'
                | '\''
                | '"'
                | '/'
                | 'N'
                | 'n'
                | 'Z'
                | 'z'
                | '$'
                | 'L'
                | 'l'
                | 'U'
                | 'u'
                | 'Q'
                | 'q'
                | '~'
                | '^'
                | '|'
                | '#'
        )
    }) {
        return Err(SliceError::InvalidInput(format!(
            "{key} only supports comma-separated unsigned degree values"
        )));
    }
    text.split(',')
        .map(str::trim)
        .map(|token| {
            if token.is_empty() || token.chars().any(char::is_whitespace) {
                return Err(SliceError::InvalidInput(format!(
                    "{key} only supports comma-separated unsigned degree values"
                )));
            }
            let value = token.parse::<f64>().map_err(|_| {
                SliceError::InvalidInput(format!("{key} must contain finite degree values"))
            })?;
            value.is_finite().then_some(value).ok_or_else(|| {
                SliceError::InvalidInput(format!("{key} must contain finite degree values"))
            })
        })
        .collect()
}

pub(crate) fn parse_sparse_infill_pattern(
    value: Option<&Value>,
) -> Result<InfillPattern, SliceError> {
    let Some(value) = value else {
        return Ok(InfillPattern::CrossHatch);
    };
    let Some(text) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "sparse_infill_pattern must be a string".to_owned(),
        ));
    };
    match text {
        "rectilinear" => Ok(InfillPattern::Rectilinear),
        "alignedrectilinear" => Ok(InfillPattern::AlignedRectilinear),
        "line" => Ok(InfillPattern::Line),
        "grid" => Ok(InfillPattern::Grid),
        "zigzag" => Ok(InfillPattern::ZigZag),
        "crosszag" => Ok(InfillPattern::CrossZag),
        "lockedzag" => Ok(InfillPattern::LockedZag),
        "crosshatch" => Ok(InfillPattern::CrossHatch),
        "triangles" | "tri-hexagon" | "cubic" | "adaptivecubic" | "quartercubic"
        | "supportcubic" | "lightning" | "honeycomb" | "3dhoneycomb" | "lateral-honeycomb"
        | "lateral-lattice" | "tpmsd" | "tpmsfk" | "gyroid" | "concentric" | "hilbertcurve"
        | "archimedeanchords" | "octagramspiral" => Err(SliceError::InvalidInput(format!(
            "sparse_infill_pattern {text} is not implemented"
        ))),
        _ => Err(SliceError::InvalidInput(format!(
            "unknown sparse_infill_pattern {text}"
        ))),
    }
}

pub(crate) fn parse_internal_solid_infill_pattern(
    value: Option<&Value>,
) -> Result<InfillPattern, SliceError> {
    parse_surface_pattern(
        "internal_solid_infill_pattern",
        value,
        InfillPattern::Monotonic,
    )
}

pub(crate) fn parse_top_surface_pattern(
    value: Option<&Value>,
) -> Result<InfillPattern, SliceError> {
    parse_surface_pattern("top_surface_pattern", value, InfillPattern::MonotonicLine)
}

pub(crate) fn parse_bottom_surface_pattern(
    value: Option<&Value>,
) -> Result<InfillPattern, SliceError> {
    parse_surface_pattern("bottom_surface_pattern", value, InfillPattern::Monotonic)
}

fn parse_surface_pattern(
    key: &str,
    value: Option<&Value>,
    default: InfillPattern,
) -> Result<InfillPattern, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let Some(text) = value.as_str() else {
        return Err(SliceError::InvalidInput(format!("{key} must be a string")));
    };
    match text {
        "rectilinear" => Ok(InfillPattern::Rectilinear),
        "alignedrectilinear" => Ok(InfillPattern::AlignedRectilinear),
        "monotonic" => Ok(InfillPattern::Monotonic),
        "monotonicline" => Ok(InfillPattern::MonotonicLine),
        "line" if key == "internal_solid_infill_pattern" => Ok(InfillPattern::Line),
        "grid" if key == "internal_solid_infill_pattern" => Ok(InfillPattern::Grid),
        "zigzag" if key == "internal_solid_infill_pattern" => Ok(InfillPattern::ZigZag),
        "concentric" => Ok(InfillPattern::Concentric),
        "concentric_internal" | "hilbertcurve" | "archimedeanchords" | "octagramspiral" => Err(
            SliceError::InvalidInput(format!("{key} {text} is not implemented")),
        ),
        _ => Err(SliceError::InvalidInput(format!("unknown {key} {text}"))),
    }
}
