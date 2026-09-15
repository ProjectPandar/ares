//! Line lexer helpers (`PressureEqualizer.cpp` process_line tail) —
//! whitespace/EOL scanning and the int/float scanners.

use crate::ExtrusionRole;

pub(super) fn is_eol(c: char) -> bool {
    c == '\r' || c == '\n' || c == ';'
}

pub(super) fn skip_ws(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

/// `parse_int` — advances past the int; `None` mirrors the
/// `InvalidArgument` throw path.
pub(super) fn split_int(text: &str) -> Option<(i32, &str)> {
    let text = skip_ws(text);
    let digits: usize = text
        .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
        .unwrap_or(text.len());
    if digits == 0 {
        return None;
    }
    let value = text[..digits].parse().ok()?;
    let rest = &text[digits..];
    let next = rest.chars().next();
    match next {
        None => Some((value, rest)),
        Some(c) if c == ' ' || c == '\t' || is_eol(c) => Some((value, rest)),
        _ => None,
    }
}

/// `parse_float` — fast_float semantics with the ws-or-eol terminator check.
pub(super) fn split_float(text: &str) -> Option<(f32, &str)> {
    let (value, rest) = split_float_opt(text)?;
    match rest.chars().next() {
        None => Some((value, rest)),
        Some(c) if c == ' ' || c == '\t' || is_eol(c) => Some((value, rest)),
        _ => None,
    }
}

pub(super) fn split_float_opt(text: &str) -> Option<(f32, &str)> {
    let text = skip_ws(text);
    let mut end = 0;
    let bytes = text.as_bytes();
    if end < bytes.len() && (bytes[end] == b'-' || bytes[end] == b'+') {
        end += 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
    }
    if end < bytes.len() && (bytes[end] == b'e' || bytes[end] == b'E') {
        let mut exp = end + 1;
        if exp < bytes.len() && (bytes[exp] == b'-' || bytes[exp] == b'+') {
            exp += 1;
        }
        let exp_start = exp;
        while exp < bytes.len() && bytes[exp].is_ascii_digit() {
            exp += 1;
        }
        if exp > exp_start {
            end = exp;
        }
    }
    if end == 0 {
        return None;
    }
    let value: f32 = text[..end].parse().ok()?;
    Some((value, &text[end..]))
}

/// `;_EXTRUSION_ROLE:<n>` — the upstream enum's discriminants
/// (`ExtrusionRole.hpp`). Ares's `ExtrusionRole` re-declares the same
/// order; map by discriminant to keep the marker byte-compatible.
pub(super) fn role_from_u32(value: u32) -> ExtrusionRole {
    const ALL: [ExtrusionRole; 20] = [
        ExtrusionRole::None,
        ExtrusionRole::Perimeter,
        ExtrusionRole::ExternalPerimeter,
        ExtrusionRole::OverhangPerimeter,
        ExtrusionRole::InternalInfill,
        ExtrusionRole::SolidInfill,
        ExtrusionRole::TopSolidInfill,
        ExtrusionRole::BottomSurface,
        ExtrusionRole::Ironing,
        ExtrusionRole::BridgeInfill,
        ExtrusionRole::InternalBridgeInfill,
        ExtrusionRole::GapFill,
        ExtrusionRole::Skirt,
        ExtrusionRole::Brim,
        ExtrusionRole::SupportMaterial,
        ExtrusionRole::SupportMaterialInterface,
        ExtrusionRole::SupportTransition,
        ExtrusionRole::WipeTower,
        ExtrusionRole::Custom,
        ExtrusionRole::Mixed,
    ];
    ALL.get(value as usize)
        .copied()
        .unwrap_or(ExtrusionRole::None)
}
