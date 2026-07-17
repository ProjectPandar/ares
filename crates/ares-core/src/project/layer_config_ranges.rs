use std::collections::BTreeSet;

use serde::Deserialize;

use crate::{
    OrcaFloat, SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
};

use super::{
    PackagePath, ProjectArchive,
    domain::ProjectObject,
    xml::{XmlRole, deserialize_xml},
};

const LAYER_CONFIG_RANGES_PATH: &str = "Metadata/layer_config_ranges.xml";
const INVALID_PREFIX: &str = "invalid project layer configuration ranges XML: ";

#[derive(Clone, Debug, PartialEq)]
pub struct LayerConfigRange {
    min_z: f64,
    max_z: f64,
    layer_height: Option<OrcaFloat>,
    region_overrides: RegionOptionOverrides,
}

impl LayerConfigRange {
    pub fn min_z(&self) -> f64 {
        self.min_z
    }

    pub fn max_z(&self) -> f64 {
        self.max_z
    }

    pub(crate) fn layer_height(&self) -> Option<OrcaFloat> {
        self.layer_height
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn region_overrides(&self) -> &RegionOptionOverrides {
        &self.region_overrides
    }
}

#[derive(Deserialize)]
#[serde(rename = "objects", deny_unknown_fields)]
struct LayerConfigDocument {
    #[serde(rename = "object", default)]
    objects: Vec<LayerObject>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerObject {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "range", default)]
    ranges: Vec<LayerRange>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerRange {
    #[serde(rename = "@min_z")]
    min_z: String,
    #[serde(rename = "@max_z")]
    max_z: String,
    #[serde(rename = "option", default)]
    options: Vec<LayerOption>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerOption {
    #[serde(rename = "@opt_key")]
    key: String,
    #[serde(rename = "$text", default)]
    value: String,
}

pub(crate) fn load(
    archive: &mut ProjectArchive<'_>,
    archive_paths: &BTreeSet<PackagePath>,
    objects: &mut [ProjectObject],
) -> Result<(), SliceError> {
    let mut paths = archive_paths
        .iter()
        .filter(|path| path.as_str().eq_ignore_ascii_case(LAYER_CONFIG_RANGES_PATH));
    let Some(path) = paths.next().cloned() else {
        return Ok(());
    };
    if paths.next().is_some() {
        return Err(invalid(format!(
            "ambiguous archive path {LAYER_CONFIG_RANGES_PATH}"
        )));
    }

    let document =
        deserialize_xml::<LayerConfigDocument>(&archive.read(&path)?, XmlRole::LayerConfigRanges)
            .map_err(|error| {
            let message = error.to_string();
            let reason = message.strip_prefix(INVALID_PREFIX).unwrap_or(&message);
            invalid(bounded(reason, 416))
        })?;
    associate(document, objects)
}

fn associate(
    document: LayerConfigDocument,
    objects: &mut [ProjectObject],
) -> Result<(), SliceError> {
    let mut ordinals = BTreeSet::new();
    for object in document.objects {
        let ordinal = parse_ordinal(&object.id)?;
        if ordinal == 0 || ordinal > objects.len() {
            return Err(invalid(format!(
                "object ordinal {} is out of range",
                bounded(&object.id, 96)
            )));
        }
        if !ordinals.insert(ordinal) {
            return Err(invalid(format!("duplicate object ordinal {ordinal}")));
        }

        let ranges = parse_ranges(object.ranges)?;
        if !ranges.is_empty() {
            objects[ordinal - 1].set_layer_config_ranges(ranges);
        }
    }
    Ok(())
}

fn parse_ordinal(value: &str) -> Result<usize, SliceError> {
    value
        .parse()
        .map_err(|_| invalid(format!("object ordinal {} is invalid", bounded(value, 96))))
}

fn parse_ranges(source: Vec<LayerRange>) -> Result<Vec<LayerConfigRange>, SliceError> {
    let mut ranges = Vec::<LayerConfigRange>::with_capacity(source.len());
    for range in source {
        let min_z = parse_bound(&range.min_z, "min_z")?;
        let max_z = parse_bound(&range.max_z, "max_z")?;
        let mut layer_height = None;
        let mut region_overrides = RegionOptionOverrides::default();
        for option in range.options {
            if option.key == "layer_height" {
                let mut object_overrides = ObjectOptionOverrides::default();
                super::super::options::deserialize_object_model_field(
                    option.key,
                    option.value,
                    &mut object_overrides,
                    &mut region_overrides,
                )
                .map_err(|error| invalid(bounded(&error.to_string(), 416)))?;
                layer_height = object_overrides.layer_height;
            } else {
                super::super::options::deserialize_region_model_field(
                    option.key,
                    option.value,
                    &mut region_overrides,
                )
                .map_err(|error| invalid(bounded(&error.to_string(), 416)))?;
            }
        }
        let parsed = LayerConfigRange {
            min_z,
            max_z,
            layer_height,
            region_overrides,
        };
        if let Some(index) = ranges
            .iter()
            .position(|old| old.min_z == min_z && old.max_z == max_z)
        {
            ranges[index] = parsed;
        } else {
            ranges.push(parsed);
        }
    }
    ranges.sort_by(|left, right| {
        left.min_z
            .total_cmp(&right.min_z)
            .then_with(|| left.max_z.total_cmp(&right.max_z))
    });
    Ok(ranges)
}

fn parse_bound(value: &str, name: &str) -> Result<f64, SliceError> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| invalid(format!("{name} {} is invalid", bounded(value, 96))))?;
    if !parsed.is_finite() {
        return Err(invalid(format!("{name} must be finite")));
    }
    Ok(parsed)
}

fn invalid(reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("{INVALID_PREFIX}{reason}"))
}

fn bounded(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        return value.to_owned();
    }
    let mut end = limit - 3;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &value[..end])
}
