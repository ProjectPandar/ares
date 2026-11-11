//! The right-hand side alternatives of `assignment_statement` and `new_variable_statement`
//! (`PlaceholderParser.cpp:2435-2470`): an initializer list, a naked vector reference, a
//! conditional expression and `repeat(count, value)`.

use super::find_top_level;

/// The `keywords` an `identifier` cannot be (`:2373-2375, 2553-2579`).
const KEYWORDS: [&str; 28] = [
    "and",
    "digits",
    "zdigits",
    "empty",
    "if",
    "int",
    "is_nil",
    "local",
    "else",
    "elsif",
    "endif",
    "false",
    "global",
    "interpolate_table",
    "min",
    "max",
    "random",
    "regex_replace",
    "filament_change",
    "repeat",
    "round",
    "floor",
    "ceil",
    "not",
    "one_of",
    "or",
    "size",
    "true",
];

/// A right-hand side, as slices of the block body.
pub(super) enum Rhs<'a> {
    /// `initializer_list` (`:2464-2470`): the element expressions of `(...)`, which also takes
    /// a single parenthesized expression.
    List(Vec<&'a str>),
    /// `repeat(count, value)`.
    Repeat(&'a str, &'a str),
    Expression,
}

impl<'a> Rhs<'a> {
    pub(super) fn parse(rhs: &'a str) -> Self {
        if let Some(inner) = enclosed(rhs) {
            return Self::List(elements(inner));
        }
        if let Some(tail) = rhs.strip_prefix("repeat")
            && let Some(inner) = enclosed(tail.trim_start())
            && let [count, value] = elements(inner)[..]
        {
            return Self::Repeat(count, value);
        }
        Self::Expression
    }
}

/// A `variable_reference` (`:2538-2544`) a right-hand side starts with: its name, whether it
/// is indexed, and the text after it.
pub(super) struct Reference<'a> {
    pub(super) name: &'a str,
    pub(super) indexed: bool,
    pub(super) rest: &'a str,
}

impl<'a> Reference<'a> {
    pub(super) fn parse(rhs: &'a str) -> Option<Self> {
        let end = rhs
            .find(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .unwrap_or(rhs.len());
        let name = &rhs[..end];
        if name.is_empty()
            || name.starts_with(|character: char| character.is_ascii_digit())
            || KEYWORDS.contains(&name)
        {
            return None;
        }
        let rest = rhs[end..].trim_start();
        if !rest.starts_with('[') {
            return Some(Self {
                name,
                indexed: false,
                rest,
            });
        }
        let close = closing(rest)?;
        Some(Self {
            name,
            indexed: true,
            rest: rest[close + 1..].trim_start(),
        })
    }
}

/// The inside of `input` when it is one bracketed group `(...)`.
fn enclosed(input: &str) -> Option<&str> {
    (input.starts_with('(') && closing(input)? == input.len() - 1)
        .then(|| &input[1..input.len() - 1])
}

/// The top-level `,`-separated expressions of a list, trimmed; none in `()`.
fn elements(inner: &str) -> Vec<&str> {
    let mut elements = Vec::new();
    let mut rest = inner;
    if rest.trim().is_empty() {
        return elements;
    }
    while let Some(comma) = find_top_level(rest, ",") {
        elements.push(rest[..comma].trim());
        rest = &rest[comma + 1..];
    }
    elements.push(rest.trim());
    elements
}

/// The byte offset of the bracket that closes the one `input` starts with, outside strings.
fn closing(input: &str) -> Option<usize> {
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, character) in input.bytes().enumerate() {
        match character {
            _ if escaped => escaped = false,
            b'\\' if in_string => escaped = true,
            b'"' => in_string = !in_string,
            _ if in_string => {}
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }
    None
}
