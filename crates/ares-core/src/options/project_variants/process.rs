//! PrintApply.cpp:1429; PrintConfig.cpp:11774–11851: selected process vectors.
use super::select::select_stride;
use crate::{ProjectSettings, SliceError};

pub(super) fn materialize(
    settings: &mut ProjectSettings,
    indices: &[usize],
) -> Result<(), SliceError> {
    macro_rules! fields {
        ($group:ident: $($field:ident),+ $(,)?) => {$(
            settings.process.$group.$field.0 = select_stride(
                &settings.process.$group.$field.0, indices, 1, stringify!($field),
            )?;
        )+};
    }
    fields!(object:
        default_acceleration,
        outer_wall_acceleration,
        inner_wall_acceleration,
        top_surface_acceleration,
        initial_layer_acceleration,
        bridge_acceleration,
        travel_acceleration,
        sparse_infill_acceleration,
        internal_solid_infill_acceleration,
        default_junction_deviation,
        default_jerk,
        outer_wall_jerk,
        inner_wall_jerk,
        infill_jerk,
        top_surface_jerk,
        initial_layer_jerk,
        travel_jerk,
        support_speed,
        support_interface_speed,
    );
    fields!(gcode:
        initial_layer_travel_acceleration,
        initial_layer_travel_jerk,
        travel_speed,
        travel_speed_z,
        initial_layer_travel_speed,
    );
    fields!(print:
        initial_layer_speed,
        initial_layer_infill_speed,
    );
    fields!(region:
        inner_wall_speed,
        outer_wall_speed,
        enable_overhang_speed,
        slowdown_for_curled_perimeters,
        print_extruder_id,
        print_extruder_variant,
        bridge_speed,
        gap_infill_speed,
        sparse_infill_speed,
        internal_solid_infill_speed,
        top_surface_speed,
        top_solid_infill_flow_ratio,
        internal_bridge_speed,
        overhang_1_4_speed,
        overhang_2_4_speed,
        overhang_3_4_speed,
        overhang_4_4_speed,
        small_perimeter_speed,
        small_perimeter_threshold,
    );
    Ok(())
}
