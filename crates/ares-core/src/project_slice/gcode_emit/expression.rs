use super::{
    lexer::tokenize,
    value::{Config, Value},
};

mod arithmetic;
mod comparison;
mod deferred;
mod functions;
mod parser;
mod type_check;

pub(super) use arithmetic::to_string;
use parser::Parser;

/// The placeholder parse path an evaluation runs under. Upstream's grammar
/// routes the two placeholder spellings into separate resolution paths, and
/// each path reports unknown symbols with its own message:
/// - brace placeholders resolve symbols via `resolve_variable`, which throws
///   "Not a variable name" (`PlaceholderParser.cpp:988-1000`);
/// - legacy bracket placeholders expand via `legacy_variable_expansion`,
///   which throws "Variable does not exist" (`PlaceholderParser.cpp:918-932`).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ParsePath {
    Brace,
    Legacy,
}

impl ParsePath {
    fn unknown_variable_message(self) -> &'static str {
        match self {
            Self::Brace => "Not a variable name",
            Self::Legacy => "Variable does not exist",
        }
    }
}

/// Unknown-symbol resolution failure. `offset` is the failing identifier's
/// byte offset in the expression input; it feeds the upstream line/caret
/// report (`PlaceholderParser.cpp:1726-1770`).
pub(crate) struct ParseFailure {
    pub(crate) message: String,
    pub(crate) offset: usize,
}

pub(crate) enum PathError {
    Parse(ParseFailure),
    Other(String),
}

impl PathError {
    /// Names the failing expression in a message that has no caret report.
    pub(crate) fn in_expression(self, expression: &str) -> Self {
        match self {
            Self::Other(message) => Self::Other(format!("{message} in {expression}")),
            parse => parse,
        }
    }

    /// Moves the caret offset of a failure by `distance` bytes.
    pub(crate) fn shifted(self, distance: usize) -> Self {
        match self {
            Self::Parse(failure) => Self::Parse(ParseFailure {
                offset: failure.offset + distance,
                ..failure
            }),
            other => other,
        }
    }
}

/// Evaluates `input` under one of the upstream placeholder parse paths so
/// unknown symbols surface with the path's message and identifier offset.
/// `filament` is the process call's extruder context: the bare-vector
/// auto-serialization of `vector_element_to_expr` selects
/// `get_at(get_extruder_id())` (`PlaceholderParser.cpp:1191-1201`).
pub(crate) fn evaluate_path(
    input: &str,
    config: &Config,
    path: ParsePath,
    filament: usize,
) -> Result<Value, PathError> {
    operand(input, config, path, filament).map(|operand| operand.value)
}

/// An `{if}` or `{elsif}` condition (`bool_expr_eval`, `PlaceholderParser.cpp:2408`): it must be
/// a boolean (`evaluate_boolean`, `:604-612`).
pub(crate) fn evaluate_condition(
    input: &str,
    config: &Config,
    filament: usize,
) -> Result<bool, PathError> {
    let operand = operand(input, config, ParsePath::Brace, filament)?;
    type_check::condition(&operand).map_err(type_failure)?;
    Ok(operand.value.as_bool())
}

/// The right-hand side assigned to an output variable, in the type of `variable`
/// (`scalar_variable_assign_scalar`, `PlaceholderParser.cpp:1396-1426`).
pub(crate) fn evaluate_assigned(
    input: &str,
    config: &Config,
    filament: usize,
    variable: &Value,
) -> Result<Value, PathError> {
    let operand = operand(input, config, ParsePath::Brace, filament)?;
    type_check::assigned(variable, &operand).map_err(type_failure)
}

fn type_failure((offset, message): type_check::Failure) -> PathError {
    PathError::Parse(ParseFailure {
        message: message.to_owned(),
        offset,
    })
}

fn operand(
    input: &str,
    config: &Config,
    path: ParsePath,
    filament: usize,
) -> Result<deferred::Operand, PathError> {
    let (tokens, offsets) = tokenize(input).map_err(PathError::Other)?;
    let mut parser = Parser::new(tokens, offsets, config, path, filament);
    match parser.parse_or() {
        Ok(operand) if parser.index == parser.tokens.len() => Ok(operand),
        Ok(_) => Err(PathError::Other(format!(
            "unexpected token in expression: {input}"
        ))),
        Err(error) => {
            if let Some((offset, message)) = parser.scalar_failure {
                return Err(PathError::Parse(ParseFailure {
                    message: message.to_owned(),
                    offset,
                }));
            }
            match parser.unknown_offset {
                Some(offset) => Err(PathError::Parse(ParseFailure {
                    message: path.unknown_variable_message().to_owned(),
                    offset,
                })),
                None => Err(PathError::Other(error)),
            }
        }
    }
}

/// Evaluates the index of an assigned vector element under the brace variable semantics.
pub(crate) fn evaluate(input: &str, config: &Config, filament: usize) -> Result<Value, String> {
    let (tokens, offsets) = tokenize(input)?;
    let mut parser = Parser::new(tokens, offsets, config, ParsePath::Brace, filament);
    let value = parser.parse_or()?.value;
    if parser.index != parser.tokens.len() {
        return Err(format!("unexpected token in expression: {input}"));
    }
    Ok(value)
}
