//! C-locale atoi/atof consumed by bbs_3mf.cpp:2855,2878.

pub(in crate::project) fn atoi(text: &str) -> i32 {
    let text = text.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let bytes = text.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    if end == start {
        return 0;
    }
    text[..end].parse().unwrap_or(0)
}

pub(in crate::project) fn atof(text: &str) -> f64 {
    let text = text.trim_start_matches(|c: char| c.is_ascii_whitespace());
    if let Ok(value) = text.parse::<f64>() {
        return value;
    }
    let sign = usize::from(text.starts_with(['+', '-']));
    let unsigned = &text[sign..];
    if unsigned
        .get(..3)
        .is_some_and(|word| word.eq_ignore_ascii_case("inf"))
    {
        return if text.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    if unsigned
        .get(..3)
        .is_some_and(|word| word.eq_ignore_ascii_case("nan"))
    {
        return if text.starts_with('-') {
            -f64::NAN
        } else {
            f64::NAN
        };
    }
    let is_hex = unsigned.starts_with("0x") || unsigned.starts_with("0X");
    if is_hex {
        let end = mantissa_end(text.as_bytes(), sign + 2, true);
        let exponent = exponent_end(text.as_bytes(), end, b'p', b'P');
        if let Some(value) = hex(&text[..exponent]) {
            return value;
        }
    }
    let end = mantissa_end(text.as_bytes(), sign, false);
    let end = exponent_end(text.as_bytes(), end, b'e', b'E');
    text[..end].parse().unwrap_or(0.0)
}

fn mantissa_end(bytes: &[u8], start: usize, hex: bool) -> usize {
    let digit = |byte: &u8| {
        if hex {
            byte.is_ascii_hexdigit()
        } else {
            byte.is_ascii_digit()
        }
    };
    let mut end = start;
    while bytes.get(end).is_some_and(digit) {
        end += 1;
    }
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        while bytes.get(end).is_some_and(digit) {
            end += 1;
        }
    }
    end
}

fn exponent_end(bytes: &[u8], end: usize, lower: u8, upper: u8) -> usize {
    if !bytes
        .get(end)
        .is_some_and(|byte| *byte == lower || *byte == upper)
    {
        return end;
    }
    let mut cursor = end + 1;
    if matches!(bytes.get(cursor), Some(b'+' | b'-')) {
        cursor += 1;
    }
    let start = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == start { end } else { cursor }
}

fn hex(word: &str) -> Option<f64> {
    let negative = word.starts_with('-');
    let unsigned = word.strip_prefix(['+', '-']).unwrap_or(word);
    let hex = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))?;
    let (mantissa, exponent) = hex
        .split_once(['p', 'P'])
        .map_or((hex, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    if mantissa.matches('.').count() > 1
        || !mantissa.bytes().any(|byte| byte.is_ascii_hexdigit())
        || !mantissa
            .bytes()
            .all(|byte| byte == b'.' || byte.is_ascii_hexdigit())
    {
        return None;
    }
    let exponent = match exponent {
        None => 0,
        Some(exponent) => {
            let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            exponent
                .parse::<i64>()
                .unwrap_or(if exponent.starts_with('-') {
                    i64::MIN
                } else {
                    i64::MAX
                })
        }
    };
    let fractional = mantissa
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let mut bits = mantissa
        .bytes()
        .filter(|byte| *byte != b'.')
        .flat_map(|byte| {
            let digit = u32::from((byte & 15) + 9 * u8::from(byte > b'9'));
            (0..4).rev().map(move |shift| digit & (1 << shift) != 0)
        })
        .skip_while(|bit| !*bit);
    let bit_count = bits.clone().count();
    let sign = u64::from(negative) << 63;
    if bit_count == 0 {
        return Some(f64::from_bits(sign));
    }
    let mut exponent = exponent
        .saturating_sub(fractional as i64 * 4)
        .saturating_add(bit_count as i64 - 1);
    if exponent > 1023 {
        return Some(f64::from_bits(sign | (0x7ff << 52)));
    }
    if exponent < -1075 {
        return Some(f64::from_bits(sign));
    }
    let precision = if exponent >= -1022 {
        53
    } else {
        (exponent + 1075) as usize
    };
    let mut significand = 0_u64;
    for _ in 0..precision {
        significand = (significand << 1) | u64::from(bits.next().unwrap_or(false));
    }
    let guard = bits.next().unwrap_or(false);
    let sticky = bits.any(|bit| bit);
    if guard && (sticky || significand & 1 != 0) {
        significand += 1;
    }
    if exponent < -1022 {
        return Some(f64::from_bits(sign | significand));
    }
    if significand == 1 << 53 {
        significand >>= 1;
        exponent += 1;
    }
    let value = if exponent > 1023 {
        0x7ff << 52
    } else {
        (((exponent + 1023) as u64) << 52) | (significand & ((1 << 52) - 1))
    };
    Some(f64::from_bits(sign | value))
}
