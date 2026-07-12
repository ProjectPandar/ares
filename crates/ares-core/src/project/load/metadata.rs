use std::collections::{BTreeMap, BTreeSet};

use crate::{SliceError, Transform3d};

use super::super::{
    domain::PlateMetadata,
    filament_sequence::FilamentSequences,
    model_settings::{Metadata, ModelSettings},
    plate::PlateJson,
};

pub(super) struct MetadataIndex {
    pub plates: Vec<PlateMetadata>,
    pub loaded_labels: BTreeMap<(u32, u32), u32>,
    pub part_transforms: BTreeMap<u32, BTreeMap<u32, Transform3d>>,
}

pub(super) fn index(settings: &ModelSettings) -> Result<MetadataIndex, SliceError> {
    let mut part_transforms = BTreeMap::new();
    for object in &settings.objects {
        let mut parts = BTreeMap::new();
        for part in &object.parts {
            let transform = match optional_value(&part.metadata, "matrix")? {
                Some(value) => Transform3d::parse_row_major(value)?,
                None => Transform3d::IDENTITY,
            };
            if parts.insert(part.id, transform).is_some() {
                return Err(invalid(format!(
                    "object {} repeats part {}",
                    object.id, part.id
                )));
            }
        }
        if part_transforms.insert(object.id, parts).is_some() {
            return Err(invalid(format!(
                "model settings repeat object {}",
                object.id
            )));
        }
    }

    let mut plates = Vec::with_capacity(settings.plates.len());
    let mut plate_ids = BTreeSet::new();
    let mut loaded_labels = BTreeMap::new();
    for plate in &settings.plates {
        let id = parse_u32(required_value(&plate.metadata, "plater_id")?, "plate ID")?;
        if id == 0 || !plate_ids.insert(id) {
            return Err(invalid(format!("invalid or repeated plate ID {id}")));
        }
        let mut instances = Vec::with_capacity(plate.model_instances.len());
        for instance in &plate.model_instances {
            let object_id = parse_u32(
                required_value(&instance.metadata, "object_id")?,
                "plate object ID",
            )?;
            let instance_id = parse_u32(
                required_value(&instance.metadata, "instance_id")?,
                "plate instance ID",
            )?;
            let loaded_label_id = parse_u32(
                required_value(&instance.metadata, "identify_id")?,
                "loaded label ID",
            )?;
            if loaded_label_id == 0
                || loaded_labels
                    .insert((object_id, instance_id), loaded_label_id)
                    .is_some()
            {
                return Err(invalid("invalid or repeated plate instance identity"));
            }
            instances.push([object_id, instance_id, loaded_label_id]);
        }
        plates.push(PlateMetadata::new(id, instances));
    }
    Ok(MetadataIndex {
        plates,
        loaded_labels,
        part_transforms,
    })
}

pub(super) fn validate_filament_plates(
    metadata: &MetadataIndex,
    sequences: &FilamentSequences,
) -> Result<(), SliceError> {
    let expected = metadata
        .plates
        .iter()
        .map(PlateMetadata::id)
        .collect::<BTreeSet<_>>();
    let actual = sequences
        .0
        .keys()
        .map(|plate_id| plate_id.get())
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(invalid(
            "filament sequence plates do not match model settings plates",
        ));
    }
    Ok(())
}

pub(super) fn validate_plate_document(document: &PlateJson) -> Result<(), SliceError> {
    for object in &document.bbox_objects {
        u32::try_from(object.id)
            .map_err(|_| invalid(format!("plate object ID {} is invalid", object.id)))?;
    }
    Ok(())
}

pub(super) fn validate_assemble(
    settings: &ModelSettings,
    instances: &BTreeSet<(u32, u32)>,
) -> Result<(), SliceError> {
    let Some(assemble) = &settings.assemble else {
        return Ok(());
    };
    for item in &assemble.items {
        if !instances.contains(&(item.object_id, item.instance_id)) {
            return Err(invalid(format!(
                "assemble item references missing object/instance ({}, {})",
                item.object_id, item.instance_id
            )));
        }
        Transform3d::parse_3mf(&item.transform)?;
        parse_offset(&item.offset)?;
    }
    Ok(())
}

fn required_value<'a>(entries: &'a [Metadata], key: &str) -> Result<&'a str, SliceError> {
    optional_value(entries, key)?.ok_or_else(|| invalid(format!("missing metadata {key:?}")))
}

fn optional_value<'a>(entries: &'a [Metadata], key: &str) -> Result<Option<&'a str>, SliceError> {
    let mut values = entries
        .iter()
        .filter(|entry| entry.key == key)
        .map(|entry| entry.value.as_str());
    let value = values.next();
    if values.next().is_some() {
        return Err(invalid(format!("repeated metadata {key:?}")));
    }
    Ok(value)
}

fn parse_u32(value: &str, name: &str) -> Result<u32, SliceError> {
    value
        .parse()
        .map_err(|_| invalid(format!("{name} {value:?} is invalid")))
}

fn parse_offset(value: &str) -> Result<[f64; 3], SliceError> {
    let values = value.split_ascii_whitespace().collect::<Vec<_>>();
    if values.len() != 3 {
        return Err(invalid("assemble offset must contain three numbers"));
    }
    let mut output = [0.0_f64; 3];
    for (output, value) in output.iter_mut().zip(values) {
        *output = value
            .parse()
            .map_err(|_| invalid("assemble offset contains an invalid number"))?;
        if !output.is_finite() {
            return Err(invalid("assemble offset values must be finite"));
        }
    }
    Ok(output)
}

fn invalid(reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("invalid project metadata: {reason}"))
}
