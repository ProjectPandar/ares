use super::loop_generation::{LoopParameters, make_loops};
use super::*;

impl SkirtPlan {
    pub(in crate::project_slice::gcode_emit) fn generate(
        traversal: &PreparedPostClassicTraversal,
        support: &[Vec<crate::project_slice::support_material::InstalledSupportLayer>],
        brim: &[Option<super::super::brim::BrimPlan>],
        tower_corners: &[Point],
        raft: Option<&super::super::layers::RaftSchedule>,
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
        let per_object = loops_count > 0 && print.skirt_type == crate::ProcessSkirtType::PerObject;
        // The contact-component slice admits only the frozen ByLayer,
        // single-instance explicit-outer envelope; other combined scopes
        // retain the existing rejection below.
        let grouped_combining = print.print_sequence == crate::ProcessPrintSequence::ByLayer
            && print.draft_shield == crate::ProcessDraftShield::Disabled
            && tower_corners.is_empty()
            && raft.is_none()
            && support.iter().all(Vec::is_empty)
            && traversal
                .resolved
                .views
                .full
                .filament
                .gcode
                .filament_diameter
                .0
                .len()
                == 1
            && (0..traversal.objects.len()).all(|index| {
                let (object, group) = traversal.print_object_config(index);
                object.object.brim_type == crate::ProcessBrimType::OuterOnly
                    && group.instance_indices.len() == 1
            });
        if per_object
            && traversal.resolved.objects.len() > 1
            && brim.iter().enumerate().any(|(index, plan)| {
                let (object, _) = traversal.print_object_config(index);
                plan.is_some()
                    && ((print.combine_brims.0 && !grouped_combining)
                        // `Print.cpp:2847-2971` / `Brim.cpp:448-460,964-977`:
                        // admitted brim kinds retain their object's occupied footprint.
                        || !matches!(
                            object.object.brim_type,
                            crate::ProcessBrimType::AutoBrim
                                | crate::ProcessBrimType::OuterOnly
                                | crate::ProcessBrimType::InnerOnly
                                | crate::ProcessBrimType::Painted
                                | crate::ProcessBrimType::BrimEars
                                | crate::ProcessBrimType::OuterAndInner
                        ))
            })
        {
            return Err(SliceError::UnsupportedProjectFeature(
                "skirt_type per-object".to_owned(),
            ));
        }
        let items = occupied::items(traversal, support, brim, layer_limit, raft);
        let mut hulls = items
            .iter()
            .map(|(_, hull)| hull.clone())
            .collect::<Vec<_>>();
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
        let initial_distance =
            ((print.skirt_distance.0 - f64::from(flow.spacing) * 0.5) / scale.factor()) as f32;
        let arc_tolerance = checked_scale(scale, 0.1)? as f64;
        let hull_polygon = Polygon::new(hull);
        let minimum_filament = print.min_skirt_length.0;
        let filament = &full.filament.gcode;
        // Print.cpp:2710-2719 uses Print::extruders(false), including
        // configured region tools even when they produce no extrusion.
        let e_per_path_mm = super::super::tool_ordering::print_extruders(traversal)
            .into_iter()
            .map(|id| {
                let diameter = filament
                    .filament_diameter
                    .0
                    .get(id)
                    .unwrap_or(&filament.filament_diameter.0[0])
                    .0;
                let ratio = super::motion::first_nullable_float(
                    std::slice::from_ref(
                        filament
                            .filament_flow_ratio
                            .get(id)
                            .unwrap_or(&filament.filament_flow_ratio[0]),
                    ),
                    1.0,
                );
                flow.mm3_per_mm * ratio / (std::f64::consts::PI * diameter.powi(2) * 0.25)
            })
            .collect::<Vec<_>>();
        let generation_groups = if per_object && traversal.objects.len() > 1 {
            member_groups::merge(
                items,
                if tower_corners.is_empty() {
                    None
                } else {
                    Some(tower_corners)
                },
                checked_scale(
                    scale,
                    print.skirt_distance.0 + loops_count as f64 * f64::from(flow.spacing),
                )? as f32,
                arc_tolerance,
            )?
        } else {
            vec![member_groups::Group {
                members: Vec::new(),
                hull: hull_polygon.clone(),
            }]
        };
        // `Print.cpp:2551-2556`: ByObject shared per-object skirts throw SlicingError.
        // Keep this unsupported scope guarded; only separate object groups are admitted.
        if per_object
            && print.print_sequence == crate::ProcessPrintSequence::ByObject
            && generation_groups
                .iter()
                .any(|group| group.members.len() > 1)
        {
            return Err(SliceError::UnsupportedProjectFeature(
                "skirt_type per-object".to_owned(),
            ));
        }
        let mut loops = Vec::new();
        let mut member_groups = Vec::new();
        let mut occupied_hull = Vec::new();
        for group in generation_groups {
            let mut distance = initial_distance;
            let start = loops.len();
            let mut group_loops = make_loops(
                &group.hull,
                &mut distance,
                LoopParameters {
                    loops_count,
                    scaled_spacing,
                    arc_tolerance,
                    minimum_filament,
                    e_per_path_mm: &e_per_path_mm,
                    scale,
                },
            )?;
            group_loops.reverse();
            loops.extend(group_loops);
            if !group.members.is_empty() {
                member_groups.push(SkirtGroup {
                    brims: super::super::brim::group_brims(traversal, &group.members, brim)?,
                    members: group.members,
                    loops: start..loops.len(),
                });
            }
            let occupied_groups =
                if loops_count == 0 && print.skirt_type == crate::ProcessSkirtType::PerObject {
                    groups::merge(
                        hulls.clone(),
                        checked_scale(scale, print.skirt_distance.0)? as f32,
                        arc_tolerance,
                    )?
                } else {
                    vec![group.hull]
                };
            for hull in occupied_groups {
                occupied_hull.extend(
                    offset_paths(
                        std::slice::from_ref(&hull),
                        distance + 0.5_f32 * scaled_spacing,
                        JoinType::Round,
                        arc_tolerance,
                    )
                    .map_err(occupied_error)?,
                );
            }
        }

        let layer_count = if loops.is_empty() { 0 } else { layer_limit };
        let start_angle_deg = object.map_or(-135.0, |value| value.object.skirt_start_angle.0);
        Ok(Some(Self {
            loops,
            member_groups,
            occupied_hull,
            width: flow.width,
            layer_count,
            start_angle_deg,
            single_loop_draft_shield: print.single_loop_draft_shield.0,
            plate_origin: traversal.project.plate_origin(),
        }))
    }
}

fn occupied_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("skirt occupied hull offset failed".into())
}
