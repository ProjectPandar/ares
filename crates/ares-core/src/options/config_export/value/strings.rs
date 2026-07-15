use super::SerializedConfigValue;

pub(super) fn escape_scalar_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\r' => output.push_str("\\r"),
            '\n' => output.push_str("\\n"),
            '\\' | '"' => {
                output.push('\\');
                output.push(character);
            }
            _ => output.push(character),
        }
    }
    output
}

pub(super) fn serialize_string_vector(values: &[SerializedConfigValue]) -> String {
    values
        .iter()
        .map(|value| quote_string_vector_element(&value.token, values.len()))
        .collect::<Vec<_>>()
        .join(";")
}

fn quote_string_vector_element(value: &str, value_count: usize) -> String {
    let should_quote = (value_count == 1 && value.is_empty())
        || value
            .chars()
            .any(|character| matches!(character, ' ' | '\t' | '\\' | '"' | '\r' | '\n'));
    if !should_quote {
        return value.to_owned();
    }
    format!("\"{}\"", escape_scalar_string(value))
}
