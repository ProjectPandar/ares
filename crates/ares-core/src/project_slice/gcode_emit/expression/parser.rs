use super::super::lexer::Token;
use super::super::value::{Config, Value};
use super::ParsePath;
use super::arithmetic::{Arithmetic, arithmetic};
use super::comparison;
use super::deferred::{self, Operand};
use super::functions::is_function_keyword;

mod call;
mod variable;

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
    /// `(offset, message)` of a scalar-context violation or a deferred
    /// operand, captured at the throw site for the line/caret report.
    pub(super) scalar_failure: Option<(usize, &'static str)>,
    /// Depth of inactive ternary branches being parsed: evaluation failures
    /// inside them are skipped, not reported (upstream `skipping()`,
    /// `PlaceholderParser.cpp:851-871,1997-2006`).
    suppressed: u32,
}

impl Parser<'_> {
    pub(super) fn parse_or(&mut self) -> Result<Operand, String> {
        let mut value = self.parse_and()?;
        while self.take(Token::Or) {
            let right = self.parse_and()?;
            self.reject_deferred(&[&value, &right])?;
            let result = value.value.as_bool() || right.value.as_bool();
            value = Operand::plain(Value::Bool(result), value.start);
        }
        if self.take(Token::Question) {
            // `evaluate_boolean` (`PlaceholderParser.cpp:604-612`) rejects a deferred condition.
            self.reject_deferred(&[&value])?;
            let condition = value.value.as_bool();
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
    fn parse_branch(&mut self, active: bool) -> Result<Operand, String> {
        if active {
            return self.parse_or();
        }
        self.suppressed += 1;
        let branch = self.parse_or();
        self.suppressed -= 1;
        branch
    }

    fn parse_and(&mut self) -> Result<Operand, String> {
        let mut value = self.parse_compare()?;
        while self.take(Token::And) {
            let right = self.parse_compare()?;
            self.reject_deferred(&[&value, &right])?;
            let result = value.value.as_bool() && right.value.as_bool();
            value = Operand::plain(Value::Bool(result), value.start);
        }
        Ok(value)
    }

    fn parse_compare(&mut self) -> Result<Operand, String> {
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
            // `regex_op` (`PlaceholderParser.cpp:754-759`) rejects a deferred subject.
            self.reject_deferred(&[&left])?;
            let pattern =
                match regex::bytes::Regex::new(&format!("^(?-u:{})$", right.value.as_string())) {
                    Ok(pattern) => pattern,
                    // Suppressed (inactive) branch: `regex_op` no-ops while
                    // skipping (`PlaceholderParser.cpp:687-691`).
                    Err(_) if self.suppressed > 0 => {
                        return Ok(Operand::plain(Value::Bool(false), left.start));
                    }
                    Err(error) => return Err(format!("invalid regex: {error}")),
                };
            let matched = pattern.is_match(left.value.as_string().as_bytes());
            if comparison == Token::RegexMatch {
                matched
            } else {
                !matched
            }
        } else {
            // `compare_op` (`PlaceholderParser.cpp:628-629`) rejects deferred operands.
            self.reject_deferred(&[&left, &right])?;
            match comparison::compare(&left.value, &right.value, comparison) {
                Ok(result) => result,
                // Suppressed (inactive) branch: `compare_op` no-ops on
                // skipped values (`:558-562`).
                Err(_) if self.suppressed > 0 => {
                    return Ok(Operand::plain(Value::Bool(false), left.start));
                }
                Err(error) => return Err(error),
            }
        };
        Ok(Operand::plain(Value::Bool(result), left.start))
    }

    fn parse_add(&mut self) -> Result<Operand, String> {
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

    fn parse_mul(&mut self) -> Result<Operand, String> {
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
    fn operate(
        &mut self,
        left: Operand,
        right: Operand,
        operation: Arithmetic,
    ) -> Result<Operand, String> {
        if self.suppressed == 0
            && let Some(start) = deferred::arithmetic_failure(&left, &right, operation)
        {
            return Err(self.deferred_failure(start));
        }
        let deferred = deferred::sum(&left, &right, operation);
        let start = left.start;
        match arithmetic(left.value, right.value, operation) {
            Ok(value) => Ok(Operand {
                value,
                deferred,
                start,
            }),
            // Suppressed (inactive) branch: a dummy that is never selected.
            Err(_) if self.suppressed > 0 => Ok(Operand::plain(Value::Bool(false), start)),
            Err(error) => Err(error),
        }
    }

    fn parse_unary(&mut self) -> Result<Operand, String> {
        if self.take(Token::Not) {
            let start = self.offsets[self.index - 1];
            let operand = self.parse_unary()?;
            self.reject_deferred(&[&operand])?;
            return Ok(Operand::plain(Value::Bool(!operand.value.as_bool()), start));
        }
        if self.take(Token::Plus) {
            let start = self.offsets[self.index - 1];
            return Ok(Operand {
                start,
                ..self.parse_unary()?
            });
        }
        if self.take(Token::Minus) {
            let start = self.offsets[self.index - 1];
            let operand = self.parse_unary()?;
            self.reject_deferred(&[&operand])?;
            let value = if let Some(value) = operand.value.as_integer() {
                Value::Integer(-value)
            } else {
                match operand.value.as_number() {
                    Some(number) => Value::Number(-number),
                    // Suppressed (inactive) branch: `unary_minus` no-ops on
                    // skipped values (`PlaceholderParser.cpp:335-340`).
                    None if self.suppressed > 0 => Value::Bool(false),
                    None => return Err("numeric expression expected".to_owned()),
                }
            };
            return Ok(Operand::plain(value, start));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Operand, String> {
        let token = self.next().ok_or("expression ended unexpectedly")?;
        let start = self.offsets[self.index - 1];
        let value = match token {
            Token::Integer(value) => Value::Integer(value),
            Token::Number(value) => Value::Number(value),
            Token::Bool(value) => Value::Bool(value),
            Token::String(value) | Token::Regex(value) => Value::String(value),
            Token::Ident(name, offset) => {
                return if is_function_keyword(&name) && self.take(Token::Left) {
                    self.call(&name, start)
                } else {
                    self.variable(&name, offset)
                };
            }
            Token::Left => return self.parenthesized(start),
            _ => return Err("primary expression expected".to_owned()),
        };
        Ok(Operand::plain(value, start))
    }

    /// A parenthesized expression keeps its deferred state; a tuple is a plain list.
    fn parenthesized(&mut self, start: usize) -> Result<Operand, String> {
        let first = self.parse_or()?;
        if !self.take(Token::Comma) {
            if !self.take(Token::Right) {
                return Err("expected closing parenthesis".to_owned());
            }
            return Ok(Operand { start, ..first });
        }
        let mut values = vec![first.value];
        loop {
            values.push(self.parse_or()?.value);
            if self.take(Token::Right) {
                return Ok(Operand::plain(Value::List(values), start));
            }
            if !self.take(Token::Comma) {
                return Err("expected comma in tuple".to_owned());
            }
        }
    }

    /// `throw_if_deferred` (`PlaceholderParser.cpp:884-888`) on each operand in order;
    /// upstream skips it inside an inactive branch.
    fn reject_deferred(&mut self, operands: &[&Operand]) -> Result<(), String> {
        match deferred::first(operands) {
            Some(start) if self.suppressed == 0 => Err(self.deferred_failure(start)),
            _ => Ok(()),
        }
    }

    /// Reports the deferred message at the operand start for the line/caret report.
    fn deferred_failure(&mut self, start: usize) -> String {
        self.scalar_failure = Some((start, deferred::MESSAGE));
        deferred::MESSAGE.to_owned()
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
