//! `std::stod` prefixes used by the adaptive PA CSV and F readers.

use regex::Regex;
use std::sync::LazyLock;

pub(super) fn parse(input: &str) -> Option<f64> {
    static NUMBER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
            r"^[+-]?(?:0[xX](?:[0-9a-fA-F]+(?:\.[0-9a-fA-F]*)?|\.[0-9a-fA-F]+)(?:[pP][+-]?[0-9]+)?|",
            r"(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?)"
        ))
        .expect("stod number prefix")
    });
    let input = input.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let literal = NUMBER.find(input)?.as_str();
    let unsigned = literal.trim_start_matches(['+', '-']);
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        return hexadecimal(&unsigned[2..]).map(|value| {
            if literal.starts_with('-') {
                -value
            } else {
                value
            }
        });
    }
    let value = literal.parse::<f64>().ok()?;
    let mantissa = literal.split(['e', 'E']).next().expect("matched mantissa");
    let nonzero = mantissa.bytes().any(|c| matches!(c, b'1'..=b'9'));
    // `std::stod` raises out_of_range on decimal overflow/underflow.
    (value.is_finite() && !value.is_subnormal() && (value != 0.0 || !nonzero)).then_some(value)
}

fn hexadecimal(literal: &str) -> Option<f64> {
    let (mantissa, exponent) = literal.split_once(['p', 'P']).unwrap_or((literal, "0"));
    let exponent = exponent.parse::<i64>().unwrap_or_else(|_| {
        if exponent.starts_with('-') {
            i64::MIN
        } else {
            i64::MAX
        }
    });
    let fractional = mantissa.split_once('.').map_or(0, |(_, tail)| tail.len()) as i64;
    let mut significand = 0_u64;
    let mut kept = 0;
    let mut dropped = 0_i64;
    let mut sticky = false;
    for digit in mantissa.chars().filter(|&c| c != '.') {
        let digit = u64::from(digit.to_digit(16).expect("matched hexadecimal digit"));
        if significand == 0 && digit == 0 {
            continue;
        }
        if kept < 16 {
            significand = (significand << 4) | digit;
            kept += 1;
        } else {
            dropped += 1;
            sticky |= digit != 0;
        }
    }
    if significand == 0 {
        return Some(0.0);
    }
    // Retain guard bits and a sticky bit so conversion rounds once, ties-to-even.
    let bits = 64 - significand.leading_zeros();
    if bits > 53 {
        let discarded = bits - 53;
        let mask = (1_u64 << discarded) - 1;
        if sticky && significand & mask == 1_u64 << (discarded - 1) {
            significand += 1;
        }
    }
    let scale = i64::from(bits - 1);
    let exponent = exponent
        .saturating_sub(4 * fractional)
        .saturating_add(4 * dropped)
        .saturating_add(scale);
    let normalized = significand as f64 / 2_f64.powi(scale as i32);
    if !(-1075..=1023).contains(&exponent) {
        return None;
    }
    let value = if exponent < -1022 {
        (normalized * 2_f64.powi(-1022)) * 2_f64.powi((exponent + 1022) as i32)
    } else {
        normalized * 2_f64.powi(exponent as i32)
    };
    // Exact hexadecimal subnormals do not raise ERANGE; inexact tiny values do.
    let exact_tiny = !value.is_subnormal()
        || value * 2_f64.powi(1022) / 2_f64.powi((exponent + 1022) as i32) == normalized;
    (value.is_finite() && value != 0.0 && exact_tiny).then_some(value)
}
