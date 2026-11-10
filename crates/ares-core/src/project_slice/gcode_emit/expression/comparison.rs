use super::super::lexer::Token;
use super::{Value, to_string};

// PlaceholderParser.cpp:558-600: compare_op and its inverted wrappers.
pub(super) fn compare(left: &Value, right: &Value, token: Token) -> Result<bool, &'static str> {
    let (operation, invert) = match token {
        Token::Eq => (Operation::Equal, false),
        Token::NotEq => (Operation::Equal, true),
        Token::Lt => (Operation::Less, false),
        Token::Le => (Operation::Greater, true),
        Token::Gt => (Operation::Greater, false),
        Token::Ge => (Operation::Less, true),
        _ => unreachable!(),
    };
    let values = match (left.scalar(), right.scalar()) {
        (Value::Integer(left), Value::Integer(right)) => {
            (*left == *right, *left < *right, *left > *right)
        }
        (
            left @ (Value::Integer(_) | Value::Number(_)),
            right @ (Value::Integer(_) | Value::Number(_)),
        ) => {
            let left = left.as_number().unwrap();
            let right = right.as_number().unwrap();
            ((left - right).abs() < 1e-8, left < right, left > right)
        }
        (
            Value::Bool(left) | Value::OptionBool(left),
            Value::Bool(right) | Value::OptionBool(right),
        ) => {
            if !matches!(operation, Operation::Equal) {
                return Err("Cannot compare the types.");
            }
            (*left == *right, false, false)
        }
        (left @ Value::String(_), right) | (left, right @ Value::String(_)) => {
            let left = to_string(left);
            let right = to_string(right);
            (left == right, left < right, left > right)
        }
        _ => return Err("Cannot compare the types."),
    };
    let result = match operation {
        Operation::Equal => values.0,
        Operation::Less => values.1,
        Operation::Greater => values.2,
    };
    Ok(result != invert)
}

enum Operation {
    Equal,
    Less,
    Greater,
}
