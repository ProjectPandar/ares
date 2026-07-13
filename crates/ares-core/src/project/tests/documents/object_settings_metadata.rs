use crate::project::{
    model_settings::{Metadata, ModelSettings, ObjectSettings},
    xml::{XmlRole, deserialize_xml},
};
use crate::{
    SliceError,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
};

mod cases;
mod decode;
mod document;
#[allow(
    dead_code,
    clippy::duplicate_mod,
    reason = "this test-owned independent oracle is deliberately reused across option and XML paths"
)]
#[path = "../../../options/tests/process_object_source/expected.rs"]
mod expected;
mod normalization;
mod projection;
mod region_handoff;

fn parse_settings(xml: &str) -> Result<ModelSettings, SliceError> {
    deserialize_xml(xml.as_bytes(), XmlRole::ModelSettings)
}

fn parse_single(key: &str, value: &str) -> Result<ObjectSettings, SliceError> {
    let xml = format!(
        r#"<config><object id="2"><metadata key="{key}" value="{value}"/></object></config>"#
    );
    let mut settings = parse_settings(&xml)?;
    Ok(settings.objects.remove(0))
}

fn object_overrides(object: &ObjectSettings) -> &ObjectOptionOverrides {
    &object.overrides
}

fn object_name(object: &ObjectSettings) -> &str {
    &object.name
}

fn object_module(object: &ObjectSettings) -> &str {
    &object.module
}

fn retained_config(object: &ObjectSettings) -> &[Metadata] {
    &object.retained_config
}

fn region_overrides(object: &ObjectSettings) -> &RegionOptionOverrides {
    &object.region_overrides
}

fn pairs(entries: &[Metadata]) -> Vec<(&str, &str)> {
    entries
        .iter()
        .map(|entry| (entry.key.as_str(), entry.value.as_str()))
        .collect()
}
