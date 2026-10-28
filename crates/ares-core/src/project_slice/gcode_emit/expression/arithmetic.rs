use super::super::value::Value;

#[derive(Clone, Copy)]
pub(super) enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

pub(super) fn arithmetic(
    left: Value,
    right: Value,
    operation: Arithmetic,
) -> Result<Value, String> {
    // PlaceholderParser.cpp:435-456: either string operand selects concatenation.
    if matches!(operation, Arithmetic::Add)
        && (matches!(left.scalar(), Value::String(_)) || matches!(right.scalar(), Value::String(_)))
    {
        let mut result = to_string(left.scalar());
        result.push_str(&to_string(right.scalar()));
        return Ok(Value::String(result));
    }
    if let (Some(left), Some(right)) = (left.as_integer(), right.as_integer()) {
        if matches!(operation, Arithmetic::Divide | Arithmetic::Remainder) && right == 0 {
            return Err("division by zero".to_owned());
        }
        return Ok(Value::Integer(match operation {
            Arithmetic::Add => left + right,
            Arithmetic::Subtract => left - right,
            Arithmetic::Multiply => left * right,
            Arithmetic::Divide => left / right,
            Arithmetic::Remainder => left % right,
        }));
    }
    let left = left
        .as_number()
        .ok_or("numeric expression expected in lhs")?;
    let right = right
        .as_number()
        .ok_or("numeric expression expected in rhs")?;
    if matches!(operation, Arithmetic::Divide | Arithmetic::Remainder) && right == 0.0 {
        return Err("division by zero".to_owned());
    }
    Ok(Value::Number(match operation {
        Arithmetic::Add => left + right,
        Arithmetic::Subtract => left - right,
        Arithmetic::Multiply => left * right,
        Arithmetic::Divide => left / right,
        Arithmetic::Remainder => left % right,
    }))
}

pub(in crate::project_slice::gcode_emit) fn to_string(value: &Value) -> String {
    // PlaceholderParser.cpp:302-327: doubles use default ostream precision.
    match value {
        Value::Number(number) if *number == 0.0 && number.is_sign_negative() => "-0".to_owned(),
        Value::Number(number) => super::super::motion::format::ostream(*number),
        value => value.as_string(),
    }
}
