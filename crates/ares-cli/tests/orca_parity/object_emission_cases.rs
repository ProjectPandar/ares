//! GCode object emission and writer-motion parity cases.

#[path = "bbl_layer_object_labels.rs"]
mod bbl_layer_object_labels;
#[path = "p1s_bylayer_brim.rs"]
mod p1s_bylayer_brim;
#[path = "print_order_as_obj_list.rs"]
mod print_order_as_obj_list;
#[path = "retract_internal_regions_routed.rs"]
mod retract_internal_regions_routed;

#[path = "deferred_lift_epsilon.rs"]
mod deferred_lift_epsilon;

#[path = "jerk_dedup_tolerance.rs"]
mod jerk_dedup_tolerance;

#[path = "pa_signed_zero.rs"]
mod pa_signed_zero;

#[path = "runner/custom_temperature_command_scan.rs"]
mod custom_temperature_command_scan;

#[path = "velocity_limit_comments.rs"]
mod velocity_limit_comments;

#[path = "merged_jerk_tolerance.rs"]
mod merged_jerk_tolerance;

#[path = "merged_jerk_precision.rs"]
mod merged_jerk_precision;

#[path = "accel_to_decel_precision.rs"]
mod accel_to_decel_precision;

#[path = "machine_jerk_rounding.rs"]
mod machine_jerk_rounding;

#[path = "repetier_jerk_precision.rs"]
mod repetier_jerk_precision;

#[path = "klipper_separate_jerk_precision.rs"]
mod klipper_separate_jerk_precision;

#[path = "klipper_separate_jerk_xy_clamp.rs"]
mod klipper_separate_jerk_xy_clamp;

#[path = "bbl_jerk_tail_precision.rs"]
mod bbl_jerk_tail_precision;

#[path = "machine_acceleration_cache_rounding.rs"]
mod machine_acceleration_cache_rounding;

#[path = "bbl_separate_jerk_tail_gate.rs"]
mod bbl_separate_jerk_tail_gate;
