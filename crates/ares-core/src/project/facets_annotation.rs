//! Model.cpp::FacetsAnnotation and TriangleSelector::TriangleSplittingData.

use serde::{Deserialize, Deserializer, de::Error};

mod selector;

pub(super) const SELECTOR_MAX_STATE: u8 = 32;

pub(crate) use selector::{
    Selector as FacetSelector, ordinary_facets, strict_annotated_facets, strict_facets,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TriangleCodes<const MAX_STATE: u8 = 16>(Vec<u8>);

impl<'de, const MAX_STATE: u8> Deserialize<'de> for TriangleCodes<MAX_STATE> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        let codes = text
            .bytes()
            .rev()
            .map(|digit| match digit {
                b'0'..=b'9' => Ok(digit - b'0'),
                b'A'..=b'F' => Ok(digit - b'A' + 10),
                _ => Err(D::Error::custom(
                    "invalid support painting hexadecimal code",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut pending = usize::from(!codes.is_empty());
        let mut cursor = 0;
        while pending != 0 {
            pending -= 1;
            pending += validate_node::<MAX_STATE>(&codes, &mut cursor).map_err(D::Error::custom)?;
        }
        if cursor != codes.len() {
            return Err(D::Error::custom("trailing facet painting codes"));
        }
        Ok(Self(codes))
    }
}

fn validate_node<const MAX_STATE: u8>(
    codes: &[u8],
    cursor: &mut usize,
) -> Result<usize, &'static str> {
    let code = *codes
        .get(*cursor)
        .ok_or("truncated support painting tree")?;
    *cursor += 1;
    let splits = code & 3;
    if splits != 0 {
        let special = code >> 2;
        if special > 2 || (splits == 3 && special != 0) {
            return Err("invalid support painting split side");
        }
        return Ok(usize::from(splits) + 1);
    }
    let state = leaf_state(code, codes, cursor).ok_or("truncated support painting state")?;
    if state > MAX_STATE {
        return Err("invalid support painting state");
    }
    Ok(0)
}

// TriangleSelector.cpp:1855–1870: F escapes into the second extension nibble.
fn leaf_state(code: u8, codes: &[u8], cursor: &mut usize) -> Option<u8> {
    let inline = code >> 2;
    if inline != 3 {
        return Some(inline);
    }
    let extension = *codes.get(*cursor)?;
    *cursor += 1;
    if extension == 15 {
        let high = *codes.get(*cursor)?;
        *cursor += 1;
        Some(high + 18)
    } else {
        Some(extension + 3)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FacetsAnnotation {
    pub(crate) mapping: Vec<(usize, usize)>,
    pub(crate) codes: Vec<u8>,
}

impl FacetsAnnotation {
    pub(crate) fn used_states(&self) -> [bool; 17] {
        let mut states = [false; 17];
        let mut cursor = 0;
        while let Some(&code) = self.codes.get(cursor) {
            cursor += 1;
            if code & 3 != 0 {
                continue;
            }
            let state = leaf_state(code, &self.codes, &mut cursor).expect("validated facet state");
            if let Some(used) = states.get_mut(usize::from(state)) {
                *used = true;
            }
        }
        states
    }

    pub(crate) fn from_triangles<'a, const MAX_STATE: u8>(
        triangles: impl Iterator<Item = &'a TriangleCodes<MAX_STATE>>,
    ) -> Self {
        let mut annotation = Self::default();
        for (triangle, codes) in triangles.enumerate() {
            if !codes.0.is_empty() {
                annotation.mapping.push((triangle, annotation.codes.len()));
                annotation.codes.extend_from_slice(&codes.0);
            }
        }
        annotation
    }
}
