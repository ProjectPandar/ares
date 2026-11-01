//! The per-polygon bed-exclusion nozzle ids of the GUI arrange job (`ArrangeJob.cpp:53-103` at
//! v2.5.0-beta `345efec64`). See `docs/architecture/web-plater-arrange-imex.md`, "A2b3".

use super::super::plate_list::InstanceKey;
use crate::arrangement::ArrangePolygon;
use crate::bed_exclusion::{active_bed_exclude_volume_mode, is_auto_filament_map_mode};
use crate::plater::PlaterSession;
use crate::profiles::VendorType;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
use crate::{BedExcludeVolumeMode, PresetCatalogHandle};

impl PlaterSession {
    /// The installed system vendors, `PresetBundle::vendors`, which the vendor test reads
    /// (`get_current_vendor_type`, `PresetBundle.cpp:992-1024`). Until set, none is installed.
    pub fn set_installed_vendors(&mut self, vendors: &PresetCatalogHandle) {
        let printer_model = &self.project.settings().printer.remaining.printer_model.0;
        self.vendor_type = vendors.current_vendor_type(printer_model);
    }

    /// `set_arrange_polygon_bed_exclusion_extruders` (`:53-81`) for the object of `key` mapped on
    /// `plate`: empty in Shared mode, without nozzles, or for a Bambu printer whose plate map is
    /// automatic; else the extruders of the object's filaments.
    pub(super) fn set_bed_exclusion_extruders(
        &self,
        polygon: &mut ArrangePolygon,
        (key, plate): (InstanceKey, usize),
        config: &DynamicPrintConfig,
    ) {
        polygon.bed_exclusion_extruder_ids.clear();
        if active_bed_exclude_volume_mode(config) == BedExcludeVolumeMode::Shared {
            return;
        }
        let extruder_count = config.option("nozzle_diameter").floats().len();
        if extruder_count == 0 {
            return;
        }
        let is_bambu = self.vendor_type == VendorType::MarlinBbl;
        if is_bambu
            && is_auto_filament_map_mode(self.plates.real_filament_map_mode(plate, &self.project))
        {
            return;
        }
        let object = &self.project.objects()[key.0];
        let mapping = self.plates.object_filament_extruders(
            (plate, &self.project, object),
            config,
            (extruder_count, is_bambu),
        );
        let mut ids: Vec<i32> = mapping
            .into_values()
            .map(|extruder| extruder as i32)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        polygon.bed_exclusion_extruder_ids = ids;
    }

    /// `set_arrange_polygon_bed_exclusion_extruders_for_all_plates` (`:83-103`): the union over
    /// every plate, or empty as soon as one plate leaves the ids empty.
    pub(super) fn set_bed_exclusion_extruders_for_all_plates(
        &self,
        polygon: &mut ArrangePolygon,
        key: InstanceKey,
        config: &DynamicPrintConfig,
    ) {
        polygon.bed_exclusion_extruder_ids.clear();
        if active_bed_exclude_volume_mode(config) == BedExcludeVolumeMode::Shared {
            return;
        }
        let mut ids = std::collections::BTreeSet::new();
        for plate in 0..self.plates.plates.len() {
            let mut mapped = polygon.clone();
            self.set_bed_exclusion_extruders(&mut mapped, (key, plate), config);
            if mapped.bed_exclusion_extruder_ids.is_empty() {
                return;
            }
            ids.extend(mapped.bed_exclusion_extruder_ids);
        }
        polygon.bed_exclusion_extruder_ids = ids.into_iter().collect();
    }
}
