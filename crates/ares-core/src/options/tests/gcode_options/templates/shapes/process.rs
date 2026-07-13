use super::*;

pub(super) fn verify_arrays() -> Vec<&'static str> {
    verify_array_fields! {
        process;
        small_area_infill_flow_compensation_model =>
            "small_area_infill_flow_compensation_model" =
            strings(&["1501,0", "1502|{opaque}", "1503\\tail\n"]),
    }
}
