pub(super) fn canonical_number(value: &str) -> Result<String, String> {
    let number = value
        .parse::<f64>()
        .map_err(|_| format!("invalid G-code number {value:?}"))?;
    if !number.is_finite() {
        return Err(format!("non-finite G-code number {value:?}"));
    }
    let mut rendered = format!("{number:.8}");
    while rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.pop();
    }
    if rendered == "-0" {
        rendered = "0".to_owned();
    }
    Ok(rendered)
}
