//! The GUI's loaded `PresetBundle`: `load_presets` runs at startup (GUI_App.cpp:3723) and after the
//! configuration wizard (ConfigWizard.cpp:2597), padding each preset's variant vectors as it loads
//! (Preset.cpp:1930, 1944). Every later `load_config_model` selects from those collections.

use super::collection::Collection;
use crate::{ProfileKind, SliceError, profiles::catalog::Catalog};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// A catalog with its process, printer and filament collections, loaded on first use.
pub(crate) struct LoadedCatalog {
    json: String,
    pub(in crate::profiles) catalog: Catalog,
    collections: OnceLock<[Collection; 3]>,
}

/// The last catalog loaded, so a session loads and pads its catalog once.
static LAST: Mutex<Option<Arc<LoadedCatalog>>> = Mutex::new(None);

/// The loaded catalog of `json`: the last one when `json` is unchanged.
pub(crate) fn loaded_catalog(json: &str) -> Result<Arc<LoadedCatalog>, SliceError> {
    let mut last = LAST.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(loaded) = last.as_ref().filter(|loaded| loaded.json == json) {
        return Ok(Arc::clone(loaded));
    }
    let loaded = Arc::new(LoadedCatalog {
        json: json.to_owned(),
        catalog: Catalog::from_json(json)?,
        collections: OnceLock::new(),
    });
    *last = Some(Arc::clone(&loaded));
    Ok(loaded)
}

impl LoadedCatalog {
    /// The loaded collection of `kind`, which a selection copies before it adds project presets.
    pub(super) fn collection(&self, kind: ProfileKind) -> Result<&Collection, SliceError> {
        let collections = match self.collections.get() {
            Some(collections) => collections,
            None => {
                let loaded = [
                    Collection::load(&self.catalog, ProfileKind::Process)?,
                    Collection::load(&self.catalog, ProfileKind::Filament)?,
                    Collection::load(&self.catalog, ProfileKind::Machine)?,
                ];
                self.collections.get_or_init(|| loaded)
            }
        };
        Ok(&collections[kind as usize])
    }
}

/// `PresetBundle::load_presets` for `catalog_json` ahead of a project load, at startup
/// (GUI_App.cpp:3723) or after the configuration wizard (ConfigWizard.cpp:2597).
pub fn load_preset_bundle(catalog_json: &str) -> Result<(), SliceError> {
    loaded_catalog(catalog_json)?
        .collection(ProfileKind::Process)
        .map(drop)
}
