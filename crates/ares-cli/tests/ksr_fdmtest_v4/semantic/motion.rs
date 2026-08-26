use std::collections::BTreeMap;

use super::number::canonical_number;

pub(super) struct Motion {
    pub(super) command: String,
    pub(super) values: BTreeMap<char, String>,
}

impl Motion {
    pub(super) fn parse(line: &str) -> Result<Option<Self>, String> {
        let mut tokens = line.split_whitespace();
        let Some(command @ ("G0" | "G1" | "G2" | "G3")) = tokens.next() else {
            return Ok(None);
        };
        let mut values = BTreeMap::new();
        for token in tokens {
            let Some(key @ ('X' | 'Y' | 'Z' | 'I' | 'J' | 'E' | 'F' | 'P')) = token.chars().next()
            else {
                continue;
            };
            values.insert(key, canonical_number(&token[1..])?);
        }
        Ok(Some(Self {
            command: command.to_owned(),
            values,
        }))
    }
}
