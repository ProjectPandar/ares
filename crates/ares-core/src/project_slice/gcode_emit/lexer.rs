#[derive(Clone, Debug, PartialEq)]
pub(super) enum Token {
    Integer(i64),
    Number(f64),
    Bool(bool),
    String(String),
    Regex(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    NotEq,
    RegexMatch,
    RegexNotMatch,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Left,
    Right,
    LeftBracket,
    RightBracket,
    Comma,
    Question,
    Colon,
}

pub(super) fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&character) = chars.peek() {
        if character.is_whitespace() {
            chars.next();
            continue;
        }
        let token = match character {
            '+' => {
                chars.next();
                Token::Plus
            }
            '-' => {
                chars.next();
                Token::Minus
            }
            '*' => {
                chars.next();
                Token::Star
            }
            '/' if matches!(
                tokens.last(),
                Some(Token::RegexMatch | Token::RegexNotMatch)
            ) =>
            {
                Token::Regex(read_regex(&mut chars)?)
            }
            '/' => {
                chars.next();
                Token::Slash
            }
            '%' => {
                chars.next();
                Token::Percent
            }
            '(' => {
                chars.next();
                Token::Left
            }
            ')' => {
                chars.next();
                Token::Right
            }
            '[' => {
                chars.next();
                Token::LeftBracket
            }
            ']' => {
                chars.next();
                Token::RightBracket
            }
            ',' => {
                chars.next();
                Token::Comma
            }
            '?' => {
                chars.next();
                Token::Question
            }
            ':' => {
                chars.next();
                Token::Colon
            }
            '=' => {
                chars.next();
                if chars.next_if_eq(&'~').is_some() {
                    Token::RegexMatch
                } else {
                    // Upstream treats `=`, `==` as equality
                    // (`PlaceholderParser.cpp:599`); consume a doubled sign.
                    chars.next_if_eq(&'=');
                    Token::Eq
                }
            }
            '!' => {
                chars.next();
                if chars.next_if_eq(&'=').is_some() {
                    Token::NotEq
                } else if chars.next_if_eq(&'~').is_some() {
                    Token::RegexNotMatch
                } else {
                    Token::Not
                }
            }
            '<' => {
                chars.next();
                if chars.next_if_eq(&'=').is_some() {
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                chars.next();
                if chars.next_if_eq(&'=').is_some() {
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '&' => {
                chars.next();
                expect_char(&mut chars, '&')?;
                Token::And
            }
            '|' => {
                chars.next();
                expect_char(&mut chars, '|')?;
                Token::Or
            }
            '"' => Token::String(read_string(&mut chars)?),
            character if character.is_ascii_digit() || character == '.' => read_number(&mut chars)?,
            character if character.is_ascii_alphabetic() || character == '_' => {
                match read_ident(&mut chars).as_str() {
                    "and" => Token::And,
                    "or" => Token::Or,
                    "not" => Token::Not,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    identifier => Token::Ident(identifier.to_owned()),
                }
            }
            _ => return Err(format!("invalid expression character: {character}")),
        };
        tokens.push(token);
    }
    Ok(tokens)
}

fn expect_char(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    expected: char,
) -> Result<(), String> {
    if chars.next_if_eq(&expected).is_some() {
        Ok(())
    } else {
        Err(format!("expected {expected}"))
    }
}

fn read_string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<String, String> {
    chars.next();
    let mut output = String::new();
    while let Some(character) = chars.next() {
        match character {
            '"' => return Ok(output),
            '\\' => output.push(chars.next().ok_or("unterminated string")?),
            character => output.push(character),
        }
    }
    Err("unterminated string".to_owned())
}

fn read_regex(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<String, String> {
    chars.next();
    let mut output = String::new();
    while let Some(character) = chars.next() {
        match character {
            '/' => return Ok(output),
            '\\' => {
                output.push(character);
                output.push(chars.next().ok_or("unterminated regex")?);
            }
            character => output.push(character),
        }
    }
    Err("unterminated regex".to_owned())
}

fn read_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<Token, String> {
    let mut value = String::new();
    while chars
        .peek()
        .is_some_and(|character| character.is_ascii_digit() || *character == '.')
    {
        value.push(chars.next().unwrap());
    }
    if value.contains('.') {
        value
            .parse()
            .map(Token::Number)
            .map_err(|_| format!("invalid number: {value}"))
    } else {
        value
            .parse()
            .map(Token::Integer)
            .map_err(|_| format!("invalid integer: {value}"))
    }
}

fn read_ident(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut value = String::new();
    while chars
        .peek()
        .is_some_and(|character| character.is_ascii_alphanumeric() || *character == '_')
    {
        value.push(chars.next().unwrap());
    }
    value
}
