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
    let (tokens, offsets) = tokenize(input).map_err(PathError::Other)?;
    let mut parser = Parser::new(tokens, offsets, config, path, filament);
    match parser.parse_or() {
        Ok(value) if parser.index == parser.tokens.len() => Ok(value.value),
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

/// Evaluates a brace-context expression (`{if …}` conditions and
/// assignment right-hand sides) under the same variable semantics.
pub(crate) fn evaluate(input: &str, config: &Config, filament: usize) -> Result<Value, String> {
    let (tokens, offsets) = tokenize(input)?;
    let mut parser = Parser::new(tokens, offsets, config, ParsePath::Brace, filament);
    let value = parser.parse_or()?.value;
    if parser.index != parser.tokens.len() {
        return Err(format!("unexpected token in expression: {input}"));
    }
    Ok(value)
}
