//! Deferred values (`PlaceholderParser.cpp:314-317`): a placeholder replaced only after the
//! G-code is generated can be printed, but not used in a calculation or a comparison
//! (`:862-889`).

use super::super::processor::DEFERRED_PLACEHOLDERS;
use super::super::value::Value;
use super::arithmetic::Arithmetic;

pub(super) const MESSAGE: &str = "This value is known only after the G-code is generated. It can \
     be printed, but not used in a calculation or a comparison.";

/// `expr::Deferred`: one deferred value, or text that contains at least one.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Deferred {
    None,
    Value,
    Text,
}

impl Deferred {
    /// `MyContext::mark_deferred` (`:1024-1034`), matched by value so that a variable that
    /// copied a deferred value is deferred too.
    pub(super) fn of(value: &Value) -> Self {
        let Value::String(text) = value else {
            return Self::None;
        };
        if !text.contains('@') {
            return Self::None;
        }
        match DEFERRED_PLACEHOLDERS
            .iter()
            .find(|(_, tag)| text.contains(tag))
        {
            Some((_, tag)) if text == tag => Self::Value,
            Some(_) => Self::Text,
            None => Self::None,
        }
    }
}

/// An evaluated expression with its deferred state and the byte offset where it starts
/// (`expr::it_range`), which the error caret points at.
pub(super) struct Operand {
    pub(super) value: Value,
    pub(super) deferred: Deferred,
    pub(super) start: usize,
}

impl Operand {
    pub(super) fn plain(value: Value, start: usize) -> Self {
        Self {
            value,
            deferred: Deferred::None,
            start,
        }
    }

    fn is_deferred(&self) -> bool {
        self.deferred != Deferred::None
    }
}

fn is_string(value: &Value) -> bool {
    matches!(value.scalar(), Value::String(_))
}

fn is_numeric(value: &Value) -> bool {
    matches!(value.scalar(), Value::Integer(_) | Value::Number(_))
}

/// The operand an arithmetic operator rejects as deferred: `operator+=` (`:491-512`) and the
/// `throw_if_not_numeric` order of `-`, `*`, `/` and `%` (`:523-596`).
pub(super) fn arithmetic_failure(
    left: &Operand,
    right: &Operand,
    operation: Arithmetic,
) -> Option<usize> {
    if matches!(operation, Arithmetic::Add) {
        return if left.deferred == Deferred::Value
            && (!is_string(&right.value) || right.deferred == Deferred::Value)
        {
            Some(left.start)
        } else if right.deferred == Deferred::Value && !is_string(&left.value) {
            Some(right.start)
        } else {
            None
        };
    }
    if left.is_deferred() {
        Some(left.start)
    } else if right.is_deferred() && is_numeric(&left.value) {
        Some(right.start)
    } else {
        None
    }
}

/// A concatenation with a deferred operand is deferred text (`:500-509`).
pub(super) fn sum(left: &Operand, right: &Operand, operation: Arithmetic) -> Deferred {
    if matches!(operation, Arithmetic::Add) && (left.is_deferred() || right.is_deferred()) {
        Deferred::Text
    } else {
        Deferred::None
    }
}

/// The first deferred parameter a function rejects, checked in order until a parameter fails
/// its own type check: `min`, `max` (`:684`), `random` (`:714`), `digits` (`:730`), `int`,
/// `round`, `floor`, `ceil` (`:409-472`) and the `interpolate_table` lookup value (`:1959`).
pub(super) fn argument_failure(name: &str, arguments: &[Operand]) -> Option<usize> {
    let checked = match name {
        "min" | "max" | "random" | "digits" | "zdigits" | "int" | "round" | "floor" | "ceil" => {
            arguments
        }
        "interpolate_table" => &arguments[..arguments.len().min(1)],
        _ => return None,
    };
    for (index, argument) in checked.iter().enumerate() {
        if argument.is_deferred() {
            return Some(argument.start);
        }
        let accepted = if index > 0 && name.ends_with("digits") {
            matches!(argument.value.scalar(), Value::Integer(_))
        } else {
            is_numeric(&argument.value)
        };
        if !accepted {
            return None;
        }
    }
    None
}

/// The first deferred operand of an operator that rejects any deferred value
/// (`throw_if_deferred`, `:884-888`).
pub(super) fn first(operands: &[&Operand]) -> Option<usize> {
    operands
        .iter()
        .find(|operand| operand.is_deferred())
        .map(|operand| operand.start)
}
