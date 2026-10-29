use std::collections::{BTreeMap, BTreeSet};

use super::numeric::{atof, atoi};
use crate::SliceError;

use super::{PackagePath, ProjectArchive, domain::ProjectObject};

const PROFILE_PATH: &str = "Metadata/layer_heights_profile.txt";

pub(super) fn load(
    archive: &mut ProjectArchive<'_>,
    paths: &BTreeSet<PackagePath>,
    objects: &mut [ProjectObject],
) -> Result<(), SliceError> {
    let mut paths = paths
        .iter()
        .filter(|path| path.as_str().eq_ignore_ascii_case(PROFILE_PATH))
        .collect::<Vec<_>>();
    // bbs_3mf.cpp:1883-1898 processes members in archive order, case-insensitively.
    paths.sort_by_key(|path| archive.entry_index(path));
    let mut profiles = BTreeMap::new();
    for path in paths {
        let bytes = archive.read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        let text = text.strip_suffix('\n').unwrap_or(&text);
        for record in text.split('\n') {
            import_record(record, &mut profiles);
        }
    }
    // bbs_3mf.cpp:2087-2090: original one-based ModelObject ordinal, before plate selection.
    for (index, object) in objects.iter_mut().enumerate() {
        if let Some(profile) = profiles.remove(&((index + 1) as i32)) {
            object.set_layer_height_profile(profile);
        }
    }
    Ok(())
}

fn import_record(record: &str, profiles: &mut BTreeMap<i32, Vec<f64>>) {
    // bbs_3mf.cpp:2840-2865: invalid records are nonfatal; retain the first ordinal.
    let Some((header, values)) = pair(record, '|') else {
        return;
    };
    let Some((_, ordinal)) = pair(header, '=') else {
        return;
    };
    let ordinal = atoi(ordinal);
    if ordinal == 0 || profiles.contains_key(&ordinal) {
        return;
    }
    let values = values.split(';').collect::<Vec<_>>();
    // bbs_3mf.cpp:2867-2871: four values and odd counts are rejected records, not slice errors.
    if values.len() <= 4 || !values.len().is_multiple_of(2) {
        return;
    }
    // bbs_3mf.cpp:2877-2878 uses atof, including prefix conversion and no-conversion zero.
    profiles.insert(ordinal, values.into_iter().map(atof).collect());
}

fn pair(text: &str, separator: char) -> Option<(&str, &str)> {
    text.split_once(separator)
        .filter(|(_, tail)| !tail.contains(separator))
}
