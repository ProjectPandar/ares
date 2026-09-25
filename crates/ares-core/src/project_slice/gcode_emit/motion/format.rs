pub(super) fn axis(value: f64) -> String {
    let value = trim_fixed(value, 3);
    if value == "0" {
        value
    } else if let Some(value) = value.strip_prefix("-0") {
        format!("-{value}")
    } else {
        value.strip_prefix('0').unwrap_or(&value).to_owned()
    }
}

pub(super) fn offset(value: f64) -> String {
    let value = trim_fixed(value, 3);
    if value == "0" {
        value
    } else if let Some(value) = value.strip_prefix("-0") {
        format!("-{value}")
    } else {
        value.strip_prefix('0').unwrap_or(&value).to_owned()
    }
}

pub(in crate::project_slice::gcode_emit) fn z(value: f64) -> String {
    offset(value)
}

pub(super) fn extrusion(value: f64) -> String {
    let value = trim_fixed(value, 5);
    if value == "0" {
        value
    } else if let Some(value) = value.strip_prefix("-0") {
        format!("-{value}")
    } else {
        value.strip_prefix('0').unwrap_or(&value).to_owned()
    }
}

fn trim_fixed(value: f64, precision: usize) -> String {
    let mut value = format!("{value:.precision$}");
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    if value.is_empty() || value == "-" || value == "-0" {
        "0".to_owned()
    } else {
        value
    }
}

/// C++ ostream default `operator<<` for a double: 6 significant digits,
/// trailing zeros dropped (`GCodeWriter.cpp:295` `<< jerk_x`). Used for the
/// M205 XY jerk values, which upstream formats with the stream default, not
/// the 3-decimal axis quantizer. Jerks stay well inside `%g`'s fixed-notation
/// range, so a fixed-precision 6-significant-digit render suffices.
pub(super) fn ostream(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    // Number of digits after the point needed for 6 significant figures:
    // `5 - floor(log10(|v|))`, clamped so sub-unit values still get digits.
    let exponent = value.abs().log10().floor() as i32;
    let decimals = (5 - exponent).clamp(0, 6) as usize;
    trim_fixed(value, decimals)
}
#[cfg(test)]
mod tests;
