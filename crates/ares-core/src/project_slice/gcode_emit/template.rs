use super::{expression::evaluate, value::Config};

pub(super) fn render(template: &str, config: &Config) -> Result<String, String> {
    let owned_lines = coalesce_directives(template);
    let lines = owned_lines.iter().map(String::as_str).collect::<Vec<_>>();
    let (output, next) = render_range(&lines, 0, lines.len(), config)?;
    if next != lines.len() {
        return Err("unbalanced template directive".to_owned());
    }
    Ok(output)
}

#[expect(
    clippy::excessive_nesting,
    reason = "keeps multiline directive scanning ordered"
)]
fn coalesce_directives(template: &str) -> Vec<String> {
    let raw_lines = template.split_inclusive('\n').collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut index = 0;
    while index < raw_lines.len() {
        let mut line = raw_lines[index].to_owned();
        let trimmed = line.trim_start().trim_end_matches(['\r', '\n']);
        if (trimmed.starts_with("{if") || trimmed.starts_with("{elsif")) && !trimmed.ends_with('}')
        {
            line = line.trim_end_matches(['\r', '\n']).to_owned();
            index += 1;
            while index < raw_lines.len() {
                let next = raw_lines[index].trim();
                line.push_str(next);
                if next.ends_with('}') {
                    break;
                }
                index += 1;
            }
        }
        lines.push(line);
        index += 1;
    }
    lines
}

#[expect(
    clippy::excessive_nesting,
    reason = "keeps template branch rendering ordered"
)]
fn render_range(
    lines: &[&str],
    mut index: usize,
    end: usize,
    config: &Config,
) -> Result<(String, usize), String> {
    let mut output = String::new();
    while index < end {
        let line = lines[index];
        if let Some(expression) = directive(line, "if") {
            let (branches, next) = find_branches(lines, index + 1, end, expression)?;
            let mut selected = None;
            for (condition, start, branch_end) in branches {
                if condition.is_empty()
                    || evaluate(&condition, config)
                        .map_err(|error| format!("{error} in {condition}"))?
                        .as_bool()
                {
                    selected = Some((start, branch_end));
                    break;
                }
            }
            if let Some((start, branch_end)) = selected {
                output.push_str(&directive_blank(lines[start - 1]));
                output.push_str(&render_range(lines, start, branch_end, config)?.0);
            }
            output.push_str(&directive_blank(lines[next - 1]));
            index = next;
            continue;
        }
        if directive(line, "else").is_some()
            || directive(line, "elsif").is_some()
            || directive(line, "endif").is_some()
        {
            return Err("unexpected template directive".to_owned());
        }
        output.push_str(&replace_line(line, config)?);
        index += 1;
    }
    Ok((output, index))
}

type Branch = (String, usize, usize);

fn find_branches(
    lines: &[&str],
    mut index: usize,
    end: usize,
    initial_condition: &str,
) -> Result<(Vec<Branch>, usize), String> {
    let mut branches = Vec::new();
    let mut depth = 0;
    let mut condition = initial_condition.trim().to_owned();
    let mut start = index;
    while index < end {
        let line = lines[index];
        if let Some(expression) = directive(line, "if") {
            depth += 1;
            let _ = expression;
        } else if directive(line, "endif").is_some() {
            if depth == 0 {
                branches.push((std::mem::take(&mut condition), start, index));
                return Ok((branches, index + 1));
            }
            depth -= 1;
        } else if depth == 0 {
            if let Some(expression) = directive(line, "elsif") {
                branches.push((std::mem::take(&mut condition), start, index));
                condition = expression.to_owned();
                start = index + 1;
            } else if directive(line, "else").is_some() {
                branches.push((std::mem::take(&mut condition), start, index));
                condition.clear();
                start = index + 1;
            }
        }
        index += 1;
    }
    Err("missing endif".to_owned())
}

fn directive<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let line = line.trim_end_matches(['\r', '\n']).trim();
    let prefix = format!("{{{name}");
    line.strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix('}'))
        .map(str::trim)
}

fn directive_blank(line: &str) -> String {
    let indentation = line.len() - line.trim_start().len();
    format!("{}\n", &line[..indentation])
}

fn replace_line(line: &str, config: &Config) -> Result<String, String> {
    let mut output = String::new();
    let mut index = 0;
    while index < line.len() {
        let remaining = &line[index..];
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
        let Some(relative_end) = placeholder_end(line, expression_start, delimiter) else {
            return Err(format!("unclosed placeholder in {line:?}"));
        };
        let expression = &line[expression_start..expression_start + relative_end];
        let value =
            evaluate(expression, config).map_err(|error| format!("{error} in {expression}"))?;
        let rendered = value.index(0).unwrap_or(&value).as_string();
        output.push_str(&rendered);
        index = expression_start + relative_end + 1;
    }
    Ok(output)
}

fn placeholder_end(line: &str, start: usize, delimiter: char) -> Option<usize> {
    if delimiter != '[' {
        return line[start..].find('}');
    }
    let mut depth = 0;
    for (offset, character) in line[start..].char_indices() {
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
mod tests {
    use super::*;
    use crate::project_slice::gcode_emit::value::Config;

    #[test]
    fn renderer_selects_nested_branches_and_replaces_values() {
        let config = Config::from_block(b"; enabled = 1\n; n = 2\n");
        let template = "{if enabled}\nA [n]\n{if n > 1}\nB\n{endif}\n{else}\nC\n{endif}\n";
        assert_eq!(render(template, &config).unwrap(), "\nA 2\n\nB\n\n\n");
    }

    #[test]
    fn renderer_coalesces_multiline_conditions_and_selects_else() {
        let config = Config::from_block(b"; enabled = 0\n; n = 2\n");
        let template = "{if enabled == 1 ||\n n == 3}\nA\n{else}\nB [n]\n{endif}\n";
        assert_eq!(render(template, &config).unwrap(), "\nB 2\n\n");
    }

    #[test]
    fn renderer_keeps_closing_blank_only_for_selected_single_branch() {
        let config = Config::from_block(b"; enabled = 1\n");
        let template = "{if enabled}\nA\n{endif}\n{if !enabled}\nB\n{endif}\n";
        assert_eq!(render(template, &config).unwrap(), "\nA\n\n\n");
    }

    #[test]
    fn renderer_keeps_only_the_closing_newline_when_no_branch_matches() {
        let config = Config::from_block(b"; enabled = 0\n");
        let template = "{if enabled}\nA\n{elsif enabled == 2}\nB\n{endif}\n";

        assert_eq!(render(template, &config).unwrap(), "\n");
    }
}
