//! Decimal `std::stod` prefixes used by the adaptive PA CSV and F readers.

use regex::Regex;
use std::sync::LazyLock;

pub(super) fn parse(input: &str) -> Option<f64> {
    static NUMBER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?")
            .expect("decimal number prefix")
    });
    let input = input.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let literal = NUMBER.find(input)?.as_str();
    let value = literal.parse::<f64>().ok()?;
    let mantissa = literal.split(['e', 'E']).next().expect("matched mantissa");
    let nonzero = mantissa.bytes().any(|c| matches!(c, b'1'..=b'9'));
    // `std::stod` raises out_of_range on decimal overflow/underflow.
    (value.is_finite() && !value.is_subnormal() && (value != 0.0 || !nonzero)).then_some(value)
}
