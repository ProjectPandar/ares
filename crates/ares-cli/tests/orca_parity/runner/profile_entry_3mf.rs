//! Selected-profile slicing over a Bambu/Orca 3MF: the embedded
//! `project_settings.config` is the full-config base and the loaded presets
//! overlay only the keys they supply
//! (`OrcaSlicer.cpp:1624-1640,2750-2812,2849-2867,2993-3011,3115-3337`).
//! Native reference: `--export-3mf` followed by `3mf --load-settings` on the
//! same flattened presets; the Rust side receives the raw vendor catalog.

use serde_json::{Value, json};

use super::{
    OrcaRunner,
    profile_entry::{placed_cube, vendor, write_json},
    stages,
};

include!("profile_entry_3mf/load_settings.rs");

include!("profile_entry_3mf/overrides.rs");

include!("profile_entry_3mf/settings_3mf.rs");

include!("profile_entry_3mf/settings_stl.rs");

include!("profile_entry_3mf/post_process.rs");

#[path = "profile_entry_3mf/machine_start_end_marker.rs"]
mod machine_start_end_marker;
#[path = "minimal_purge_on_wipe_tower.rs"]
mod minimal_purge_on_wipe_tower;
