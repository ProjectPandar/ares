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

/// `EPSILON` (`libslic3r.h:57`).
const EPSILON: f64 = 1e-4;

pub(super) enum InterpolateTableError {
    /// `expr_x.throw_exception` (`PlaceholderParser.cpp:2007`).
    OutsideRange,
    /// Table point `point` has a larger x than the next one (`:1982-1984`).
    NotIncreasing {
        point: usize,
    },
    Invalid(&'static str),
}

/// `InterpolateTableContext::evaluate` (`PlaceholderParser.cpp:1971-2009`).
pub(super) fn interpolate_table(args: &[Value]) -> Result<Value, InterpolateTableError> {
    use InterpolateTableError::{Invalid, NotIncreasing, OutsideRange};
    let x = args
        .first()
        .and_then(Value::as_number)
        .ok_or(Invalid("interpolate_table requires a numeric value"))?;
    let table = args[1..]
        .iter()
        .map(|point| {
            let mut values = point.iter_list();
            let x = values
                .next()
                .and_then(Value::as_number)
                .ok_or(Invalid("interpolate_table point requires x"))?;
            let y = values
                .next()
                .and_then(Value::as_number)
                .ok_or(Invalid("interpolate_table point requires y"))?;
            Ok((x, y))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (Some(&(first_x, first_y)), Some(&(last_x, last_y))) = (table.first(), table.last()) else {
        return Err(Invalid("interpolate_table requires points"));
    };
    let mut evaluated = None;
    for (point, &[(x0, y0), (x1, y1)]) in table.array_windows().enumerate() {
        if x0 > x1 {
            return Err(NotIncreasing { point });
        }
        if evaluated.is_none() && x >= x0 && x <= x1 {
            evaluated = Some(if x == x0 {
                y0
            } else if x == x1 {
                y1
            } else if (x0 - x1).abs() < EPSILON {
                0.5 * (y0 + y1)
            } else {
                // `lerp` (`libslic3r.h:313-317`).
                let t = (x - x0) / (x1 - x0);
                (1.0 - t) * y0 + t * y1
            });
        }
    }
    match evaluated {
        Some(y) => Ok(Value::number(y)),
        None if x > first_x - EPSILON && x < first_x => Ok(Value::number(first_y)),
        None if x > last_x && x < last_x + EPSILON => Ok(Value::number(last_y)),
        None => Err(OutsideRange),
    }
}
