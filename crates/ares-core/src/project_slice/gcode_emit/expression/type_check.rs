//! Operator and function type errors of `expr` (`PlaceholderParser.cpp:391-748`, `:845-858`):
//! only `TYPE_INT` and `TYPE_DOUBLE` are numeric (`:313`), and nothing converts between types.

use super::super::value::Value;
use super::arithmetic::{Arithmetic, to_string};
use super::deferred::{self, Operand};

/// The operand start the error caret points at, and the message.
pub(super) type Failure = (usize, &'static str);

pub(super) fn is_string(value: &Value) -> bool {
    matches!(value.scalar(), Value::String(_))
}

fn is_numeric(value: &Value) -> bool {
    matches!(value.scalar(), Value::Integer(_) | Value::Number(_))
}

fn is_bool(value: &Value) -> bool {
    matches!(value.scalar(), Value::Bool(_) | Value::OptionBool(_))
}

/// `expr::throw_exception` (`:869-875`): a deferred operand reports the deferred message.
fn fail(operand: &Operand, message: &'static str) -> Failure {
    let message = if operand.is_deferred() {
        deferred::MESSAGE
    } else {
        message
    };
    (operand.start, message)
}

fn numeric(operand: &Operand, message: &'static str) -> Result<(), Failure> {
    if is_numeric(&operand.value) {
        Ok(())
    } else {
        Err(fail(operand, message))
    }
}

fn integer(operand: &Operand, message: &'static str) -> Result<(), Failure> {
    if matches!(operand.value.scalar(), Value::Integer(_)) {
        Ok(())
    } else {
        Err(fail(operand, message))
    }
}

/// `operator+=` (`:510-517`) once neither operand is a string, `-=`, `*=` (`:523-557`), and
/// `/=`, `%=` with their zero divisor (`:559-595`); the left operand first.
pub(super) fn arithmetic(
    left: &Operand,
    right: &Operand,
    operation: Arithmetic,
) -> Result<(), Failure> {
    let (left_message, right_message) = match operation {
        Arithmetic::Add if is_string(&left.value) || is_string(&right.value) => return Ok(()),
        Arithmetic::Add => (
            "Cannot add non-numeric types.",
            "Cannot add non-numeric types.",
        ),
        Arithmetic::Subtract => (
            "Cannot subtract non-numeric types.",
            "Cannot subtract non-numeric types.",
        ),
        Arithmetic::Multiply => (
            "Cannot multiply with non-numeric type.",
            "Cannot multiply with non-numeric type.",
        ),
        Arithmetic::Divide | Arithmetic::Remainder => (
            "Cannot divide a non-numeric type.",
            "Cannot divide with a non-numeric type.",
        ),
    };
    numeric(left, left_message)?;
    numeric(right, right_message)?;
    if matches!(operation, Arithmetic::Divide | Arithmetic::Remainder)
        && right.value.as_number() == Some(0.0)
    {
        return Err((right.start, "Division by zero"));
    }
    Ok(())
}

/// `unary_minus` (`:391-406`).
pub(super) fn negation(operand: &Operand) -> Result<(), Failure> {
    numeric(operand, "Cannot apply unary minus operator.")
}

/// `unary_not` (`:475-488`).
pub(super) fn inversion(operand: &Operand) -> Result<(), Failure> {
    if is_bool(&operand.value) {
        Ok(())
    } else {
        Err(fail(operand, "Cannot apply a not operator."))
    }
}

/// `evaluate_boolean` (`:604-612`) of a ternary condition.
pub(super) fn condition(operand: &Operand) -> Result<(), Failure> {
    if is_bool(&operand.value) {
        Ok(())
    } else {
        Err(fail(operand, "Not a boolean expression"))
    }
}

/// `logical_op` (`:853-857`), reported from the left operand.
pub(super) fn logical(left: &Operand, right: &Operand) -> Result<(), Failure> {
    if is_bool(&left.value) && is_bool(&right.value) {
        Ok(())
    } else {
        Err((
            left.start,
            "Cannot apply logical operation to non-boolean operands.",
        ))
    }
}

/// The parameter checks of the function keywords, in order: `int`, `round`, `floor`, `ceil`
/// (`:409-472`), `min`, `max` (`:684-690`), `random` (`:714-720`), `digits`, `zdigits`
/// (`:730-740`) and the `interpolate_table` lookup value (`:1959`).
pub(super) fn arguments(name: &str, arguments: &[Operand]) -> Result<(), Failure> {
    let Some(first) = arguments.first() else {
        return Ok(());
    };
    match name {
        "int" => numeric(first, "Cannot convert to integer."),
        "round" => numeric(first, "Cannot round a non-numeric value."),
        "floor" => numeric(first, "Cannot floor a non-numeric value."),
        "ceil" => numeric(first, "Cannot ceil a non-numeric value."),
        "min" | "max" | "random" => arguments
            .iter()
            .try_for_each(|argument| numeric(argument, "Not a numeric type.")),
        "digits" | "zdigits" => {
            numeric(first, "Not a numeric type.")?;
            let messages = [
                "digits: second parameter must be integer",
                "digits: third parameter must be integer",
            ];
            arguments[1..]
                .iter()
                .zip(messages)
                .try_for_each(|(argument, message)| integer(argument, message))
        }
        "interpolate_table" => numeric(first, "Interpolation value must be a number."),
        _ => Ok(()),
    }
}

/// `scalar_variable_assign_scalar` (`:1396-1426`): an assigned value takes the type of the
/// variable. `check_numeric` (`:1382-1385`) reports through `expr::throw_exception`; the boolean
/// check reports through `ctx->throw_exception`, without the deferred message.
pub(super) fn assigned(variable: &Value, operand: &Operand) -> Result<Value, Failure> {
    const NOT_NUMERIC: &str = "Right side is not a numeric expression";
    let value = operand.value.scalar();
    match variable {
        // `as_i` (`:327`) truncates a double.
        Value::Integer(_) => match value {
            Value::Integer(integer) => Ok(Value::Integer(*integer)),
            Value::Number(number) => Ok(Value::Integer(*number as i64)),
            _ => Err(fail(operand, NOT_NUMERIC)),
        },
        Value::Number(_) => match value {
            Value::Integer(integer) => Ok(Value::Number(*integer as f64)),
            Value::Number(number) => Ok(Value::Number(*number)),
            _ => Err(fail(operand, NOT_NUMERIC)),
        },
        Value::String(_) => Ok(Value::String(to_string(&operand.value))),
        Value::Bool(_) | Value::OptionBool(_) if is_bool(value) => Ok(Value::Bool(value.as_bool())),
        Value::Bool(_) | Value::OptionBool(_) => {
            Err((operand.start, "Right side is not a boolean expression"))
        }
        Value::Nil | Value::List(_) => Ok(operand.value.clone()),
    }
}

/// An element of `vector_variable_assign_initializer_list` (`:1651-1694`) or
/// `vector_variable_assign_expr_with_count` (`:1465-1490`): it takes the type of the vector like
/// `assigned`, but a non-boolean reports through `expr::throw_exception`.
pub(super) fn element(variable: &Value, operand: &Operand) -> Result<Value, Failure> {
    if matches!(variable, Value::Bool(_)) && !is_bool(&operand.value) {
        return Err(fail(operand, "Right side is not a boolean expression"));
    }
    assigned(variable, operand)
}

/// `evaluate_count` (`:1387-1394`): the number of elements `repeat` fills a vector with.
pub(super) fn count(operand: &Operand) -> Result<usize, Failure> {
    match operand.value.scalar() {
        Value::Integer(count) => usize::try_from(*count)
            .map_err(|_| fail(operand, "Negative number of elements specified.")),
        _ => Err(fail(
            operand,
            "Expected number of elements to fill a vector with.",
        )),
    }
}
