use std::collections::BTreeSet;

use serde::Deserialize;

use super::super::{
    PackagePath, ProjectArchive,
    xml::{XmlRole, deserialize_xml},
};
use super::deferred;
use crate::{Project, SliceError};

mod preview;

pub(super) const PREVIEW_REFERENCES: [(&str, &str); 4] = [
    ("thumbnail_file", "Metadata/plate_1.png"),
    ("thumbnail_no_light_file", "Metadata/plate_no_light_1.png"),
    ("top_file", "Metadata/top_1.png"),
    ("pick_file", "Metadata/pick_1.png"),
];
pub(super) const SMALL_PREVIEW: &str = "Metadata/plate_1_small.png";
const CUT_INFORMATION: &str = "Metadata/cut_information.xml";

#[derive(Default)]
pub(super) struct SavedArchiveExtras {
    pub previews: Option<PlatePreviews>,
    pub auxiliary: Vec<(String, Vec<u8>)>,
}

pub(super) struct PlatePreviews {
    pub primary: Vec<u8>,
    pub small: Vec<u8>,
    pub secondary: [Vec<u8>; 3],
}

pub(super) fn load(
    archive: &mut ProjectArchive<'_>,
    project: &Project,
    mut additional: Vec<PackagePath>,
) -> Result<SavedArchiveExtras, SliceError> {
    let unsupported = || deferred("additional saved archive members");
    let partial_previews = || deferred("incomplete saved plate previews");
    let mut referenced = [None; 4];
    for plate in &project.documents().model_settings.plates {
        for field in &plate.metadata {
            if let Some(index) = PREVIEW_REFERENCES
                .iter()
                .position(|(key, _)| *key == field.key)
                && referenced[index].replace(field.value.as_str()).is_some()
            {
                return Err(partial_previews());
            }
        }
    }
    let has_previews = referenced.iter().flatten().any(|path| !path.is_empty());
    if has_previews
        && !referenced
            .into_iter()
            .zip(PREVIEW_REFERENCES)
            .all(|(input, (_, expected))| input == Some(expected))
    {
        return Err(partial_previews());
    }

    additional.sort_unstable_by_key(|path| archive.entry_index(path));
    let mut images = [const { None }; 4];
    let mut auxiliary = Vec::new();
    let mut group_order = [None; 3];
    for path in additional {
        let name = archive.entry_name(&path)?;
        if has_previews
            && let Some(index) = PREVIEW_REFERENCES
                .iter()
                .position(|(_, entry)| *entry == name)
        {
            images[index] = Some(read_nonempty(archive, &path)?);
        } else if name == CUT_INFORMATION {
            validate_inert_cut(&archive.read(&path)?, project.objects().len())?;
        } else if let Some(group) = auxiliary_group(&name) {
            // bbs_3mf.cpp:9136–9189 walks directories breadth first. These picture
            // paths have equal depth; first appearance orders sibling directories.
            let first = *group_order[group].get_or_insert_with(|| archive.entry_index(&path));
            auxiliary.push((first, (name, read_nonempty(archive, &path)?)));
        } else {
            return Err(unsupported());
        }
    }
    auxiliary.sort_by_key(|(group, _)| *group);
    let previews = if has_previews {
        let [Some(primary), Some(no_light), Some(top), Some(pick)] = images else {
            return Err(partial_previews());
        };
        let (primary, small) = preview::regenerate(&primary)?;
        Some(PlatePreviews {
            primary,
            small,
            secondary: [no_light, top, pick],
        })
    } else {
        None
    };
    Ok(SavedArchiveExtras {
        previews,
        auxiliary: auxiliary.into_iter().map(|(_, member)| member).collect(),
    })
}

fn read_nonempty(
    archive: &mut ProjectArchive<'_>,
    path: &PackagePath,
) -> Result<Vec<u8>, SliceError> {
    let bytes = archive.read(path)?;
    if bytes.is_empty() {
        return Err(deferred("empty saved image"));
    }
    Ok(bytes)
}

fn auxiliary_group(name: &str) -> Option<usize> {
    let (group, file) = name.strip_prefix("Auxiliaries/")?.split_once('/')?;
    match group {
        ".thumbnails"
            if matches!(
                file,
                "thumbnail_3mf.png" | "thumbnail_middle.png" | "thumbnail_small.png"
            ) =>
        {
            Some(0)
        }
        "Model Pictures" | "Profile Pictures"
            if !file.contains('/')
                && file
                    .strip_suffix(".webp")
                    .is_some_and(|stem| !stem.is_empty()) =>
        {
            Some(if group == "Model Pictures" { 1 } else { 2 })
        }
        _ => None,
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CutInformation {
    #[serde(rename = "object", default)]
    objects: Vec<CutObject>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CutObject {
    #[serde(rename = "@id")]
    ordinal: usize,
    cut_id: CutId,
    #[serde(rename = "connectors")]
    _connectors: Option<NoConnectors>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CutId {
    #[serde(rename = "@id")]
    id: u64,
    #[serde(rename = "@check_sum")]
    _checksum: u64,
    #[serde(rename = "@connectors_cnt")]
    connector_count: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoConnectors {}

fn validate_inert_cut(bytes: &[u8], objects: usize) -> Result<(), SliceError> {
    let cut: CutInformation = deserialize_xml(bytes, XmlRole::CutInformation)?;
    let mut ordinals = BTreeSet::new();
    if cut.objects.into_iter().any(|object| {
        !(1..=objects).contains(&object.ordinal)
            || !ordinals.insert(object.ordinal)
            || object.cut_id.id != 0
            || object.cut_id.connector_count != 0
    }) {
        return Err(deferred("active or unrecognized cut information"));
    }
    Ok(())
}
