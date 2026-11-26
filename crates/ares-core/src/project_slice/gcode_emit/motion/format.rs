#[inline]
pub(super) fn push_axis(output: &mut Vec<u8>, value: f64) {
    push_fixed(output, value, 3);
}

#[inline]
pub(super) fn push_offset(output: &mut Vec<u8>, value: f64) {
    push_fixed(output, value, 3);
}

#[inline]
pub(super) fn push_extrusion(output: &mut Vec<u8>, value: f64) {
    push_fixed(output, value, 5);
}

pub(super) fn axis(value: f64) -> String {
    fixed_string(value, 3)
}

pub(super) fn offset(value: f64) -> String {
    axis(value)
}

pub(in crate::project_slice::gcode_emit) fn z(value: f64) -> String {
    axis(value)
}

pub(super) fn extrusion(value: f64) -> String {
    fixed_string(value, 5)
}

fn fixed_string(value: f64, precision: usize) -> String {
    let mut buf = [0_u8; 32];
    let len = format_fixed_buf(value, precision, &mut buf);
    std::str::from_utf8(&buf[..len])
        .expect("fixed formatter produces ASCII")
        .to_owned()
}

#[inline]
fn push_fixed(output: &mut Vec<u8>, value: f64, precision: usize) {
    let mut buf = [0_u8; 32];
    let len = format_fixed_buf(value, precision, &mut buf);
    output.extend_from_slice(&buf[..len]);
}

/// `GCodeFormatter::emit_axis` (GCodeWriter.cpp:1224-1265):
/// `v_int = int64_t(std::round(v * pow_10[digits]))` — round-half-away
/// in the scaled integer domain, strip trailing zeros and leading zero
/// before the decimal point (e.g. `0.5` → `.5`, `-0.5` → `-.5`).
fn format_fixed_buf(value: f64, precision: usize, buf: &mut [u8; 32]) -> usize {
    let (scale, scale_u64) = match precision {
        3 => (1_000.0, 1_000_u64),
        5 => (100_000.0, 100_000_u64),
        _ => (10f64.powi(precision as i32), 10_u64.pow(precision as u32)),
    };
    let scaled = (value * scale).round();
    let abs_int = (scaled.abs() as i64) as u64;
    if abs_int == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut len = 0;
    if scaled < 0.0 {
        buf[len] = b'-';
        len += 1;
    }
    let int_part = abs_int / scale_u64;
    let frac_part = abs_int % scale_u64;
    if int_part > 0 {
        let start = len;
        let mut n = int_part;
        while n > 0 {
            buf[len] = b'0' + (n % 10) as u8;
            len += 1;
            n /= 10;
        }
        buf[start..len].reverse();
    }
    if precision > 0 && frac_part > 0 {
        buf[len] = b'.';
        len += 1;
        let start = len;
        let mut n = frac_part;
        for i in (0..precision).rev() {
            buf[start + i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        let mut end = start + precision;
        while end > start && buf[end - 1] == b'0' {
            end -= 1;
        }
        len = end;
    }
    len
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
        return if value.is_sign_negative() { "-0" } else { "0" }.to_owned();
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
