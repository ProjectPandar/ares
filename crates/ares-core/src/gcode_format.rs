use crate::PipelineStage;

pub(crate) fn format_pipeline_stages(stages: &[PipelineStage]) -> String {
    stages
        .iter()
        .map(|stage| stage.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn format_decimal_list(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format_decimal(*value))
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn format_decimal(value: f64) -> String {
    let mut text = format!("{value:.6}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

pub(crate) fn ensure_trailing_newline(text: String) -> String {
    // PlaceholderParser's skipper consumes leading whitespace before the
    // first grammar token (`PlaceholderParser.cpp` phrase_parse with
    // `client::skipper`) — applied here to mirror `process()`.
    // Upstream skipper chars are exactly space/tab/CR/LF
    // (`ascii_char_skipper_parser`, PlaceholderParser.cpp:1976-1986) — other
    // unicode whitespace (e.g. form feed) must survive.
    let mut text = text.trim_start_matches([' ', '\t', '\r', '\n']).to_owned();
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text
}
