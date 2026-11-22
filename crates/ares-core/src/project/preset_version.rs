use crate::{ORCA_SLICER_COMPATIBILITY_VERSION, SliceError};

use super::{ArchiveLimits, ProjectArchive, model_identity::ModelIdentity};

/// Checks the root model's application version without deserializing its meshes
/// (`bbs_3mf.cpp:1537-1550, 1944-1947`; `OrcaSlicer.cpp:1758-1779`).
pub fn validate_project_version(
    input: impl AsRef<[u8]>,
    allow_newer_file: bool,
) -> Result<(), SliceError> {
    if allow_newer_file {
        return Ok(());
    }
    let mut archive = ProjectArchive::open(input.as_ref(), ArchiveLimits::PROJECT)?;
    let identity = ModelIdentity::read(&mut archive)?;
    let orca = identity
        .metadata()
        .rfind(|entry| entry.name == "OrcaSlicer")
        .and_then(|entry| major_minor(&entry.value));
    let file = orca.or_else(|| {
        identity
            .metadata()
            .rev()
            .filter(|entry| entry.name == "Application")
            .find_map(|entry| {
                entry
                    .value
                    .strip_prefix("OrcaSlicer-")
                    .or_else(|| entry.value.strip_prefix("BambuStudio-"))
            })
            .and_then(major_minor)
    });
    if let (Some(file), Some(current)) = (file, major_minor(ORCA_SLICER_COMPATIBILITY_VERSION))
        && file > current
    {
        return Err(SliceError::FileVersionNotSupported);
    }
    Ok(())
}

// Semver.hpp::parse/to_string; deps_src/semver/semver.c::semver_parse_version.
pub(crate) fn parse(value: &str) -> Option<String> {
    if value.len() > 255
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
    {
        return None;
    }
    let (number, metadata) = value
        .split_once('+')
        .map_or((value, None), |(number, suffix)| (number, Some(suffix)));
    let (number, prerelease) = number
        .split_once('-')
        .map_or((number, None), |(number, suffix)| (number, Some(suffix)));
    let mut parts = [0_i32; 4];
    let mut count = 0;
    for part in number.split('.') {
        if part.len() > 50 {
            return None;
        }
        *parts.get_mut(count)? = if part.is_empty() {
            0
        } else {
            part.parse().ok()?
        };
        count += 1;
    }
    if count < 2 {
        return None;
    }
    let patch = if count == 4 {
        parts[2].checked_mul(100)?.checked_add(parts[3])?
    } else {
        parts[2]
    };
    let mut result = format!("{}.{}.{}.{}", parts[0], parts[1], patch / 100, patch % 100);
    if let Some(suffix) = prerelease {
        result.push('-');
        result.push_str(suffix);
    }
    if let Some(suffix) = metadata {
        result.push('+');
        result.push_str(suffix);
    }
    Some(result)
}

fn major_minor(value: &str) -> Option<(i32, i32)> {
    let normalized = parse(value)?;
    let mut parts = normalized.split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}
