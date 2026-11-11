use super::{
    expression::{ParsePath, evaluate, evaluate_condition, evaluate_path},
    value::{Config, Value},
};
use parser::{Arm, Node, parse};

mod error;
mod macro_block;
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
    // Output variables created by `local`/`global` live for this render
    // only (`config_local`, `PlaceholderParser.cpp:889-906`): their prior
    // absence is journaled and restored afterwards.
    let mut renderer = Renderer {
        template,
        config,
        filament,
        journal: Vec::new(),
    };
    let output = renderer.render_nodes(&nodes);
    restore_output_variables(renderer.config, &mut renderer.journal);
    output
}

/// Drops output variables created during a render, restoring any value a
/// `local`/`global` declaration replaced (`config_local` lifetime,
/// `PlaceholderParser.cpp:889-906,1384-1448`).
fn restore_output_variables(config: &mut Config, journal: &mut Vec<(String, Option<Value>)>) {
    for (name, previous) in journal.drain(..).rev() {
        match previous {
            Some(value) => config.insert(name, value),
            None => config.remove(&name),
        }
    }
}

/// Render state shared by node and text rendering: the config mutated by
/// assignments and the journal of `local`/`global` output variables.
struct Renderer<'a> {
    template: &'a str,
    config: &'a mut Config,
    filament: usize,
    journal: Vec<(String, Option<Value>)>,
}

impl Renderer<'_> {
    fn render_nodes(&mut self, nodes: &[Node]) -> Result<String, String> {
        let mut output = String::new();
        for node in nodes {
            match node {
                Node::Text { text, offset } => output.push_str(&self.render_text(text, *offset)?),
                Node::Branch { arms } => output.push_str(&self.render_branch(arms)?),
            }
        }
        Ok(output)
    }

    /// `if_else_output` (`PlaceholderParser.cpp:2341-2349`): every condition is evaluated in
    /// order, also after an arm was taken (`:2345`), and the first true arm renders when its
    /// condition is reached.
    fn render_branch(&mut self, arms: &[Arm]) -> Result<String, String> {
        let mut output = None;
        for arm in arms {
            let holds = match &arm.condition {
                None => true,
                Some(condition) => evaluate_condition(&condition.text, self.config, self.filament)
                    .map_err(|error| {
                        let error = error.in_expression(&condition.text);
                        error::describe(self.template, error, condition.offset)
                    })?,
            };
            if holds && output.is_none() {
                output = Some(self.render_nodes(&arm.nodes)?);
            }
        }
        Ok(output.unwrap_or_default())
    }

    fn render_text(&mut self, text: &str, text_offset: usize) -> Result<String, String> {
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
            // Statement lists inside one brace block (`macros`,
            // `PlaceholderParser.cpp:2162-2174`): `;`-separated statements,
            // `local`/`global` and inline `if/then/elsif/else/endif`.
            let start = text_offset + expression_start;
            let template = self.template;
            if delimiter == '{' && macro_block::is_macro_body(expression) {
                let rendered = macro_block::evaluate_block(
                    expression,
                    self.config,
                    self.filament,
                    &mut self.journal,
                )
                .map_err(|error| error::describe(template, error, start))?;
                output.push_str(&rendered);
                index = expression_start + relative_end + 1;
                continue;
            }
            if let Some(parts) = assignment_parts(expression) {
                self.assign(expression, parts, start)?;
                index = expression_start + relative_end + 1;
                continue;
            }
            let value =
                evaluate_path(expression, self.config, path, self.filament).map_err(|error| {
                    error::describe(template, error.in_expression(expression), start)
                })?;
            // `PlaceholderParser.cpp:915`: legacy vectors use the process call's
            // logical filament context, not a template's physical extruder ID.
            let vector_index = match path {
                ParsePath::Legacy => self.filament,
                ParsePath::Brace => 0,
            };
            let value = value.index(vector_index).unwrap_or(&value);
            value.ensure_defined()?;
            let rendered = match path {
                ParsePath::Legacy => value.as_legacy_string(),
                ParsePath::Brace => super::expression::to_string(value),
            };
            output.push_str(&rendered);
            index = expression_start + relative_end + 1;
        }
        Ok(output)
    }

    /// Upstream supports assignment statements (`{var[idx] = expr}`) that
    /// store into the parser and render nothing; `start` is the template
    /// offset of `expression`.
    fn assign(
        &mut self,
        expression: &str,
        (name, index, rhs): (&str, Option<&str>, &str),
        start: usize,
    ) -> Result<(), String> {
        let index = match index {
            Some(index) => match evaluate(index, self.config, self.filament)?.as_number() {
                Some(number) if number >= 0.0 => Some(number as usize),
                _ => return Err(format!("invalid assignment index in {expression}")),
            },
            None => None,
        };
        let value =
            evaluate_path(rhs, self.config, ParsePath::Brace, self.filament).map_err(|error| {
                let error = error.in_expression(expression);
                error::describe(self.template, error, start + offset_in(expression, rhs))
            })?;
        self.config.assign(name, index, value);
        Ok(())
    }
}

/// Byte offset of `part` within `whole`, of which it is a slice.
fn offset_in(whole: &str, part: &str) -> usize {
    part.as_ptr().addr() - whole.as_ptr().addr()
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
