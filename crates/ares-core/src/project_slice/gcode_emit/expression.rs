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
        let result = if let (Some(left), Some(right)) = (left.as_number(), right.as_number()) {
            match comparison {
                Token::Eq => left == right,
                Token::NotEq => left != right,
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
            let operator = if self.take(Token::Plus) {
                Some(1.0)
            } else if self.take(Token::Minus) {
                Some(-1.0)
            } else {
                None
            };
            let Some(operator) = operator else {
                return Ok(value);
            };
            let right = self
                .parse_mul()?
                .as_number()
                .ok_or("numeric expression expected in addition rhs")?;
            let left = value
                .as_number()
                .ok_or("numeric expression expected in addition lhs")?;
            value = Value::Number(if operator > 0.0 {
                left + right
            } else {
                left - right
            });
        }
    }

    fn parse_mul(&mut self) -> Result<Value, String> {
        let parsed = self.parse_unary()?;
        if !matches!(
            self.peek(),
            Some(Token::Star | Token::Slash | Token::Percent)
        ) {
            return Ok(parsed);
        }
        let mut value = parsed
            .as_number()
            .map(Value::Number)
            .ok_or("numeric expression expected in multiplication lhs")?;
        loop {
            let operation = if self.take(Token::Star) {
                0
            } else if self.take(Token::Slash) {
                1
            } else if self.take(Token::Percent) {
                2
            } else {
                return Ok(value);
            };
            let right = self
                .parse_unary()?
                .as_number()
                .ok_or("numeric expression expected in multiplication rhs")?;
            let left = value.as_number().unwrap();
            value = Value::Number(match operation {
                0 => left * right,
                1 => left / right,
                2 => left % right,
                _ => unreachable!(),
            });
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
            return Ok(Value::Number(
                -self
                    .parse_unary()?
                    .as_number()
                    .ok_or("numeric expression expected")?,
            ));
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
            Token::Number(value) => Ok(Value::Number(value)),
            Token::Bool(value) => Ok(Value::Bool(value)),
            Token::String(value) => Ok(Value::String(value)),
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
                    return function(&name, args);
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
                let value = self.parse_or()?;
                if !self.take(Token::Right) {
                    return Err("expected closing parenthesis".to_owned());
                }
                Ok(value)
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

fn function(name: &str, args: Vec<Value>) -> Result<Value, String> {
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
        _ => Err(format!("unknown function: {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_slice::gcode_emit::value::Config;

    fn config() -> Config {
        Config::from_block(b"; n = 2\n; values = 3,5\n; type = PLA\n")
    }

    #[test]
    fn expression_supports_indexing_arithmetic_and_functions() {
        assert_eq!(
            evaluate("values[1]", &config()).unwrap().as_number(),
            Some(5.0)
        );
        assert_eq!(
            evaluate("ceil(n / 2)", &config()).unwrap().as_number(),
            Some(1.0)
        );
        assert_eq!(
            evaluate("values[1] + ceil(n / 2)", &config())
                .unwrap()
                .as_number(),
            Some(6.0)
        );
        assert!(
            evaluate("type == \"PLA\" && max(n, 3) == 3", &config())
                .unwrap()
                .as_bool()
        );
    }

    #[test]
    fn expression_supports_word_operators_and_boolean_literals() {
        assert!(evaluate("true and not false", &config()).unwrap().as_bool());
        assert!(evaluate("false or n == 2", &config()).unwrap().as_bool());
        assert!(!evaluate("n != 2 or false", &config()).unwrap().as_bool());
    }

    #[test]
    fn expression_supports_modulo_unary_plus_and_conditionals() {
        assert_eq!(
            evaluate("+values[1] % 2", &config()).unwrap().as_number(),
            Some(1.0)
        );
        assert_eq!(
            evaluate("n > 1 ? values[0] : values[1]", &config())
                .unwrap()
                .as_number(),
            Some(3.0)
        );
    }
}
