use super::{
    expression::{ParsePath, PathError, evaluate, evaluate_path},
    value::Config,
};
use parser::{Node, parse};

mod error;
mod parser;

pub(super) fn render(template: &str, config: &mut Config) -> Result<String, String> {
    render_for_filament(template, config, 0)
}

pub(super) fn render_for_filament(
    template: &str,
    config: &mut Config,
    filament: usize,
) -> Result<String, String> {
    // `process_macro` parses the template value byte-for-byte with no trim
    // and no prefix (`PlaceholderParser.cpp:2516-2526`); the ASCII
    // whitespace skipper drops the leading ' '/'\t'/'\r'/'\n' from the
    // output only (the MMU start gcode's `" _SP_PRINT_START"` loses its
    // leading space), while parse positions — and the error line/caret
    // report (`process_error_message` `:1731-1770`) — stay relative to the
    // untrimmed template, so a skipped blank line still counts.
    let nodes = parse(template)?;
    render_nodes(&nodes, template, config, filament)
}

#[expect(
    clippy::excessive_nesting,
    reason = "branch selection stays with node rendering"
)]
fn render_nodes(
    nodes: &[Node],
    template: &str,
    config: &mut Config,
    filament: usize,
) -> Result<String, String> {
    let mut output = String::new();
    for node in nodes {
        match node {
            Node::Text { text, offset } => {
                output.push_str(&render_text(text, *offset, template, config, filament)?)
            }
            Node::Branch { arms } => {
                for arm in arms {
                    let selected = match &arm.condition {
                        None => true,
                        Some(condition) => evaluate(condition, config, filament)
                            .map_err(|error| format!("{error} in {condition}"))?
                            .as_bool(),
                    };
                    if selected {
                        output.push_str(&render_nodes(&arm.nodes, template, config, filament)?);
                        break;
                    }
                }
            }
        }
    }
    Ok(output)
}

fn render_text(
    text: &str,
    text_offset: usize,
    template: &str,
    config: &mut Config,
    filament: usize,
) -> Result<String, String> {
    let mut output = String::new();
    let mut index = 0;
    while index < text.len() {
        let remaining = &text[index..];
        let next_square = remaining.find('[');
        let next_brace = remaining.find('{');
        let Some((offset, delimiter)) = [
            next_square.map(|offset| (offset, '[')),
            next_brace.map(|offset| (offset, '{')),
        ]
        .into_iter()
        .flatten()
        .min_by_key(|(offset, _)| *offset) else {
            output.push_str(remaining);
            break;
        };
        output.push_str(&remaining[..offset]);
        index += offset;
        let expression_start = index + 1;
        let Some(relative_end) = placeholder_end(text, expression_start, delimiter) else {
            return Err(format!("unclosed placeholder in {text:?}"));
        };
        let expression = &text[expression_start..expression_start + relative_end];
        // The placeholder spelling selects the upstream parse path: brace
        // placeholders resolve symbols via resolve_variable
        // (`PlaceholderParser.cpp:988-1000`), bracket placeholders expand
        // via legacy_variable_expansion (`:918-932`); each path reports
        // unknown symbols with its own message.
        let path = match delimiter {
            '{' => ParsePath::Brace,
            _ => ParsePath::Legacy,
        };
        // Upstream supports assignment statements (`{var[idx] = expr}`) that
        // store into the parser and render nothing.
        if let Some((name, index_expression, rhs)) = assignment_parts(expression) {
            let assignment_index = match index_expression {
                Some(index_expression) => {
                    match evaluate(index_expression, config, filament)?.as_number() {
                        Some(number) if number >= 0.0 => Some(number as usize),
                        _ => return Err(format!("invalid assignment index in {expression}")),
                    }
                }
                None => None,
            };
            let value = evaluate(rhs, config, filament)
                .map_err(|error| format!("{error} in {expression}"))?;
            config.assign(name, assignment_index, value);
            index = expression_start + relative_end + 1;
            continue;
        }
        let value = match evaluate_path(expression, config, path, filament) {
            Ok(value) => value,
            Err(PathError::Parse(failure)) => {
                return Err(error::report(
                    template,
                    &failure,
                    text_offset + expression_start + failure.offset,
                ));
            }
            Err(PathError::Other(message)) => {
                return Err(format!("{message} in {expression}"));
            }
        };
        // `PlaceholderParser.cpp:915`: legacy vectors use the process call's
        // logical filament context, not a template's physical extruder ID.
        let vector_index = match path {
            ParsePath::Legacy => filament,
            ParsePath::Brace => 0,
        };
        let value = value.index(vector_index).unwrap_or(&value);
        let rendered = match path {
            ParsePath::Legacy => value.as_legacy_string(),
            ParsePath::Brace => super::expression::to_string(value),
        };
        output.push_str(&rendered);
        index = expression_start + relative_end + 1;
    }
    Ok(output)
}

/// Splits `name[index] = rhs` assignment statements; returns the variable
/// name, the optional index expression and the right-hand side.
fn assignment_parts(expression: &str) -> Option<(&str, Option<&str>, &str)> {
    let chars: Vec<char> = expression.chars().collect();
    let mut depth = 0_i32;
    for (position, character) in chars.iter().enumerate() {
        match character {
            '[' => depth += 1,
            ']' => depth -= 1,
            '=' if depth == 0 => {
                let previous = if position > 0 {
                    chars[position - 1]
                } else {
                    ' '
                };
                let next = chars.get(position + 1).copied().unwrap_or(' ');
                if matches!(previous, '=' | '<' | '>' | '!' | '~') || next == '=' {
                    return None;
                }
                let lhs = expression[..position].trim();
                let rhs = expression[position + 1..].trim();
                let (name, index) = match lhs.split_once('[') {
                    Some((name, index)) if index.ends_with(']') => {
                        (name.trim(), Some(index[..index.len() - 1].trim()))
                    }
                    _ => (lhs, None),
                };
                let valid_name =
                    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                let valid_index = index.is_none_or(|index| {
                    !index.is_empty()
                        && index.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                });
                if valid_name && valid_index {
                    return Some((name, index, rhs));
                }
            }
            _ => {}
        }
    }
    None
}

fn placeholder_end(text: &str, start: usize, delimiter: char) -> Option<usize> {
    if delimiter != '[' {
        return text[start..].find('}');
    }
    let mut depth = 0;
    for (offset, character) in text[start..].char_indices() {
        match character {
            '[' => depth += 1,
            ']' if depth == 0 => return Some(offset),
            ']' => depth -= 1,
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests;
