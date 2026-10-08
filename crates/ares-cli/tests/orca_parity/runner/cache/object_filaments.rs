use super::{Stage, StageError, io};
use serde_json::Value;
use std::io::Read as _;

/// Rewrite the exported project so the i-th object prints with
/// `object_filaments[i]` (1-based extruder), and expand single-entry
/// `filament_*` lists to the filament count. The native export composes
/// every object with `extruder=1` and collapses identical per-filament
/// values to one entry; the UI saves N entries for every filament option,
/// and the G-code generation reads per-filament vectors by index (a
/// 1-entry list for a 2-filament project crashes it). Both patches live
/// in `Metadata/model_settings.config` and `Metadata/project_settings.config`,
/// which both slicers read from the 3mf.
pub(super) fn assign_object_filaments(
    project: &[u8],
    object_filaments: &[u32],
) -> Result<Vec<u8>, StageError> {
    if object_filaments.is_empty() {
        return Ok(project.to_vec());
    }
    let settings_name = "Metadata/model_settings.config";
    let project_settings_name = "Metadata/project_settings.config";
    let mut source = zip::ZipArchive::new(std::io::Cursor::new(project))
        .map_err(|error| io(Stage::Export, error))?;
    let mut settings = String::new();
    source
        .by_name(settings_name)
        .map_err(|error| io(Stage::Export, error))?
        .read_to_string(&mut settings)
        .map_err(|error| io(Stage::Export, error))?;
    let patched = patch_extruder_metadata(&settings, object_filaments);
    let mut project_settings = String::new();
    source
        .by_name(project_settings_name)
        .map_err(|error| io(Stage::Export, error))?
        .read_to_string(&mut project_settings)
        .map_err(|error| io(Stage::Export, error))?;
    let expanded = expand_filament_lists(&project_settings);
    let mut output = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut output);
        let options: zip::write::SimpleFileOptions = Default::default();
        for index in 0..source.len() {
            let mut member = source
                .by_index(index)
                .map_err(|error| io(Stage::Export, error))?;
            if member.name() == settings_name {
                writer
                    .start_file(settings_name, options)
                    .map_err(|error| io(Stage::Export, error))?;
                std::io::Write::write_all(&mut writer, patched.as_bytes())
                    .map_err(|error| io(Stage::Export, error))?;
            } else if member.name() == project_settings_name {
                writer
                    .start_file(project_settings_name, options)
                    .map_err(|error| io(Stage::Export, error))?;
                std::io::Write::write_all(&mut writer, expanded.as_bytes())
                    .map_err(|error| io(Stage::Export, error))?;
            } else {
                let mut bytes = Vec::new();
                member
                    .read_to_end(&mut bytes)
                    .map_err(|error| io(Stage::Export, error))?;
                writer
                    .start_file(member.name().to_owned(), options)
                    .map_err(|error| io(Stage::Export, error))?;
                std::io::Write::write_all(&mut writer, &bytes)
                    .map_err(|error| io(Stage::Export, error))?;
            }
        }
        writer.finish().map_err(|error| io(Stage::Export, error))?;
    }
    Ok(output.into_inner())
}

/// Replace the `extruder` metadata value inside each `<object>` block, in
/// document order.
fn patch_extruder_metadata(settings: &str, object_filaments: &[u32]) -> String {
    let mut result = String::with_capacity(settings.len());
    let mut tail = settings;
    let mut index = 0usize;
    while let Some(object_start) = tail.find("<object ") {
        let Some(close_rel) = tail[object_start..].find("</object>") else {
            break;
        };
        let block_end = object_start + close_rel + "</object>".len();
        result.push_str(&tail[..object_start]);
        let block = &tail[object_start..block_end];
        let patched = object_filaments
            .get(index)
            .and_then(|filament| set_extruder(block, *filament))
            .unwrap_or_else(|| block.to_owned());
        result.push_str(&patched);
        index += 1;
        tail = &tail[block_end..];
    }
    result.push_str(tail);
    result
}

fn set_extruder(block: &str, filament: u32) -> Option<String> {
    let key = "<metadata key=\"extruder\" value=\"";
    let position = block.find(key)?;
    let value_start = position + key.len();
    let length = block[value_start..].find('"')?;
    let mut patched = block.to_string();
    patched.replace_range(value_start..value_start + length, &filament.to_string());
    Some(patched)
}

/// Expand every single-entry `filament_*` list to the filament count (the
/// count of `filament_colour`), matching the shape the UI saves for a
/// multi-filament project.
fn expand_filament_lists(settings: &str) -> String {
    let mut value: serde_json::Value = match serde_json::from_str(settings) {
        Ok(value) => value,
        Err(_) => return settings.to_owned(),
    };
    let Some(map) = value.as_object_mut() else {
        return settings.to_owned();
    };
    let Some(count) = map
        .get("filament_colour")
        .and_then(Value::as_array)
        .map(Vec::len)
    else {
        return settings.to_owned();
    };
    if count < 2 {
        return settings.to_owned();
    }
    for (key, entry) in map.iter_mut() {
        if !key.starts_with("filament_") {
            continue;
        }
        let Some(list) = entry.as_array_mut() else {
            continue;
        };
        while !list.is_empty() && list.len() < count {
            let last = list.last().cloned();
            if let Some(last) = last {
                list.push(last);
            }
        }
    }
    serde_json::to_string(&value).unwrap_or_else(|_| settings.to_owned())
}
