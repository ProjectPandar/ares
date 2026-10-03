//! C-locale strtod word semantics consumed by `GCodeReader.cpp:292–322`.

pub(super) fn parse(word: &str) -> Option<f64> {
    if let Ok(value) = word.parse::<f64>() {
        return Some(value);
    }
    let unsigned = word.strip_prefix(['+', '-']).unwrap_or(word);
    if unsigned
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("nan"))
        && let Some(payload) = unsigned[3..]
            .strip_prefix('(')
            .and_then(|tail| tail.strip_suffix(')'))
        && payload
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Some(f64::NAN);
    }
    hex(word)
}

pub(super) fn consumes_prefix(word: &str) -> bool {
    let word = word.strip_prefix(['+', '-']).unwrap_or(word);
    word.as_bytes().first().is_some_and(u8::is_ascii_digit)
        || word
            .strip_prefix('.')
            .is_some_and(|tail| tail.as_bytes().first().is_some_and(u8::is_ascii_digit))
        || word.get(..3).is_some_and(|prefix| {
            prefix.eq_ignore_ascii_case("inf") || prefix.eq_ignore_ascii_case("nan")
        })
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
            let digit = char::from(byte)
                .to_digit(16)
                .expect("validated hexadecimal digit");
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
