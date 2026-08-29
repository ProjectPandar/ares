use super::{
    lexer::{Token, tokenize},
    value::{Config, Value},
};

pub(super) fn evaluate(input: &str, config: &Config) -> Result<Value, String> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        index: 0,
        config,
    };
    let value = parser.parse_or()?;
    if parser.index != parser.tokens.len() {
        return Err(format!("unexpected token in expression: {input}"));
    }
    Ok(value)
}

struct Parser<'a> {
    tokens: Vec<Token>,
    index: usize,
    config: &'a Config,
}

impl Parser<'_> {
    fn parse_or(&mut self) -> Result<Value, String> {
        let mut value = self.parse_and()?;
        while self.take(Token::Or) {
            let right = self.parse_and()?.as_bool();
            value = Value::Bool(value.as_bool() || right);
        }
        if self.take(Token::Question) {
            let when_true = self.parse_or()?;
            if !self.take(Token::Colon) {
                return Err("expected colon in conditional expression".to_owned());
            }
            let when_false = self.parse_or()?;
            value = if value.as_bool() {
                when_true
            } else {
                when_false
            };
        }
        Ok(value)
    }

    fn parse_and(&mut self) -> Result<Value, String> {
        let mut value = self.parse_compare()?;
        while self.take(Token::And) {
            let right = self.parse_compare()?.as_bool();
            value = Value::Bool(value.as_bool() && right);
        }
        Ok(value)
    }

    fn parse_compare(&mut self) -> Result<Value, String> {
        let left = self.parse_add()?;
        let comparison = match self.peek() {
            Some(Token::Eq) => Some(Token::Eq),
            Some(Token::NotEq) => Some(Token::NotEq),
            Some(Token::RegexMatch) => Some(Token::RegexMatch),
            Some(Token::RegexNotMatch) => Some(Token::RegexNotMatch),
            Some(Token::Lt) => Some(Token::Lt),
            Some(Token::Le) => Some(Token::Le),
            Some(Token::Gt) => Some(Token::Gt),
            Some(Token::Ge) => Some(Token::Ge),
            _ => None,
        };
        let Some(comparison) = comparison else {
            return Ok(left);
        };
        self.index += 1;
        let right = self.parse_add()?;
        let result = if matches!(comparison, Token::RegexMatch | Token::RegexNotMatch) {
            let matched = regex::Regex::new(&right.as_string())
                .map_err(|error| format!("invalid regex: {error}"))?
                .is_match(&left.as_string());
            if comparison == Token::RegexMatch {
                matched
            } else {
                !matched
            }
        } else if let (Some(left), Some(right)) = (left.as_number(), right.as_number()) {
            match comparison {
                // Upstream compares doubles with a 1e-8 tolerance
                // (`PlaceholderParser.cpp:5679-5681`).
                Token::Eq => (left - right).abs() < 1e-8,
                Token::NotEq => (left - right).abs() >= 1e-8,
                Token::Lt => left < right,
                Token::Le => left <= right,
                Token::Gt => left > right,
                Token::Ge => left >= right,
                _ => unreachable!(),
            }
        } else {
            let left = left.as_string();
            let right = right.as_string();
            match comparison {
                Token::Eq => left == right,
                Token::NotEq => left != right,
                Token::Lt => left < right,
                Token::Le => left <= right,
                Token::Gt => left > right,
                Token::Ge => left >= right,
                _ => unreachable!(),
            }
        };
        Ok(Value::Bool(result))
    }

    fn parse_add(&mut self) -> Result<Value, String> {
        let mut value = self.parse_mul()?;
        loop {
            let operation = if self.take(Token::Plus) {
                Arithmetic::Add
            } else if self.take(Token::Minus) {
                Arithmetic::Subtract
            } else {
                return Ok(value);
            };
            value = arithmetic(value, self.parse_mul()?, operation)?;
        }
    }

    fn parse_mul(&mut self) -> Result<Value, String> {
        let mut value = self.parse_unary()?;
        loop {
            let operation = if self.take(Token::Star) {
                Arithmetic::Multiply
            } else if self.take(Token::Slash) {
                Arithmetic::Divide
            } else if self.take(Token::Percent) {
                Arithmetic::Remainder
            } else {
                return Ok(value);
            };
            value = arithmetic(value, self.parse_unary()?, operation)?;
        }
    }

    fn parse_unary(&mut self) -> Result<Value, String> {
        if self.take(Token::Not) {
            return Ok(Value::Bool(!self.parse_unary()?.as_bool()));
        }
        if self.take(Token::Plus) {
            return self.parse_unary();
        }
        if self.take(Token::Minus) {
            let value = self.parse_unary()?;
            return if let Some(value) = value.as_integer() {
                Ok(Value::Integer(-value))
            } else {
                Ok(Value::Number(
                    -value.as_number().ok_or("numeric expression expected")?,
                ))
            };
        }
        self.parse_primary()
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "keeps expression grammar parsing ordered"
    )]
    fn parse_primary(&mut self) -> Result<Value, String> {
        let token = self.next().ok_or("expression ended unexpectedly")?;
        match token {
            Token::Integer(value) => Ok(Value::Integer(value)),
            Token::Number(value) => Ok(Value::Number(value)),
            Token::Bool(value) => Ok(Value::Bool(value)),
            Token::String(value) | Token::Regex(value) => Ok(Value::String(value)),
            Token::Ident(name) => {
                if self.take(Token::Left) {
                    let mut args = Vec::new();
                    if !self.take(Token::Right) {
                        loop {
                            args.push(self.parse_or()?);
                            if self.take(Token::Right) {
                                break;
                            }
                            if !self.take(Token::Comma) {
                                return Err("expected comma in function".to_owned());
                            }
                        }
                    }
                    return function(&name, args, self.config);
                }
                let mut value = self
                    .config
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| format!("unknown placeholder: {name}"))?;
                while self.take(Token::LeftBracket) {
                    let index = self
                        .parse_or()?
                        .as_number()
                        .ok_or("index must be numeric")? as usize;
                    if !self.take(Token::RightBracket) {
                        return Err("expected closing index bracket".to_owned());
                    }
                    value = value
                        .index(index)
                        .cloned()
                        .ok_or_else(|| format!("placeholder index out of range: {name}"))?;
                }
                Ok(value)
            }
            Token::Left => {
                let first = self.parse_or()?;
                if self.take(Token::Comma) {
                    let mut values = vec![first];
                    loop {
                        values.push(self.parse_or()?);
                        if self.take(Token::Right) {
                            break;
                        }
                        if !self.take(Token::Comma) {
                            return Err("expected comma in tuple".to_owned());
                        }
                    }
                    Ok(Value::List(values))
                } else {
                    if !self.take(Token::Right) {
                        return Err("expected closing parenthesis".to_owned());
                    }
                    Ok(first)
                }
            }
            _ => Err("primary expression expected".to_owned()),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned();
        self.index += usize::from(token.is_some());
        token
    }

    fn take(&mut self, expected: Token) -> bool {
        if self.peek() == Some(&expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy)]
enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

fn arithmetic(left: Value, right: Value, operation: Arithmetic) -> Result<Value, String> {
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

fn function(name: &str, args: Vec<Value>, config: &Config) -> Result<Value, String> {
    if name == "interpolate_table" {
        return interpolate_table(&args);
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
        "ceil" if numbers.len() == 1 => Ok(Value::Number(numbers[0].ceil())),
        "random" if numbers.len() == 2 && numbers[0] <= numbers[1] => {
            let unit = config.random_unit();
            let integer = numbers.iter().all(|number| number.fract() == 0.0);
            let result = if integer {
                numbers[0] + (unit * (numbers[1] - numbers[0] + 1.0)).floor()
            } else {
                numbers[0] + unit * (numbers[1] - numbers[0])
            };
            Ok(Value::Number(result))
        }
        "digits" if matches!(numbers.len(), 2 | 3) => {
            let width = numbers[1].clamp(0.0, 64.0) as usize;
            let rendered = if let Some(decimals) = numbers.get(2) {
                let decimals = decimals.clamp(0.0, 64.0) as usize;
                format!("{:.decimals$}", numbers[0])
            } else {
                format!("{:.0}", numbers[0])
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

#[cfg(test)]
mod tests;
