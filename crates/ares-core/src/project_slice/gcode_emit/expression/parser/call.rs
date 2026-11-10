//! Function keyword calls of `unary_expression` (`PlaceholderParser.cpp:2477-2499`).

use super::super::super::lexer::Token;
use super::super::super::value::Value;
use super::super::deferred::Operand;
use super::super::functions::{InterpolateTableError, function, interpolate_table};
use super::super::type_check;
use super::Parser;

impl Parser<'_> {
    /// Parses the arguments after `name(` and applies the function.
    pub(super) fn call(&mut self, name: &str, start: usize) -> Result<Operand, String> {
        // PlaceholderParser.cpp:1687,2333: size takes an unindexed
        // vector variable, not a scalar or an arbitrary expression.
        if name == "size" {
            let Some(Token::Ident(variable, _)) = self.next() else {
                return Err("parameter of size() is not a vector variable".to_owned());
            };
            if !self.take(Token::Right) {
                return Err("parameter of size() is not a vector variable".to_owned());
            }
            let value = match self.config.get(&variable) {
                Some(Value::List(values)) => Value::Integer(values.len() as i64),
                // Suppressed (inactive) branch: resolution no-ops
                // while skipping; a non-identifier argument or a
                // missing `)` stays a grammar failure.
                _ if self.suppressed > 0 => Value::Bool(false),
                _ => return Err("parameter of size() is not a vector variable".to_owned()),
            };
            return Ok(Operand::plain(value, start));
        }
        let (args, starts) = self.arguments()?;
        self.check(type_check::arguments(name, &args))?;
        let args = args.into_iter().map(|arg| arg.value).collect::<Vec<_>>();
        let value = if name == "interpolate_table" {
            self.interpolate_table(&args, &starts)?
        } else {
            match function(name, args, self.config) {
                Ok(value) => value,
                // Suppressed (inactive) branch: function actions no-op
                // on skipped values (`PlaceholderParser.cpp:617-668`).
                Err(_) if self.suppressed > 0 => Value::Bool(false),
                Err(error) => return Err(error),
            }
        };
        Ok(Operand::plain(value, start))
    }

    /// The comma-separated arguments up to `)`, with the first token index of each.
    fn arguments(&mut self) -> Result<(Vec<Operand>, Vec<usize>), String> {
        let mut args = Vec::new();
        let mut starts = Vec::new();
        if self.take(Token::Right) {
            return Ok((args, starts));
        }
        loop {
            starts.push(self.index);
            args.push(self.parse_or()?);
            if self.take(Token::Right) {
                return Ok((args, starts));
            }
            if !self.take(Token::Comma) {
                return Err("expected comma in function".to_owned());
            }
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
}
