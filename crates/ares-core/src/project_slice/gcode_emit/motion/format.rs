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
    // `GCodeFormatter::emit_axis` (GCodeWriter.cpp:1224-1265):
    // `v_int = int64_t(std::round(v * pow_10[digits]))` — round-half-away
    // in the scaled integer domain, then place the decimal point into the
    // integer digits and strip trailing zeros.
    let scale = 10f64.powi(precision as i32);
    let scaled = (value * scale).round();
    let negative = scaled < 0.0;
    let digits = (scaled.abs() as i64).to_string();
    let (int, frac) = if digits.len() > precision {
        digits.split_at(digits.len() - precision)
    } else {
        ("0", digits.as_str())
    };
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(int);
    if precision > 0 {
        let frac: String = std::iter::repeat_n('0', precision.saturating_sub(frac.len()))
            .chain(frac.chars())
            .collect();
        let frac = frac.trim_end_matches('0');
        if !frac.is_empty() {
            out.push('.');
            out.push_str(frac);
        }
    }
    if out.is_empty() || out == "-" || out == "-0" {
        "0".to_owned()
    } else {
        out
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
    // Upstream uses `std::ostream <<` which rounds to decimal nearest
    // (round-half-even on binary ties like 1.015625 → "1.01562"), NOT the
    // emit_axis half-away integer-domain rounding of `trim_fixed`.
    let decimals = (5 - exponent).clamp(0, 6) as usize;
    let fixed = format!("{value:.decimals$}");
    let trimmed = fixed.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_owned()
}
#[cfg(test)]
mod tests;
