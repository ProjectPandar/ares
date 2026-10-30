//! Inline macro-block statement grammar for the inside of one `{...}`
//! block (`PlaceholderParser.cpp:2157-2186,2162-2174,2287-2300`):
//! `;`-separated statements, `local`/`global` declarations and inline
//! `if/then/elsif/else/endif` blocks. Only the selected arm of an inline
//! `if` is evaluated, mirroring upstream's conditional suppression.

use super::super::expression::{evaluate, to_string};
use super::super::value::{Config, Value};

/// Whether a `{...}` body uses the statement grammar: a top-level `;`, a
/// leading `local`/`global` declaration, or an inline `if ... then` block.
pub(super) fn is_macro_body(body: &str) -> bool {
    find_top_level(body, ";").is_some()
        || starts_keyword(body, "local")
        || starts_keyword(body, "global")
        || (starts_keyword(body, "if") && find_keyword(body, "then").is_some())
}

/// Whether an `{if` condition area contains a top-level `then` keyword,
/// marking an inline macro block instead of a text-form directive
/// (`if_macros`, `PlaceholderParser.cpp:2157`).
pub(super) fn has_top_level_then(condition: &str) -> bool {
    find_keyword(condition, "then").is_some()
}

/// Evaluates the statement list of one `{...}` block and returns its
/// output (`to_string2` per expression statement, `:532-537`).
/// `journal` records output variables created by `local`/`global` so the
/// caller can drop them when the template render ends (upstream
/// `config_local` lifetime, `:889-906`).
pub(super) fn evaluate_block(
    body: &str,
    config: &mut Config,
    filament: usize,
    journal: &mut Vec<(String, Option<Value>)>,
) -> Result<String, String> {
    let (statements, rest) = parse_statements(body, &[])?;
    if !rest.trim_start().is_empty() {
        return Err(format!("unexpected {rest:?} in macro block"));
    }
    let mut output = String::new();
    evaluate_statements(&statements, config, filament, journal, &mut output)?;
    Ok(output)
}

enum Statement {
    Expression(String),
    Assignment {
        name: String,
        index: Option<String>,
        rhs: String,
    },
    Declare {
        name: String,
        rhs: String,
    },
    If(Vec<Arm>),
}

struct Arm {
    condition: Option<String>,
    statements: Vec<Statement>,
}

/// Parses statements until a stop keyword (`elsif`/`else`/`endif`) or the
/// end of input; the returned rest starts at the stop keyword.
fn parse_statements<'a>(
    input: &'a str,
    stops: &[&str],
) -> Result<(Vec<Statement>, &'a str), String> {
    let mut statements = Vec::new();
    let mut rest = input;
    loop {
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            return Ok((statements, trimmed));
        }
        if stops.iter().any(|stop| starts_keyword(trimmed, stop)) {
            return Ok((statements, trimmed));
        }
        if let Some(tail) = trimmed.strip_prefix(';') {
            rest = tail;
            continue;
        }
        if starts_keyword(trimmed, "if") {
            let (statement, tail) = parse_if(trimmed)?;
            statements.push(statement);
            rest = tail;
            continue;
        }
        let end = statement_end(trimmed, stops);
        statements.push(parse_statement(trimmed[..end].trim())?);
        rest = &trimmed[end..];
    }
}

/// Extent of one non-`if` statement: the next top-level `;`, a stop
/// keyword or the end of input.
fn statement_end(input: &str, stops: &[&str]) -> usize {
    let mut end = find_top_level(input, ";").unwrap_or(input.len());
    for stop in stops {
        if let Some(offset) = find_keyword(input, stop) {
            end = end.min(offset);
        }
    }
    end
}

/// Parses one inline `if COND then ... (elsif COND then ...)*
/// (else ...)? endif` block; `input` starts at the `if` keyword.
fn parse_if(input: &str) -> Result<(Statement, &str), String> {
    let after_if = &input["if".len()..];
    let then =
        find_keyword(after_if, "then").ok_or_else(|| "expected then in macro if".to_owned())?;
    let condition = after_if[..then].trim().to_owned();
    let mut rest = &after_if[then + "then".len()..];
    let mut arms = Vec::new();
    let (statements, tail) = parse_statements(rest, &["elsif", "else", "endif"])?;
    arms.push(Arm {
        condition: Some(condition),
        statements,
    });
    rest = tail;
    loop {
        if starts_keyword(rest, "elsif") {
            // `kw["elsif"] > bool_expr > if_macros` also requires `then`.
            let after_elsif = &rest["elsif".len()..];
            let then = find_keyword(after_elsif, "then")
                .ok_or_else(|| "expected then in macro elsif".to_owned())?;
            let condition = after_elsif[..then].trim().to_owned();
            let (statements, tail) = parse_statements(
                &after_elsif[then + "then".len()..],
                &["elsif", "else", "endif"],
            )?;
            arms.push(Arm {
                condition: Some(condition),
                statements,
            });
            rest = tail;
            continue;
        }
        if starts_keyword(rest, "else") {
            let (statements, tail) = parse_statements(&rest["else".len()..], &["endif"])?;
            arms.push(Arm {
                condition: None,
                statements,
            });
            rest = tail;
        }
        if starts_keyword(rest, "endif") {
            return Ok((Statement::If(arms), &rest["endif".len()..]));
        }
        return Err("expected elsif, else or endif in macro if".to_owned());
    }
}

