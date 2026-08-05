use serde_json::Value;

use crate::SliceError;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ExtraSolidInfills {
    entries: Vec<ExtraSolidEntry>,
}

#[derive(Clone, Debug, PartialEq)]
enum ExtraSolidEntry {
    Repeating { interval: usize, count: usize },
    ExplicitRange { start: usize, count: usize },
}

impl ExtraSolidInfills {
    pub(super) fn parse(value: Option<&Value>) -> Result<Self, SliceError> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let Some(raw) = value.as_str() else {
            return Err(invalid());
        };
        Self::parse_raw(raw)
    }

    pub(crate) fn parse_raw(raw: &str) -> Result<Self, SliceError> {
        let pattern = normalize_pattern(raw);
        if pattern.is_empty() {
            return Ok(Self::default());
        }

        if pattern.contains(',') {
            parse_list(&pattern)
        } else {
            parse_repeating(&pattern)
        }
    }

    pub(crate) fn matches_layer(&self, layer_index: usize) -> bool {
        let Some(layer_number) = layer_index.checked_add(1) else {
            return false;
        };
        self.entries.iter().any(|entry| match *entry {
            ExtraSolidEntry::Repeating { interval, count } => {
                layer_number >= interval && layer_number % interval < count
            }
            ExtraSolidEntry::ExplicitRange { start, count } => {
                start
                    .checked_add(count)
                    .is_some_and(|end| layer_number >= start && layer_number < end)
            }
        })
    }
}

fn normalize_pattern(raw: &str) -> String {
    let mut pattern = raw
        .chars()
        .filter(|ch| !matches!(ch, ' ' | '\t' | '\n' | '\r'))
        .collect::<String>();
    if matches!(pattern.as_bytes().first(), Some(b'"' | b'\'')) {
        pattern.remove(0);
    }
    if matches!(pattern.as_bytes().last(), Some(b'"' | b'\'')) {
        pattern.pop();
    }
    pattern
}

fn parse_list(pattern: &str) -> Result<ExtraSolidInfills, SliceError> {
    let mut entries = Vec::new();
    for token in pattern.split(',') {
        if token.is_empty() {
            return Err(invalid());
        }
        entries.push(parse_explicit_token(token)?);
    }
    Ok(ExtraSolidInfills { entries })
}

fn parse_repeating(pattern: &str) -> Result<ExtraSolidInfills, SliceError> {
    let (interval, count) = if pattern.contains('#') {
        parse_base_count(pattern)?
    } else {
        (parse_positive_usize(pattern)?, 1)
    };
    Ok(ExtraSolidInfills {
        entries: vec![ExtraSolidEntry::Repeating { interval, count }],
    })
}

fn parse_explicit_token(token: &str) -> Result<ExtraSolidEntry, SliceError> {
    if token.contains('#') {
        let (start, count) = parse_base_count(token)?;
        Ok(ExtraSolidEntry::ExplicitRange { start, count })
    } else {
        let start = parse_positive_usize(token)?;
        Ok(ExtraSolidEntry::ExplicitRange { start, count: 1 })
    }
}

fn parse_base_count(token: &str) -> Result<(usize, usize), SliceError> {
    let (base, count) = token.split_once('#').ok_or_else(invalid)?;
    if count.contains('#') {
        return Err(invalid());
    }
    let base = parse_positive_usize(base)?;
    let count = if count.is_empty() {
        1
    } else {
        parse_positive_usize(count)?
    };
    Ok((base, count))
}

fn parse_positive_usize(text: &str) -> Result<usize, SliceError> {
    let parsed = text.parse::<i32>().map_err(|_| invalid())?;
    if parsed <= 0 {
        return Err(invalid());
    }
    Ok(parsed as usize)
}

fn invalid() -> SliceError {
    SliceError::InvalidInput("invalid extra_solid_infills pattern".to_owned())
}
