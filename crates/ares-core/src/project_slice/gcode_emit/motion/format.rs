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

/// C++ ostream default `operator<<` for a double: 6 significant digits
/// (`GCodeWriter.cpp:295` `<< jerk_x`), i.e. printf `%g`: round to the
/// precision FIRST, then pick fixed vs scientific on the rounded
/// exponent (999999.5 → `1e+06`, not `1000000`).
pub(in crate::project_slice::gcode_emit) fn ostream(value: f64) -> String {
    ostream_with_precision(value, 6)
}

pub(super) fn ostream_with_precision(value: f64, precision: usize) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    // Round to the requested significant digits in one step (half-even,
    // like C++ iostreams), renormalizing any carry into the exponent.
    // Dividing manually by 10^exp would double-round values like 10.03125.
    let decimal_places = precision - 1;
    let rendered = format!("{value:.decimal_places$e}");
    let (mantissa, exponent) = rendered.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let (sign, mantissa) = mantissa
        .strip_prefix('-')
        .map_or(("", mantissa), |m| ("-", m));
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    // %g uses scientific notation outside [-4, precision).
    if !(-4..precision as i32).contains(&exponent) {
        let (first, rest) = digits.split_at(1);
        let mantissa = if rest.is_empty() {
            first.to_owned()
        } else {
            format!("{first}.{rest}")
        };
        return format!("{sign}{mantissa}e{exponent:+03}");
    }
    // Fixed: place the point `exponent + 1` digits in.
    let point = exponent + 1;
    let fixed = if point <= 0 {
        format!("0.{}{digits}", "0".repeat((-point) as usize))
    } else if (point as usize) >= digits.len() {
        format!("{digits}{}", "0".repeat(point as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    // Trim only a fractional part; integer digit strings keep their
    // zeros (100000 must not collapse to 1).
    let fixed = if fixed.contains('.') {
        fixed.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        fixed
    };
    format!("{sign}{fixed}")
}
#[cfg(test)]
mod tests;
