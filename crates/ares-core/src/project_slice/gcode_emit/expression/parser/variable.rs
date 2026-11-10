//! Variable reads of `unary_expression` (`variable_value`, `PlaceholderParser.cpp:1492-1502`).

use super::super::super::lexer::Token;
use super::super::super::value::Value;
use super::super::ParsePath;
use super::super::deferred::{Deferred, Operand};
use super::Parser;

impl Parser<'_> {
    /// Resolves the variable `name` at byte `offset`, with its optional indices, and marks a
    /// value that holds a deferred placeholder (`mark_deferred`, `:1499`).
    pub(super) fn variable(&mut self, name: &str, offset: usize) -> Result<Operand, String> {
        let mut value = match self.config.get(name).cloned() {
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
                return Err("Referencing a vector variable when scalar is expected".to_owned());
            }
        }
        while self.take(Token::LeftBracket) {
            value = self.indexed(name, value)?;
        }
        if self.suppressed > 0 {
            return Ok(Operand::plain(value, offset));
        }
        value.ensure_defined()?;
        Ok(Operand {
            deferred: Deferred::of(&value),
            value,
            start: offset,
        })
    }

    fn indexed(&mut self, name: &str, value: Value) -> Result<Value, String> {
        let index = match self.parse_or()?.value.as_number() {
            Some(number) => Some(number as usize),
            // Suppressed (inactive) branch: `evaluate_index`
            // no-ops while skipping (`:1699-1703`).
            None if self.suppressed > 0 => None,
            None => return Err("index must be numeric".to_owned()),
        };
        if !self.take(Token::RightBracket) {
            return Err("expected closing index bracket".to_owned());
        }
        Ok(match index {
            Some(index) => match value.index(index) {
                Some(indexed) => indexed.clone(),
                // Suppressed (inactive) branch: indexing failures
                // no-op while skipping.
                None if self.suppressed > 0 => Value::Bool(false),
                None => return Err(format!("placeholder index out of range: {name}")),
            },
            None => Value::Bool(false),
        })
    }
}
