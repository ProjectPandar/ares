use super::{Config, Value};

/// The `unary_expression` function keywords of PlaceholderParser.cpp:2315-2334.
/// Only these spellings parse as function calls; every other identifier
/// before `(` is a plain variable reference, so an unknown name like
/// `format` fails symbol resolution in `resolve_variable` (`:989-1002`,
/// "Not a variable name") with its line/caret instead of entering
/// function-argument evaluation.
const FUNCTION_KEYWORDS: [&str; 15] = [
    "min",
    "max",
    "random",
    "filament_change",
    "digits",
    "zdigits",
    "int",
    "round",
    "ceil",
    "floor",
    "is_nil",
    "one_of",
    "empty",
    "size",
    "interpolate_table",
];

pub(crate) fn is_function_keyword(name: &str) -> bool {
    FUNCTION_KEYWORDS.contains(&name)
}

pub(crate) fn function(name: &str, args: Vec<Value>, config: &Config) -> Result<Value, String> {
    if name == "interpolate_table" {
        return interpolate_table(&args);
    }
    if name == "random" {
        return config.random(&args);
    }
    let numbers = args
        .iter()
        .map(|value| {
            value
                .as_number()
                .ok_or("numeric function argument expected")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let integer_result = |operation: fn(f64) -> f64| {
        Value::Integer(
            args[0]
                .as_integer()
                .unwrap_or_else(|| i64::from(operation(numbers[0]) as i32)),
        )
    };
    match name {
        "min" | "max" => {
            let maximum = name == "max";
            let result = match args
                .iter()
                .map(Value::as_integer)
                .collect::<Option<Vec<_>>>()
            {
                Some(integers) => integers
                    .into_iter()
                    .reduce(if maximum { i64::max } else { i64::min })
                    .map(Value::Integer),
                None => numbers
                    .into_iter()
                    .reduce(if maximum { f64::max } else { f64::min })
                    .map(Value::Number),
            };
            result.ok_or_else(|| format!("{name} requires an argument"))
        }
        "int" if numbers.len() == 1 => Ok(integer_result(f64::trunc)),
        "round" if numbers.len() == 1 => Ok(integer_result(f64::round)),
        "floor" if numbers.len() == 1 => Ok(integer_result(f64::floor)),
        "ceil" if numbers.len() == 1 => Ok(integer_result(f64::ceil)),
        "digits" | "zdigits" if matches!(numbers.len(), 2 | 3) => {
            let width = numbers[1].clamp(0.0, 64.0) as usize;
            let rendered = if let Some(decimals) = numbers.get(2) {
                let decimals = decimals.clamp(0.0, 64.0) as usize;
                format!("{:.decimals$}", numbers[0])
            } else {
                let integer = args[0]
                    .as_integer()
                    .unwrap_or_else(|| i64::from(numbers[0].round() as i32));
                integer.to_string()
            };
            Ok(Value::String(pad_number(
                rendered,
                width,
                name == "zdigits",
            )))
        }
        _ => Err(format!("unknown function: {name}")),
    }
}

/// `expr::digits<leading_zeros>` (`PlaceholderParser.cpp:663-687`) pads
/// the formatted number to `width`: `%*d`/`%*.*lf` right-justify with
/// spaces, `%0*d`/`%0*.*lf` zero-pad after a leading sign.
fn pad_number(rendered: String, width: usize, leading_zeros: bool) -> String {
    if !leading_zeros {
        return format!("{rendered:>width$}");
    }
    if rendered.len() >= width {
        return rendered;
    }
    match rendered.as_bytes().first() {
        Some(b'-') => format!("-{}{}", "0".repeat(width - rendered.len()), &rendered[1..]),
        _ => format!("{}{}", "0".repeat(width - rendered.len()), rendered),
    }
}

fn interpolate_table(args: &[Value]) -> Result<Value, String> {
    let value = args
        .first()
        .and_then(Value::as_number)
        .ok_or("interpolate_table requires a numeric value")?;
    let points = args[1..]
        .iter()
        .map(|point| {
            let mut values = point.iter_list();
            let x = values
                .next()
                .and_then(Value::as_number)
                .ok_or("interpolate_table point requires x")?;
            let y = values
                .next()
                .and_then(Value::as_number)
                .ok_or("interpolate_table point requires y")?;
            Ok((x, y))
        })
        .collect::<Result<Vec<_>, &str>>()?;
    let Some(&(first_x, first_y)) = points.first() else {
        return Err("interpolate_table requires points".to_owned());
    };
    if value <= first_x {
        return Ok(Value::number(first_y));
    }
    for pair in points.windows(2) {
        let [(left_x, left_y), (right_x, right_y)] = pair else {
            unreachable!()
        };
        if value <= *right_x {
            let ratio = (value - left_x) / (right_x - left_x);
            return Ok(Value::number(left_y + ratio * (right_y - left_y)));
        }
    }
    Ok(Value::number(points.last().unwrap().1))
}
