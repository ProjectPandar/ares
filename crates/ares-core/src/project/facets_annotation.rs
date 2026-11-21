//! Model.cpp::FacetsAnnotation and TriangleSelector::TriangleSplittingData.

use serde::{Deserialize, Deserializer, de::Error};

mod selector;

pub(crate) use selector::{
    Selector as FacetSelector, ordinary_facets, strict_annotated_facets, strict_facets,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TriangleCodes(Vec<u8>);

impl<'de> Deserialize<'de> for TriangleCodes {
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
            pending += validate_node(&codes, &mut cursor).map_err(D::Error::custom)?;
        }
        if cursor != codes.len() {
            return Err(D::Error::custom("trailing facet painting codes"));
        }
        Ok(Self(codes))
    }
}

fn validate_node(codes: &[u8], cursor: &mut usize) -> Result<usize, &'static str> {
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
    if code >> 2 == 3 {
        let state = *codes
            .get(*cursor)
            .ok_or("truncated support painting state")?;
        if state > 13 {
            return Err("invalid support painting state");
        }
        *cursor += 1;
    }
    Ok(0)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FacetsAnnotation {
    pub(crate) mapping: Vec<(usize, usize)>,
    pub(crate) codes: Vec<u8>,
}

impl FacetsAnnotation {
    pub(crate) fn used_states(&self) -> [bool; 17] {
        let mut states = [false; 17];
        let mut codes = self.codes.iter().copied();
        while let Some(code) = codes.next() {
            if code & 3 != 0 {
                continue;
            }
            let state = if code >> 2 == 3 {
                codes
                    .next()
                    .map_or(usize::MAX, |extension| usize::from(extension) + 3)
            } else {
                usize::from(code >> 2)
            };
            // TriangleSelector.cpp:1918 skips states outside the supported palette.
            if let Some(used) = states.get_mut(state) {
                *used = true;
            }
        }
        states
    }

    pub(crate) fn from_triangles<'a>(triangles: impl Iterator<Item = &'a TriangleCodes>) -> Self {
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
