#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    Integer(i64),
    Number(f64),
    Bool(bool),
    String(String),
    Regex(String),
    /// Identifier with its byte offset in the expression input (feeds the
    /// upstream caret report, `PlaceholderParser.cpp:1726-1770`).
    Ident(String, usize),
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

/// Tokens with the byte offset of each token in `input`.
pub(crate) fn tokenize(input: &str) -> Result<(Vec<Token>, Vec<usize>), String> {
    let mut tokens = Vec::new();
    let mut offsets = Vec::new();
    let mut rest = input;
    while let Some(character) = rest.chars().next() {
        if character.is_whitespace() {
            rest = &rest[character.len_utf8()..];
            continue;
        }
        // Byte offset of the token being scanned within `input`.
        let offset = input.len() - rest.len();
        let token = match character {
            '+' => {
                rest = &rest[1..];
                Token::Plus
            }
            '-' => {
                rest = &rest[1..];
                Token::Minus
            }
            '*' => {
                rest = &rest[1..];
                Token::Star
            }
            '/' if matches!(
                tokens.last(),
                Some(Token::RegexMatch | Token::RegexNotMatch)
            ) =>
            {
                Token::Regex(read_regex(&mut rest)?)
            }
            '/' => {
                rest = &rest[1..];
                Token::Slash
            }
            '%' => {
                rest = &rest[1..];
                Token::Percent
            }
            '(' => {
                rest = &rest[1..];
                Token::Left
            }
            ')' => {
                rest = &rest[1..];
                Token::Right
            }
            '[' => {
                rest = &rest[1..];
                Token::LeftBracket
            }
            ']' => {
                rest = &rest[1..];
                Token::RightBracket
            }
            ',' => {
                rest = &rest[1..];
                Token::Comma
            }
            '?' => {
                rest = &rest[1..];
                Token::Question
            }
            ':' => {
                rest = &rest[1..];
                Token::Colon
            }
            '=' => {
                rest = &rest[1..];
                if rest.starts_with('~') {
                    rest = &rest[1..];
                    Token::RegexMatch
                } else {
                    // Upstream treats `=`, `==` as equality
                    // (`PlaceholderParser.cpp:599`); consume a doubled sign.
                    rest = rest.strip_prefix('=').unwrap_or(rest);
                    Token::Eq
                }
            }
            '!' => {
                rest = &rest[1..];
                if let Some(tail) = rest.strip_prefix('=') {
                    rest = tail;
                    Token::NotEq
                } else if let Some(tail) = rest.strip_prefix('~') {
                    rest = tail;
                    Token::RegexNotMatch
                } else {
                    Token::Not
                }
            }
            '<' => {
                rest = &rest[1..];
                if let Some(tail) = rest.strip_prefix('=') {
                    rest = tail;
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                rest = &rest[1..];
                if let Some(tail) = rest.strip_prefix('=') {
                    rest = tail;
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '&' => {
                rest = &rest[1..];
                expect_char(&mut rest, '&')?;
                Token::And
            }
            '|' => {
                rest = &rest[1..];
                expect_char(&mut rest, '|')?;
                Token::Or
            }
            '"' => Token::String(read_string(&mut rest)?),
            character if character.is_ascii_digit() || character == '.' => read_number(&mut rest)?,
            character if character.is_ascii_alphabetic() || character == '_' => {
                match read_ident(rest).as_str() {
                    "and" => {
                        rest = &rest[3..];
                        Token::And
                    }
                    "or" => {
                        rest = &rest[2..];
                        Token::Or
                    }
                    "not" => {
                        rest = &rest[3..];
                        Token::Not
                    }
                    "true" => {
                        rest = &rest[4..];
                        Token::Bool(true)
                    }
                    "false" => {
                        rest = &rest[5..];
                        Token::Bool(false)
                    }
                    identifier => {
                        rest = &rest[identifier.len()..];
                        Token::Ident(identifier.to_owned(), offset)
                    }
                }
            }
            _ => return Err(format!("invalid expression character: {character}")),
        };
        tokens.push(token);
        offsets.push(offset);
    }
    Ok((tokens, offsets))
}

fn expect_char(rest: &mut &str, expected: char) -> Result<(), String> {
    if let Some(tail) = rest.strip_prefix(expected) {
        *rest = tail;
        Ok(())
    } else {
        Err(format!("expected {expected}"))
    }
}

fn read_string(rest: &mut &str) -> Result<String, String> {
    *rest = &rest[1..];
    let mut output = String::new();
    let mut chars = rest.chars();
    while let Some(character) = chars.next() {
        match character {
            '"' => {
                *rest = chars.as_str();
                return Ok(output);
            }
            '\\' => {
                let escaped = chars.next().ok_or("unterminated string")?;
                output.push(escaped);
            }
            character => output.push(character),
        }
    }
    Err("unterminated string".to_owned())
}

fn read_regex(rest: &mut &str) -> Result<String, String> {
    *rest = &rest[1..];
    let mut output = String::new();
    let mut chars = rest.chars();
    while let Some(character) = chars.next() {
        match character {
            '/' => {
                *rest = chars.as_str();
                return Ok(output);
            }
            '\\' => {
                output.push(character);
                let escaped = chars.next().ok_or("unterminated regex")?;
                output.push(escaped);
            }
            character => output.push(character),
        }
    }
    Err("unterminated regex".to_owned())
}

fn read_number(rest: &mut &str) -> Result<Token, String> {
    let digits: String = rest
        .chars()
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .collect();
    *rest = &rest[digits.len()..];
    if digits.contains('.') {
        digits
            .parse()
            .map(Token::Number)
            .map_err(|_| format!("invalid number: {digits}"))
    } else {
        digits
            .parse()
            .map(Token::Integer)
            .map_err(|_| format!("invalid integer: {digits}"))
    }
}

fn read_ident(rest: &str) -> String {
    rest.chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect()
}
