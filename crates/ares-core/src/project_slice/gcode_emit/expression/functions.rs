use super::{Config, Value};

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
    match name {
        "min" => numbers
            .into_iter()
            .reduce(f64::min)
            .map(Value::Number)
            .ok_or("min requires an argument".to_owned()),
        "max" => numbers
            .into_iter()
            .reduce(f64::max)
            .map(Value::Number)
            .ok_or("max requires an argument".to_owned()),
        "int" if numbers.len() == 1 => Ok(Value::Integer(
            args[0]
                .as_integer()
                .unwrap_or_else(|| i64::from(numbers[0] as i32)),
        )),
        "round" if numbers.len() == 1 => Ok(Value::Number(numbers[0].round())),
        "floor" if numbers.len() == 1 => Ok(Value::Number(numbers[0].floor())),
        "ceil" if numbers.len() == 1 => Ok(Value::Number(numbers[0].ceil())),
        "digits" if matches!(numbers.len(), 2 | 3) => {
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
            Ok(Value::String(format!("{rendered:>width$}")))
        }
        _ => Err(format!("unknown function: {name}")),
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
