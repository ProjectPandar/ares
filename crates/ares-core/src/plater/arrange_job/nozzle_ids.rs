//! The per-polygon bed-exclusion nozzle ids of the GUI arrange job (`ArrangeJob.cpp:53-103` at
//! v2.5.0-beta `345efec64`). See `docs/architecture/web-plater-arrange-imex.md`, "A2b3".

use crate::PresetCatalogHandle;
use crate::plater::PlaterSession;

impl PlaterSession {
    /// The installed system vendors, `PresetBundle::vendors`, which the vendor test reads
    /// (`get_current_vendor_type`, `PresetBundle.cpp:992-1024`). Until set, none is installed.
    pub fn set_installed_vendors(&mut self, _vendors: &PresetCatalogHandle) {
        todo!("PlaterSession::set_installed_vendors")
    }
}
