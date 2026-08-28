use super::{expression::evaluate, value::Config};

/// Orca template node: literal text or an `{if}`/`{elsif}`/`{else}` branch.
/// Directives are consumed with their condition text (including any newline
/// inside a multiline condition); every other byte, newlines included, is
/// preserved verbatim, so whole-line and inline directives share one engine.
enum Node {
    Text(String),
    Branch { arms: Vec<Arm> },
}

/// One `{if}`/`{elsif}`/`{else}` arm; `None` condition is the catch-all
/// `{else}`.
struct Arm {
    condition: Option<String>,
    nodes: Vec<Node>,
}

pub(super) fn render(template: &str, config: &mut Config) -> Result<String, String> {
    let nodes = parse(template)?;
    render_nodes(&nodes, config)
}

fn parse(template: &str) -> Result<Vec<Node>, String> {
    let mut parser = Parser {
        template,
        offset: 0,
    };
    let (nodes, terminator) = parser.parse_branch()?;
    match terminator {
        None => Ok(nodes),
        Some(_) => Err("unexpected template directive".to_owned()),
    }
}

struct Parser<'a> {
    template: &'a str,
    offset: usize,
}

#[expect(
    clippy::excessive_nesting,
    reason = "recursive descent keeps directive scanning ordered"
)]
impl Parser<'_> {
    /// Parses nodes until a branch terminator (`elsif`/`else`/`endif`), which
    /// is consumed and reported without turning it into a node.
    fn parse_branch(&mut self) -> Result<(Vec<Node>, Option<Directive>), String> {
        let mut nodes = Vec::new();
        loop {
            let Some((start, end, directive)) = self.next_directive() else {
                let text = &self.template[self.offset..];
                if !text.is_empty() {
                    nodes.push(Node::Text(text.to_owned()));
                }
                self.offset = self.template.len();
                return Ok((nodes, None));
            };
            if start > self.offset {
                nodes.push(Node::Text(self.template[self.offset..start].to_owned()));
            }
            self.offset = end;
            match directive {
                Directive::If(condition) => {
                    let arms = self.parse_arms(condition)?;
                    nodes.push(Node::Branch { arms });
                }
                terminator => return Ok((nodes, Some(terminator))),
            }
        }
    }

    /// Parses the `{if}`/`{elsif}`/`{else}` arms that follow the opening
    /// condition, consuming the closing `{endif}`.
    fn parse_arms(&mut self, condition: String) -> Result<Vec<Arm>, String> {
        let mut arms = vec![Arm {
            condition: (!condition.is_empty()).then_some(condition),
            nodes: Vec::new(),
        }];
        let (nodes, terminator) = self.parse_branch()?;
        arms.last_mut().unwrap().nodes = nodes;
        let mut terminator = terminator;
        loop {
            match terminator {
                Some(Directive::Elsif(condition)) => {
                    let (nodes, next) = self.parse_branch()?;
                    arms.push(Arm {
                        condition: (!condition.is_empty()).then_some(condition),
                        nodes,
                    });
                    terminator = next;
                }
                Some(Directive::Else) => {
                    let (nodes, next) = self.parse_branch()?;
                    if !matches!(next, Some(Directive::Endif)) {
                        return Err("missing endif".to_owned());
                    }
                    arms.push(Arm {
                        condition: None,
                        nodes,
                    });
                    return Ok(arms);
                }
                Some(Directive::Endif) => return Ok(arms),
                _ => return Err("missing endif".to_owned()),
            }
        }
    }

    /// Finds the next directive token and returns `(start, end, directive)`
    /// where `end` is just past the closing `}`. `self.offset` is not
    /// advanced; the caller keeps the text before `start`.
    fn next_directive(&self) -> Option<(usize, usize, Directive)> {
        let bytes = self.template.as_bytes();
        let mut index = self.offset;
        while index < bytes.len() {
            if bytes[index] != b'{' {
                index += 1;
                continue;
            }
            let rest = &self.template[index + 1..];
            let trimmed = rest.trim_start_matches([' ', '\t']);
            let keyword_start = index + 1 + (rest.len() - trimmed.len());
            let (name, length) = if trimmed
                .strip_prefix("if")
                .is_some_and(|tail| tail.starts_with([' ', '\t', '(']) || tail.starts_with('}'))
            {
                ("if", "if".len())
            } else if trimmed
                .strip_prefix("elsif")
                .is_some_and(|tail| tail.starts_with([' ', '\t']) || tail.starts_with('}'))
            {
                ("elsif", "elsif".len())
            } else if trimmed
                .strip_prefix("else")
                .is_some_and(|tail| tail.trim_start().starts_with('}'))
            {
                ("else", "else".len())
            } else if trimmed
                .strip_prefix("endif")
                .is_some_and(|tail| tail.trim_start().starts_with('}'))
            {
                ("endif", "endif".len())
            } else {
                index += 1;
                continue;
            };
            let end = self.directive_end(keyword_start)?;
            let condition = self.template[keyword_start + length..end].trim().to_owned();
            if name == "if" {
                return Some((index, end + 1, Directive::If(condition)));
            }
            let directive = match name {
                "elsif" => Directive::Elsif(condition),
                "else" => Directive::Else,
                _ => Directive::Endif,
            };
            return Some((index, end + 1, directive));
        }
        None
    }

    /// Index of the `}` closing a directive opened at `start` (already past
    /// `{`). Square-bracket subscripts may contain `}`-free indices but are
    /// still skipped for safety.
    fn directive_end(&self, start: usize) -> Option<usize> {
        let mut square_depth: u32 = 0;
        for (offset, character) in self.template[start..].char_indices() {
            match character {
                '[' => square_depth += 1,
                ']' => square_depth = square_depth.saturating_sub(1),
                '}' if square_depth == 0 => return Some(start + offset),
                _ => {}
            }
        }
        None
    }
}

enum Directive {
    If(String),
    Elsif(String),
    Else,
    Endif,
}

#[expect(
    clippy::excessive_nesting,
    reason = "branch selection stays with node rendering"
)]
fn render_nodes(nodes: &[Node], config: &mut Config) -> Result<String, String> {
    let mut output = String::new();
    for node in nodes {
        match node {
            Node::Text(text) => output.push_str(&render_text(text, config)?),
            Node::Branch { arms } => {
                for arm in arms {
                    let selected = match &arm.condition {
                        None => true,
                        Some(condition) => evaluate(condition, config)
                            .map_err(|error| format!("{error} in {condition}"))?
                            .as_bool(),
                    };
                    if selected {
                        output.push_str(&render_nodes(&arm.nodes, config)?);
                        break;
                    }
                }
            }
        }
    }
    Ok(output)
}

fn render_text(text: &str, config: &mut Config) -> Result<String, String> {
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
        // Upstream supports assignment statements (`{var[idx] = expr}`) that
        // store into the parser and render nothing.
        if let Some((name, index_expression, rhs)) = assignment_parts(expression) {
            let assignment_index = match index_expression {
                Some(index_expression) => match evaluate(index_expression, config)?.as_number() {
                    Some(number) if number >= 0.0 => Some(number as usize),
                    _ => return Err(format!("invalid assignment index in {expression}")),
                },
                None => None,
            };
            let value =
                evaluate(rhs, config).map_err(|error| format!("{error} in {expression}"))?;
            config.assign(name, assignment_index, value);
            index += expression_start + relative_end + 1;
            continue;
        }
        let value =
            evaluate(expression, config).map_err(|error| format!("{error} in {expression}"))?;
        let rendered = value.index(0).unwrap_or(&value).as_string();
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
