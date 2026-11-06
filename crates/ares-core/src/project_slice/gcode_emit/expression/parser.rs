use super::super::lexer::Token;
use super::super::value::{Config, Value};
use super::ParsePath;
use super::arithmetic::{Arithmetic, arithmetic};
use super::comparison;
use super::functions::{InterpolateTableError, function, interpolate_table, is_function_keyword};

pub(super) struct Parser<'a> {
    pub(super) tokens: Vec<Token>,
    /// Byte offset of each token in the expression input.
    offsets: Vec<usize>,
    pub(super) index: usize,
    config: &'a Config,
    path: ParsePath,
    filament: usize,
    /// Byte offset of the identifier whose resolution failed, captured at
    /// the unknown-variable throw site for the caret report.
    pub(super) unknown_offset: Option<usize>,
    /// `(offset, message)` of a scalar-context violation, captured at the
    /// bare-vector throw site for the line/caret report.
    pub(super) scalar_failure: Option<(usize, &'static str)>,
    /// Depth of inactive ternary branches being parsed: evaluation failures
    /// inside them are skipped, not reported (upstream `skipping()`,
    /// `PlaceholderParser.cpp:851-871,1997-2006`).
    suppressed: u32,
}

impl Parser<'_> {
    pub(super) fn parse_or(&mut self) -> Result<Value, String> {
        let mut value = self.parse_and()?;
        while self.take(Token::Or) {
            let right = self.parse_and()?.as_bool();
            value = Value::Bool(value.as_bool() || right);
        }
        if self.take(Token::Question) {
            let condition = value.as_bool();
            // `conditional_expression` suppresses the inactive branch's
            // evaluation (`PlaceholderParser.cpp:2215-2221`): parse it for
            // structure, skip its failures, keep only the selected arm.
            let when_true = self.parse_branch(condition)?;
            if !self.take(Token::Colon) {
                return Err("expected colon in conditional expression".to_owned());
            }
            let when_false = self.parse_branch(!condition)?;
            value = if condition { when_true } else { when_false };
        }
        Ok(value)
    }

    /// Parses one ternary branch. Inside the inactive branch every
    /// evaluation action no-ops, mirroring upstream's `m_depth_suppressed`
    /// (`PlaceholderParser.cpp:851-871`); grammar failures still propagate.
    fn parse_branch(&mut self, active: bool) -> Result<Value, String> {
        if active {
            return self.parse_or();
        }
        self.suppressed += 1;
        let branch = self.parse_or();
        self.suppressed -= 1;
        branch
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
            let pattern = match regex::bytes::Regex::new(&format!("^(?-u:{})$", right.as_string()))
            {
                Ok(pattern) => pattern,
                // Suppressed (inactive) branch: `regex_op` no-ops while
                // skipping (`PlaceholderParser.cpp:687-691`).
                Err(_) if self.suppressed > 0 => return Ok(Value::Bool(false)),
                Err(error) => return Err(format!("invalid regex: {error}")),
            };
            let matched = pattern.is_match(left.as_string().as_bytes());
            if comparison == Token::RegexMatch {
                matched
            } else {
                !matched
            }
        } else {
            match comparison::compare(&left, &right, comparison) {
                Ok(result) => result,
                // Suppressed (inactive) branch: `compare_op` no-ops on
                // skipped values (`:558-562`).
                Err(_) if self.suppressed > 0 => return Ok(Value::Bool(false)),
                Err(error) => return Err(error),
            }
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
            let right = self.parse_mul()?;
            value = self.operate(value, right, operation)?;
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
            let right = self.parse_unary()?;
            value = self.operate(value, right, operation)?;
        }
    }

    /// `arithmetic` with inactive-branch suppression: upstream's operators
    /// no-op on skipped values (`PlaceholderParser.cpp:435-530`).
    fn operate(&self, left: Value, right: Value, operation: Arithmetic) -> Result<Value, String> {
        match arithmetic(left, right, operation) {
            Ok(value) => Ok(value),
            // Suppressed (inactive) branch: a dummy that is never selected.
            Err(_) if self.suppressed > 0 => Ok(Value::Bool(false)),
            Err(error) => Err(error),
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
                match value.as_number() {
                    Some(number) => Ok(Value::Number(-number)),
                    // Suppressed (inactive) branch: `unary_minus` no-ops on
                    // skipped values (`PlaceholderParser.cpp:335-340`).
                    None if self.suppressed > 0 => Ok(Value::Bool(false)),
                    None => Err("numeric expression expected".to_owned()),
                }
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
                            // Suppressed (inactive) branch: resolution no-ops
                            // while skipping; a non-identifier argument or a
                            // missing `)` stays a grammar failure.
                            _ if self.suppressed > 0 => Ok(Value::Bool(false)),
                            _ => Err("parameter of size() is not a vector variable".to_owned()),
                        };
                    }
                    let mut args = Vec::new();
                    let mut starts = Vec::new();
                    if !self.take(Token::Right) {
                        loop {
                            starts.push(self.index);
                            args.push(self.parse_or()?);
                            if self.take(Token::Right) {
                                break;
                            }
                            if !self.take(Token::Comma) {
                                return Err("expected comma in function".to_owned());
                            }
                        }
                    }
                    if name == "interpolate_table" {
                        return self.interpolate_table(&args, &starts);
                    }
                    return match function(&name, args, self.config) {
                        Ok(value) => Ok(value),
                        // Suppressed (inactive) branch: function actions no-op
                        // on skipped values (`PlaceholderParser.cpp:617-668`).
                        Err(_) if self.suppressed > 0 => Ok(Value::Bool(false)),
                        Err(error) => Err(error),
                    };
                }
                let mut value = match self.config.get(&name).cloned() {
                    Some(value) => value,
                    // Suppressed (inactive) branch: `variable_value` no-ops
                    // while skipping (`PlaceholderParser.cpp:1348-1355`).
                    None if self.suppressed > 0 => Value::Bool(false),
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
                    if values
                        .iter()
                        .all(|item| matches!(item, Value::Number(_) | Value::Nil))
                    {
                        value = values
                            .get(self.filament)
                            .or_else(|| values.first())
                            .cloned()
                            .unwrap();
                    } else if self.suppressed > 0 {
                        // Suppressed (inactive) branch:
                        // `vector_element_to_expr` no-ops while skipping
                        // (`PlaceholderParser.cpp:1092-1094`).
                        value = Value::Bool(false);
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
                    let index = match self.parse_or()?.as_number() {
                        Some(number) => Some(number as usize),
                        // Suppressed (inactive) branch: `evaluate_index`
                        // no-ops while skipping (`:1699-1703`).
                        None if self.suppressed > 0 => None,
                        None => return Err("index must be numeric".to_owned()),
                    };
                    if !self.take(Token::RightBracket) {
                        return Err("expected closing index bracket".to_owned());
                    }
                    value = match index {
                        Some(index) => match value.index(index) {
                            Some(indexed) => indexed.clone(),
                            // Suppressed (inactive) branch: indexing failures
                            // no-op while skipping.
                            None if self.suppressed > 0 => Value::Bool(false),
                            None => return Err(format!("placeholder index out of range: {name}")),
                        },
                        None => Value::Bool(false),
                    };
                }
                if self.suppressed == 0 {
                    value.ensure_defined()?;
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

    /// `interpolate_table` failures report at the x expression
    /// (`expr_x.it_range`, `PlaceholderParser.cpp:2007`) or at the earlier
    /// point's x (`it_range_x`, `:1982-1984`); `starts` holds each
    /// argument's first token index.
    fn interpolate_table(&mut self, args: &[Value], starts: &[usize]) -> Result<Value, String> {
        let (token, message) = match interpolate_table(args) {
            Ok(value) => return Ok(value),
            // Suppressed (inactive) branch: no evaluation failure.
            Err(_) if self.suppressed > 0 => return Ok(Value::Bool(false)),
            Err(InterpolateTableError::Invalid(message)) => return Err(message.to_owned()),
            Err(InterpolateTableError::OutsideRange) => {
                (starts[0], "Interpolation value is outside the table range")
            }
            Err(InterpolateTableError::NotIncreasing { point }) => (
                starts[point + 1] + 1,
                "X coordinates of the table must be increasing",
            ),
        };
        self.scalar_failure = Some((self.offsets[token], message));
        Err(message.to_owned())
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

impl<'a> Parser<'a> {
    pub(super) fn new(
        tokens: Vec<Token>,
        offsets: Vec<usize>,
        config: &'a Config,
        path: ParsePath,
        filament: usize,
    ) -> Self {
        Self {
            tokens,
            offsets,
            index: 0,
            config,
            path,
            filament,
            unknown_offset: None,
            scalar_failure: None,
            suppressed: 0,
        }
    }
}
