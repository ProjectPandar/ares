use std::path::Path;

#[path = "discovery_tests.rs"]
mod tests;

pub(super) fn vendors(root: &Path) -> Result<Vec<String>, String> {
    let mut vendors = Vec::new();
    for entry in std::fs::read_dir(root).map_err(|error| format!("{root:?}: {error}"))? {
        let entry = entry.map_err(|error| format!("{root:?}: {error}"))?;
        let path = entry.path();
        if !std::fs::metadata(&path)
            .map_err(|error| format!("{path:?}: {error}"))?
            .is_dir()
        {
            continue;
        }
        let machine = path.join("machine");
        match std::fs::symlink_metadata(&machine) {
            Ok(_) => {
                if !std::fs::metadata(&machine)
                    .map_err(|error| format!("{machine:?}: {error}"))?
                    .is_dir()
                {
                    return Err(format!("{machine:?}: machine inventory is not a directory"));
                }
                vendors.push(
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|name| format!("non-UTF8 vendor name {name:?}"))?,
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{machine:?}: {error}")),
        }
    }
    vendors.sort();
    Ok(vendors)
}
