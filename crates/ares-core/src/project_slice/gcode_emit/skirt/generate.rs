use super::*;

impl SkirtPlan {
    pub(in crate::project_slice::gcode_emit) fn generate(
        traversal: &PreparedPostClassicTraversal,
        support: &[Vec<crate::project_slice::support_material::InstalledSupportLayer>],
        brim: &[Option<super::brim::BrimPlan>],
        tower_corners: &[Point],
        raft: Option<&super::layers::RaftSchedule>,
    ) -> Result<Option<Self>, SliceError> {
        let print = &traversal.resolved.views.full.process.print;
        let loops_count = usize::try_from(print.skirt_loops.0).unwrap_or(0);
        let infinite_skirt =
            print.draft_shield == crate::ProcessDraftShield::Enabled && loops_count > 0;
        let layer_limit = if infinite_skirt {
            usize::MAX
        } else {
            usize::try_from(print.skirt_height.0).unwrap_or(0)
        };
        if layer_limit == 0 {
            return Ok(None);
        }
        if loops_count > 0
            && print.skirt_type != crate::ProcessSkirtType::Combined
            && !(print.skirt_type == crate::ProcessSkirtType::PerObject
                && traversal.resolved.objects.len() == 1)
        {
            return Err(SliceError::UnsupportedProjectFeature(
                "skirt_type per-object".to_owned(),
            ));
        }
        let mut hulls = occupied::hulls(traversal, support, brim, layer_limit, raft);
        if !tower_corners.is_empty() {
            hulls.push(tower_corners.to_vec());
        }
        let hull = convex_hull(&hulls.iter().flatten().copied().collect::<Vec<_>>());
        if hull.len() < 3 {
            return Ok(None);
        }

        // `Print::skirt_flow` (`Print.cpp:2028-2049`).
        let full = &traversal.resolved.views.full;
        let object = traversal.resolved.objects.first();
        let initial_width = if print.initial_layer_line_width.is_non_positive() {
            object.map_or(full.process.object.line_width, |value| {
                value.object.line_width
            })
        } else {
            print.initial_layer_line_width
        };
        let nozzle = full
            .project
            .print
            .nozzle_diameter
            .0
            .first()
            .map_or(0.4, |value| value.0) as f32;
        let flow = build_nonbridging_flow(
            initial_width,
            print.initial_layer_print_height.0 as f32,
            nozzle,
        )?;

        // Loop centerlines (`Print.cpp:2718-2772`): start at
        // `skirt_distance - spacing/2`, step by spacing, inner-to-outer,
        // then reverse for emission.
        let scale = traversal.scale;
        let scaled_spacing = (f64::from(flow.spacing) / scale.factor()) as f32;
        let mut distance =
            ((print.skirt_distance.0 - f64::from(flow.spacing) * 0.5) / scale.factor()) as f32;
        let arc_tolerance = checked_scale(scale, 0.1)? as f64;
        let hull_polygon = Polygon::new(hull);
        let minimum_filament = print.min_skirt_length.0;
        let filament = &full.filament.gcode;
        let diameter = filament
            .filament_diameter
            .0
            .first()
            .map_or(1.75, |diameter| diameter.0);
        let flow_ratio = super::motion::first_nullable_float(&filament.filament_flow_ratio, 1.0);
        let e_per_path_mm =
            flow.mm3_per_mm * flow_ratio / (std::f64::consts::PI * diameter.powi(2) * 0.25);
        let mut extruded_filament = 0.0;
        let mut loops = Vec::with_capacity(loops_count);
        while loops_count > 0 && (loops.len() < loops_count || extruded_filament < minimum_filament)
        {
            distance += scaled_spacing;
            let offset = offset_paths(
                std::slice::from_ref(&hull_polygon),
                distance.abs(),
                JoinType::Round,
                arc_tolerance,
            )
            .map_err(|_| SliceError::InvalidInput("skirt offset failed".to_owned()))?;
            let Some(loop_polygon) = offset.first() else {
                break;
            };
            let simplified = simplify_closed_points(
                loop_polygon.points().to_vec(),
                checked_scale(scale, 0.05)? as f64,
            );
            if simplified.len() < 3 {
                break;
            }
            extruded_filament += closed_length(&simplified) * scale.factor() * e_per_path_mm;
            loops.push(simplified);
        }
        let groups = if loops_count == 0 && print.skirt_type == crate::ProcessSkirtType::PerObject {
            groups::merge(
                hulls,
                checked_scale(scale, print.skirt_distance.0)? as f32,
                arc_tolerance,
            )?
        } else {
            vec![hull_polygon]
        };
        let mut occupied_hull = Vec::new();
        for hull in groups {
            occupied_hull.extend(
                offset_paths(
                    std::slice::from_ref(&hull),
                    distance + 0.5_f32 * scaled_spacing,
                    JoinType::Round,
                    arc_tolerance,
                )
                .map_err(|_| {
                    SliceError::InvalidInput("skirt occupied hull offset failed".to_owned())
                })?,
            );
        }
        loops.reverse();

        let layer_count = if loops.is_empty() { 0 } else { layer_limit };
        let start_angle_deg = object.map_or(-135.0, |value| value.object.skirt_start_angle.0);
        Ok(Some(Self {
            loops,
            occupied_hull,
            width: flow.width,
            layer_count,
            start_angle_deg,
            single_loop_draft_shield: print.single_loop_draft_shield.0,
            plate_origin: traversal.project.plate_origin(),
        }))
    }

}
