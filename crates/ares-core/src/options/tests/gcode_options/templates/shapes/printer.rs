use super::*;

pub(super) fn verify_arrays() -> Vec<&'static str> {
    verify_array_fields! {
        printer;
        physical_extruder_map => "physical_extruder_map" = OrcaInts(Vec::new()),
        nozzle_flush_dataset => "nozzle_flush_dataset" = NullableInts(nullable_ints(&[
            None,
            Some(1101),
            None,
            Some(1103),
        ])),
        wrapping_exclude_area => "wrapping_exclude_area" =
            Point2dList(vec![Point2d::new(12.5, -7.25)]),
        retraction_distances_when_cut => "retraction_distances_when_cut" =
            floats(&[1201.1, 1202.2, 1203.3, 1204.4]),
        long_retractions_when_cut => "long_retractions_when_cut" =
            bools(&[true, false, false, true]),
        z_hop_types => "z_hop_types" = ZHopTypes(vec![
            ZHopType::Auto,
            ZHopType::Normal,
            ZHopType::Slope,
            ZHopType::Spiral,
        ]),
        travel_slope => "travel_slope" = floats(&[13.1, 13.2, 13.3, 13.4]),
        retract_lift_enforce => "retract_lift_enforce" = RetractLiftEnforces(vec![
            RetractLiftEnforce::AllSurfaces,
            RetractLiftEnforce::TopOnly,
            RetractLiftEnforce::BottomOnly,
            RetractLiftEnforce::TopAndBottom,
        ]),
        nozzle_type => "nozzle_type" = NullableNozzleTypes(vec![
            Nullable::Nil,
            Nullable::Value(NozzleType::Brass),
            Nullable::Value(NozzleType::HardenedSteel),
            Nullable::Nil,
        ]),
        extruder_type => "extruder_type" = ExtruderTypes(vec![
            ExtruderType::DirectDrive,
            ExtruderType::Bowden,
            ExtruderType::DirectDrive,
        ]),
        printer_extruder_id => "printer_extruder_id" = ints(&[1401, 1402, 1403, 1404]),
        printer_extruder_variant => "printer_extruder_variant" =
            strings(&["variant-0", "variant-1", "variant-2", "variant-3"]),
    }
}