fn parse_statement(statement: &str) -> Result<Statement, String> {
    for keyword in ["local", "global"] {
        if starts_keyword(statement, keyword) {
            let rest = &statement[keyword.len()..];
            let equals = rest
                .find('=')
                .ok_or_else(|| format!("expected '=' in {keyword} declaration of {rest:?}"))?;
            let name = rest[..equals].trim();
            let valid = !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
                && !name
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_digit());
            if !valid {
                return Err(format!("invalid variable name {name:?} in {statement}"));
            }
            return Ok(Statement::Declare {
                name: name.to_owned(),
                rhs: rest[equals + 1..].trim().to_owned(),
            });
        }
    }
    if let Some((name, index, rhs)) = super::assignment_parts(statement) {
        return Ok(Statement::Assignment {
            name: name.to_owned(),
            index: index.map(str::to_owned),
            rhs: rhs.to_owned(),
        });
    }
    Ok(Statement::Expression(statement.to_owned()))
}

/// Evaluates an assignment's optional index expression to a valid
/// vector index (`assignment_statement`, `:2170-2174`).
fn assignment_index(
    name: &str,
    index: Option<&str>,
    config: &Config,
    filament: usize,
) -> Result<Option<usize>, String> {
    let Some(index) = index else {
        return Ok(None);
    };
    match evaluate(index, config, filament)?.as_number() {
        Some(number) if number >= 0.0 => Ok(Some(number as usize)),
        _ => Err(format!("invalid assignment index in {name}[{index}]")),
    }
}

/// First arm of an inline `if` whose condition holds: arms are
/// evaluated in order and the `else` arm has no condition
/// (`if_else_output`, `:2178-2186`).
fn selected_arm<'a>(
    arms: &'a [Arm],
    config: &Config,
    filament: usize,
) -> Result<Option<&'a [Statement]>, String> {
    for arm in arms {
        let selected = match &arm.condition {
            None => true,
            Some(condition) => evaluate(condition, config, filament)
                .map_err(|error| format!("{error} in {condition}"))?
                .as_bool(),
        };
        if selected {
            return Ok(Some(&arm.statements));
        }
    }
    Ok(None)
}

/// Evaluates a `;`-separated statement list, appending rendered
/// expression statements to `output` (`PlaceholderParser.cpp:2157-2186`).
fn evaluate_statements(
    statements: &[Statement],
    config: &mut Config,
    filament: usize,
    journal: &mut Vec<(String, Option<Value>)>,
    output: &mut String,
) -> Result<(), String> {
    for statement in statements {
        match statement {
            Statement::Expression(expression) => {
                let value = evaluate(expression, config, filament)
                    .map_err(|error| format!("{error} in {expression}"))?;
                output.push_str(&to_string(&value));
            }
            Statement::Assignment { name, index, rhs } => {
                let value =
                    evaluate(rhs, config, filament).map_err(|error| format!("{error} in {rhs}"))?;
                let index = assignment_index(name, index.as_deref(), config, filament)?;
                config.assign(name, index, value);
            }
            Statement::Declare { name, rhs } => {
                let value =
                    evaluate(rhs, config, filament).map_err(|error| format!("{error} in {rhs}"))?;
                // `new_old_variable` (`:1384-1448`): a name that already
                // resolves assigns that variable in place; a new name
                // becomes an output variable restored after the render.
                if config.get(name).is_none() {
                    journal.push((name.clone(), None));
                }
                config.assign(name, None, value);
            }
            Statement::If(arms) => {
                if let Some(statements) = selected_arm(arms, config, filament)? {
                    evaluate_statements(statements, config, filament, journal, output)?;
                }
            }
        }
    }
    Ok(())
}

fn is_word(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn starts_keyword(input: &str, keyword: &str) -> bool {
    input.strip_prefix(keyword).is_some_and(|tail| {
        tail.chars()
            .next()
            .is_none_or(|character| !is_word(character))
    })
}

/// Byte offset of `needle` outside double-quoted strings and bracket
/// nesting (`kw = distinct(alnum | '_')` boundaries are the caller's job).
fn find_top_level(input: &str, needle: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    let mut offset = 0;
    while offset < bytes.len() {
        let character = bytes[offset];
        if in_string {
            if escaped {
                escaped = false;
            } else if character == b'\\' {
                escaped = true;
            } else if character == b'"' {
                in_string = false;
            }
            offset += 1;
            continue;
        }
        match character {
            b'"' => in_string = true,
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            _ if depth == 0 && bytes[offset..].starts_with(needle.as_bytes()) => {
                return Some(offset);
            }
            _ => {}
        }
        offset += 1;
    }
    None
}

/// Byte offset of `keyword` at top level with word boundaries on both
/// sides (`kw = distinct(alnum | '_')`, `PlaceholderParser.cpp:2110`).
fn find_keyword(input: &str, keyword: &str) -> Option<usize> {
    let mut search = 0;
    while let Some(offset) = find_top_level(&input[search..], keyword) {
        let offset = search + offset;
        let before_ok = input[..offset]
            .chars()
            .next_back()
            .is_none_or(|character| !is_word(character));
        let after = &input[offset + keyword.len()..];
        if before_ok && after.chars().next().is_none_or(|c| !is_word(c)) {
            return Some(offset);
        }
        search = offset + 1;
    }
    None
}
