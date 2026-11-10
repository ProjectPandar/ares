//! Deferred values (`PlaceholderParser.cpp:314-317`): a placeholder replaced only after the
//! G-code is generated can be printed, but not used in a calculation or a comparison
//! (`:862-889`).

use super::super::processor::DEFERRED_PLACEHOLDERS;
use super::super::value::Value;
use super::arithmetic::Arithmetic;
use super::type_check::is_string;

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

    pub(super) fn is_deferred(&self) -> bool {
        self.deferred != Deferred::None
    }
}

/// The operand `operator+=` (`:491-499`) rejects as deferred before any type check.
pub(super) fn addition_failure(left: &Operand, right: &Operand) -> Option<usize> {
    if left.deferred == Deferred::Value
        && (!is_string(&right.value) || right.deferred == Deferred::Value)
    {
        Some(left.start)
    } else if right.deferred == Deferred::Value && !is_string(&left.value) {
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

/// The first deferred operand of an operator that rejects any deferred value
/// (`throw_if_deferred`, `:884-888`).
pub(super) fn first(operands: &[&Operand]) -> Option<usize> {
    operands
        .iter()
        .find(|operand| operand.is_deferred())
        .map(|operand| operand.start)
}
