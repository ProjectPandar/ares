pub(super) fn uses_sloped_lift(z_hop_type: crate::ZHopType) -> bool {
    z_hop_type != crate::ZHopType::Normal
}

pub(in crate::project_slice::gcode_emit::motion) fn can_skip_retraction(
    reduce_infill_retraction: bool,
    has_sparse_infill: bool,
    previous_feature: Option<&str>,
    current_is_perimeter: bool,
    inside_internal_surface: bool,
) -> bool {
    reduce_infill_retraction
        && has_sparse_infill
        && !matches!(previous_feature, Some("Outer wall" | "Overhang wall"))
        && !current_is_perimeter
        && inside_internal_surface
    // `GCode::needs_retraction` also exempts support-role travels fully
    // inside `support_layer->support_islands` (`GCode.cpp:7618-7632`).
    // Ares does not generate `SupportLayer` islands yet (raft layers
    // leave them empty upstream too, `SupportCommon.cpp:1439-1441`), so
    // the exemption is structurally unreachable here and returns false
    // exactly as upstream does without a support layer.
}
