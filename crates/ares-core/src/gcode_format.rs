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

pub(crate) fn ensure_trailing_newline(mut text: String) -> String {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text
}
