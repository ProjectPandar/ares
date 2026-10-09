use super::{BTreeMap, Runner, Value};

pub(crate) fn vendors(root: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut vendors = Vec::new();
    for entry in entries.flatten() {
        // Only directories that carry a `machine/` inventory contribute
        // printers — filament-only collections like `OrcaFilamentLibrary`
        // have no machine presets and previously produced a fake
        // ARES_ERROR row in the printer denominator.
        if let Some(name) = entry
            .path()
            .join("machine")
            .is_dir()
            .then(|| entry.file_name())
            .and_then(|name| name.to_str().map(ToOwned::to_owned))
        {
            vendors.push(name);
        }
    }
    vendors.sort();
    vendors
}

#[allow(dead_code)]
fn _type_assertions(map: &BTreeMap<String, Value>) {
    let _ = map.len();
    let _: Option<&Runner> = None;
}
