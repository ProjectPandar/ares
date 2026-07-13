use super::*;

pub(super) fn verify_arrays() -> Vec<&'static str> {
    verify_array_fields! {
        project;
        deretraction_speed => "deretraction_speed" = floats(&[1601.1, 1602.2, 1603.3, 1604.4]),
        filament_ids => "filament_ids" = strings(&["project-singleton"]),
        filament_map => "filament_map" = ints(&[1611, 1612, 1613]),
        retract_before_wipe => "retract_before_wipe" = percents(&[17.1, 17.2, 17.3, 17.4]),
        retraction_length => "retraction_length" = floats(&[1801.1, 1802.2, 1803.3, 1804.4]),
        retract_length_toolchange => "retract_length_toolchange" =
            floats(&[1811.1, 1812.2, 1813.3, 1814.4]),
        z_hop => "z_hop" = floats(&[1821.1, 1822.2, 1823.3, 1824.4]),
        retract_lift_above => "retract_lift_above" = floats(&[1831.1, 1832.2, 1833.3, 1834.4]),
        retract_lift_below => "retract_lift_below" = floats(&[1841.1, 1842.2, 1843.3, 1844.4]),
        retract_restart_extra => "retract_restart_extra" = floats(&[1851.1, 1852.2, 1853.3, 1854.4]),
        retract_restart_extra_toolchange => "retract_restart_extra_toolchange" =
            floats(&[1861.1, 1862.2, 1863.3, 1864.4]),
        retraction_speed => "retraction_speed" = floats(&[1871.1, 1872.2, 1873.3, 1874.4]),
        nozzle_volume_type => "nozzle_volume_type" = NozzleVolumeTypes(Vec::new()),
        extruder_ams_count => "extruder_ams_count" =
            AmsCounts(owned_strings(&["ams-alpha", "ams-beta"])),
    }
}
