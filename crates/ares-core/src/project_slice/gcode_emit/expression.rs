use super::{
    lexer::{Token, tokenize},
    value::{Config, Value},
};

mod arithmetic;
mod comparison;
mod functions;
#[cfg(test)]
mod tests;

use arithmetic::{Arithmetic, arithmetic};
use functions::{function, is_function_keyword};

pub(super) use arithmetic::to_string;

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
    let tokens = tokenize(input).map_err(PathError::Other)?;
    let mut parser = Parser {
        tokens,
        index: 0,
        config,
        path,
        filament,
        unknown_offset: None,
        scalar_failure: None,
    };
    match parser.parse_or() {
        Ok(value) if parser.index == parser.tokens.len() => Ok(value),
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
    let mut parser = Parser {
        tokens: tokenize(input)?,
        index: 0,
        config,
        path: ParsePath::Brace,
        filament,
        unknown_offset: None,
        scalar_failure: None,
    };
    let value = parser.parse_or()?;
    if parser.index != parser.tokens.len() {
        return Err(format!("unexpected token in expression: {input}"));
    }
    Ok(value)
}

struct Parser<'a> {
    tokens: Vec<Token>,
    index: usize,
    config: &'a Config,
    path: ParsePath,
    filament: usize,
    /// Byte offset of the identifier whose resolution failed, captured at
    /// the unknown-variable throw site for the caret report.
    unknown_offset: Option<usize>,
    /// `(offset, message)` of a scalar-context violation, captured at the
    /// bare-vector throw site for the line/caret report.
    scalar_failure: Option<(usize, &'static str)>,
}

