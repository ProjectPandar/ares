pub(super) fn axis(value: f64) -> String {
    let scale = 1_000.0;
    let value = trim_fixed((value * scale).round() / scale, 3);
    if value == "0" {
        value
    } else if let Some(value) = value.strip_prefix("-0") {
        format!("-{value}")
    } else {
        value.strip_prefix('0').unwrap_or(&value).to_owned()
    }
}

pub(super) fn offset(value: f64) -> String {
    let scale = 1_000.0;
    let value = trim_fixed((value * scale).round() / scale, 3);
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
    let value = trim_fixed((value * 100_000.0).round() / 100_000.0, 5);
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
#[cfg(test)]
mod tests;
