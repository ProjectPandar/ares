//! Upstream template parse-error report (`PlaceholderParser.cpp:1726-1770`).

use super::super::expression::{ParseFailure, PathError};

/// Formats the error text stored per failed template: a header naming the
/// line number and the parse-path message, the physical line containing the
/// failing identifier, and a caret at the identifier's byte column.
/// `offset` is the identifier's byte offset within the whole template.
pub(super) fn report(template: &str, failure: &ParseFailure, offset: usize) -> String {
    let before = &template[..offset];
    let line_nr = 1 + before.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = before.rfind('\n').map_or(0, |position| position + 1);
    let line_end = template[offset..]
        .find('\n')
        .map_or(template.len(), |position| offset + position);
    let error_line = &template[line_start..line_end];
    format!(
        "Parsing error at line {line_nr}: {message}\n{error_line}\n{caret}^\n",
        message = failure.message,
        caret = " ".repeat(offset - line_start)
    )
}

/// The error text of a failure in an expression that starts at template byte `offset`.
pub(super) fn describe(template: &str, error: PathError, offset: usize) -> String {
    match error {
        PathError::Parse(failure) => report(template, &failure, offset + failure.offset),
        PathError::Other(message) => message,
    }
}