impl Parser<'_> {
    fn parse_or(&mut self) -> Result<Value, String> {
        let mut value = self.parse_and()?;
        while self.take(Token::Or) {
            let right = self.parse_and()?.as_bool();
            value = Value::Bool(value.as_bool() || right);
        }
        if self.take(Token::Question) {
            let when_true = self.parse_or()?;
            if !self.take(Token::Colon) {
                return Err("expected colon in conditional expression".to_owned());
            }
            let when_false = self.parse_or()?;
            value = if value.as_bool() {
                when_true
            } else {
                when_false
            };
        }
        Ok(value)
    }

    fn parse_and(&mut self) -> Result<Value, String> {
        let mut value = self.parse_compare()?;
        while self.take(Token::And) {
            let right = self.parse_compare()?.as_bool();
            value = Value::Bool(value.as_bool() && right);
        }
        Ok(value)
    }

    fn parse_compare(&mut self) -> Result<Value, String> {
        let left = self.parse_add()?;
        let comparison = match self.peek() {
            Some(Token::Eq) => Some(Token::Eq),
            Some(Token::NotEq) => Some(Token::NotEq),
            Some(Token::RegexMatch) => Some(Token::RegexMatch),
            Some(Token::RegexNotMatch) => Some(Token::RegexNotMatch),
            Some(Token::Lt) => Some(Token::Lt),
            Some(Token::Le) => Some(Token::Le),
            Some(Token::Gt) => Some(Token::Gt),
            Some(Token::Ge) => Some(Token::Ge),
            _ => None,
        };
        let Some(comparison) = comparison else {
            return Ok(left);
        };
        self.index += 1;
        let right = self.parse_add()?;
        let result = if matches!(comparison, Token::RegexMatch | Token::RegexNotMatch) {
            let matched = regex::bytes::Regex::new(&format!("^(?-u:{})$", right.as_string()))
                .map_err(|error| format!("invalid regex: {error}"))?
                .is_match(left.as_string().as_bytes());
            if comparison == Token::RegexMatch {
                matched
            } else {
                !matched
            }
        } else {
            comparison::compare(&left, &right, comparison)?
        };
        Ok(Value::Bool(result))
    }

    fn parse_add(&mut self) -> Result<Value, String> {
        let mut value = self.parse_mul()?;
        loop {
            let operation = if self.take(Token::Plus) {
                Arithmetic::Add
            } else if self.take(Token::Minus) {
                Arithmetic::Subtract
            } else {
                return Ok(value);
            };
            value = arithmetic(value, self.parse_mul()?, operation)?;
        }
    }

    fn parse_mul(&mut self) -> Result<Value, String> {
        let mut value = self.parse_unary()?;
        loop {
            let operation = if self.take(Token::Star) {
                Arithmetic::Multiply
            } else if self.take(Token::Slash) {
                Arithmetic::Divide
            } else if self.take(Token::Percent) {
                Arithmetic::Remainder
            } else {
                return Ok(value);
            };
            value = arithmetic(value, self.parse_unary()?, operation)?;
        }
    }

    fn parse_unary(&mut self) -> Result<Value, String> {
        if self.take(Token::Not) {
            return Ok(Value::Bool(!self.parse_unary()?.as_bool()));
        }
        if self.take(Token::Plus) {
            return self.parse_unary();
        }
        if self.take(Token::Minus) {
            let value = self.parse_unary()?;
            return if let Some(value) = value.as_integer() {
                Ok(Value::Integer(-value))
            } else {
                Ok(Value::Number(
                    -value.as_number().ok_or("numeric expression expected")?,
                ))
            };
        }
        self.parse_primary()
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "keeps expression grammar parsing ordered"
    )]
    fn parse_primary(&mut self) -> Result<Value, String> {
        let token = self.next().ok_or("expression ended unexpectedly")?;
        match token {
            Token::Integer(value) => Ok(Value::Integer(value)),
            Token::Number(value) => Ok(Value::Number(value)),
            Token::Bool(value) => Ok(Value::Bool(value)),
            Token::String(value) | Token::Regex(value) => Ok(Value::String(value)),
            Token::Ident(name, offset) => {
                if is_function_keyword(&name) && self.take(Token::Left) {
                    // PlaceholderParser.cpp:1687,2333: size takes an unindexed
                    // vector variable, not a scalar or an arbitrary expression.
                    if name == "size" {
                        let Some(Token::Ident(variable, _)) = self.next() else {
                            return Err("parameter of size() is not a vector variable".to_owned());
                        };
                        if !self.take(Token::Right) {
                            return Err("parameter of size() is not a vector variable".to_owned());
                        }
                        return match self.config.get(&variable) {
                            Some(Value::List(values)) => Ok(Value::Integer(values.len() as i64)),
                            _ => Err("parameter of size() is not a vector variable".to_owned()),
                        };
                    }
                    let mut args = Vec::new();
                    if !self.take(Token::Right) {
                        loop {
                            args.push(self.parse_or()?);
                            if self.take(Token::Right) {
                                break;
                            }
                            if !self.take(Token::Comma) {
                                return Err("expected comma in function".to_owned());
                            }
                        }
                    }
                    return function(&name, args, self.config);
                }
                let mut value = match self.config.get(&name).cloned() {
                    Some(value) => value,
                    None => {
                        self.unknown_offset = Some(offset);
                        return Err(format!("unknown placeholder: {name}"));
                    }
                };
                // `vector_element_to_expr` (`PlaceholderParser.cpp:1166-1211`):
                // a bare (non-indexed) vector reference only resolves for
                // float vectors — `size() == 1 ? get_at(0) :
                // get_at(get_extruder_id())` — while every other vector
                // type throws `:1204`. The legacy `[var]` spelling keeps
                // whole-vector expansion (`:915`).
                if self.path == ParsePath::Brace
                    && !matches!(self.peek(), Some(Token::LeftBracket))
                    && let Value::List(values) = &value
                    && !values.is_empty()
                {
                    if values.iter().all(|item| matches!(item, Value::Number(_))) {
                        value = values
                            .get(self.filament)
                            .or_else(|| values.first())
                            .cloned()
                            .unwrap();
                    } else {
                        self.scalar_failure = Some((
                            offset,
                            "Referencing a vector variable when scalar is expected",
                        ));
                        return Err(
                            "Referencing a vector variable when scalar is expected".to_owned()
                        );
                    }
                }
                while self.take(Token::LeftBracket) {
                    let index = self
                        .parse_or()?
                        .as_number()
                        .ok_or("index must be numeric")? as usize;
                    if !self.take(Token::RightBracket) {
                        return Err("expected closing index bracket".to_owned());
                    }
                    value = value
                        .index(index)
                        .cloned()
                        .ok_or_else(|| format!("placeholder index out of range: {name}"))?;
                }
                Ok(value)
            }
            Token::Left => {
                let first = self.parse_or()?;
                if self.take(Token::Comma) {
                    let mut values = vec![first];
                    loop {
                        values.push(self.parse_or()?);
                        if self.take(Token::Right) {
                            break;
                        }
                        if !self.take(Token::Comma) {
                            return Err("expected comma in tuple".to_owned());
                        }
                    }
                    Ok(Value::List(values))
                } else {
                    if !self.take(Token::Right) {
                        return Err("expected closing parenthesis".to_owned());
                    }
                    Ok(first)
                }
            }
            _ => Err("primary expression expected".to_owned()),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned();
        self.index += usize::from(token.is_some());
        token
    }

    fn take(&mut self, expected: Token) -> bool {
        if self.peek() == Some(&expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }
}
