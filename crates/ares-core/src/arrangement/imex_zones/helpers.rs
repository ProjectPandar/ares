//! The `IMEXHelpers` the zone layout calls (`libslic3r/IMEXHelpers.cpp`, `.hpp` at v2.5.0-beta
//! `345efec64`): the mode lookup, the active-tool parsers and the gantry grouping.

use std::collections::BTreeMap;

use crate::DynamicPrintConfig;

/// `MAXIMUM_EXTRUDER_NUMBER` (`libslic3r.h:70`).
const MAXIMUM_EXTRUDER_NUMBER: i32 = 64;

/// `ImexRole` with its on-disk letters (`IMEXHelpers.hpp:345-372`).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ImexRole {
    Primary,
    Copy,
    Mirror,
    Span,
}

/// `imex_role_from_suffix` (`IMEXHelpers.cpp:318-325`): one letter of the role table, else Copy.
fn imex_role_from_suffix(suffix: &str) -> ImexRole {
    match suffix.as_bytes() {
        [b'P'] => ImexRole::Primary,
        [b'C'] => ImexRole::Copy,
        [b'M'] => ImexRole::Mirror,
        [b'S'] => ImexRole::Span,
        _ => ImexRole::Copy,
    }
}

/// `strip_whitespace` (`IMEXHelpers.cpp:44-49`): every `isspace` byte removed.
fn strip_whitespace(token: &str) -> String {
    token
        .chars()
        .filter(|c| !matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r'))
        .collect()
}

/// `std::stoi`: an optional sign and the leading decimal digits; `None` where it throws.
fn stoi(text: &str) -> Option<i32> {
    let text = text.trim_start_matches([' ', '\t', '\n', '\x0b', '\x0c', '\r']);
    let digits_from = usize::from(text.starts_with(['+', '-']));
    let digits = text[digits_from..]
        .bytes()
        .take_while(u8::is_ascii_digit)
        .count();
    if digits == 0 {
        return None;
    }
    text[..digits_from + digits].parse().ok()
}

/// A token's index text and its suffix after ':', if any.
fn split_token(token: &str) -> (&str, Option<&str>) {
    match token.find(':') {
        Some(colon) => (&token[..colon], Some(&token[colon + 1..])),
        None => (token, None),
    }
}

/// `parse_imex_active_tools` (`IMEXHelpers.cpp:327-349`): `(physical index, role)` per token; a
/// bare index is Copy here.
pub(crate) fn parse_imex_active_tools(active_tools: &str) -> Vec<(i32, ImexRole)> {
    active_tools
        .split(',')
        .map(strip_whitespace)
        .filter(|token| !token.is_empty())
        .filter_map(|token| {
            let (index, suffix) = split_token(&token);
            let phys = stoi(index)?;
            (0..MAXIMUM_EXTRUDER_NUMBER)
                .contains(&phys)
                .then(|| (phys, suffix.map_or(ImexRole::Copy, imex_role_from_suffix)))
        })
        .collect()
}

/// `imex_primary_tool_for_mode` (`IMEXHelpers.cpp:422-455`): the `:P` tool, else the first bare
/// index, else -1.
pub(crate) fn imex_primary_tool_for_mode(active_tools: &str) -> i32 {
    let mut first_bare = -1;
    for token in active_tools.split(',').map(strip_whitespace) {
        if token.is_empty() {
            continue;
        }
        let (index, suffix) = split_token(&token);
        let Some(idx) = stoi(index).filter(|&idx| idx >= 0) else {
            continue;
        };
        match suffix {
            None if first_bare < 0 => first_bare = idx,
            None => {}
            Some(suffix) if imex_role_from_suffix(suffix) == ImexRole::Primary => return idx,
            Some(_) => {}
        }
    }
    first_bare
}

/// `ImexGantryGroup` (`IMEXHelpers.hpp:527-533`).
pub(crate) struct ImexGantryGroup {
    pub(crate) gantry_index: i32,
    pub(crate) representative_phys: i32,
    pub(crate) aggregate: bool,
}

/// `group_imex_active_tools_by_gantry` (`IMEXHelpers.cpp:365-420`): the active tools by gantry;
/// with a Span on the primary's gantry, another gantry whose two or more tools share one role
/// collapses to its representative (the tool in the primary's column, else the lowest).
pub(crate) fn group_imex_active_tools_by_gantry(
    active_tools: &str,
    tools_per_gantry: i32,
) -> Vec<ImexGantryGroup> {
    let tpg = tools_per_gantry.max(1);
    let primary = imex_primary_tool_for_mode(active_tools);
    if primary < 0 {
        return Vec::new();
    }
    let (primary_gantry, primary_col) = (primary / tpg, primary % tpg);
    let mut by_gantry = BTreeMap::<i32, Vec<(i32, ImexRole)>>::new();
    let mut span_seen = false;
    for (phys, role) in parse_imex_active_tools(active_tools) {
        let gantry = phys / tpg;
        by_gantry.entry(gantry).or_default().push((phys, role));
        span_seen |= gantry == primary_gantry && role == ImexRole::Span;
    }
    by_gantry
        .into_iter()
        .map(|(gantry, tools)| {
            let paired = gantry * tpg + primary_col;
            let (representative_phys, representative_role) = tools
                .iter()
                .find(|(phys, _)| *phys == paired)
                .or_else(|| tools.iter().min_by_key(|(phys, _)| *phys))
                .copied()
                .expect("a gantry bucket holds a tool");
            let same_role = tools.iter().all(|(_, role)| *role == representative_role);
            ImexGantryGroup {
                gantry_index: gantry,
                representative_phys,
                aggregate: span_seen && gantry != primary_gantry && tools.len() >= 2 && same_role,
            }
        })
        .collect()
}

/// `find_imex_mode(cfg, name).active_tools` (`IMEXHelpers.cpp:648-678`): the mode's
/// `imex_mode_active_tools` entry, "" when the name is missing or that array is short.
pub(crate) fn find_imex_mode_active_tools(cfg: &DynamicPrintConfig, name: &str) -> String {
    let names = cfg.option("imex_mode_names");
    let tools = cfg.option("imex_mode_active_tools");
    names
        .strings()
        .iter()
        .position(|candidate| candidate == name)
        .and_then(|index| tools.strings().get(index).cloned())
        .unwrap_or_default()
}
