//! Explicit outer/inner brim per PrintObject, rewriting the simple
//! `outer_inner_brim_area` + `makeBrimInfillImpl` path
//! (`Brim.cpp:421-570,819-878`).

use crate::{
    ProcessBrimType, SliceError,
    project_slice::perimeters::{
        classic::traversal::PreparedPostClassicTraversal, flow::build_nonbridging_flow,
    },
    project_slice::support_material::InstalledSupportLayer,
};
impl BrimPlan {
    pub(super) fn generate_all<'a>(
        traversal: &PreparedPostClassicTraversal,
        support: &[Vec<InstalledSupportLayer>],
        tower_exclusions: &[crate::geometry::ExPolygon],
        first_layer_regions: impl Fn(
            usize,
        )
            -> &'a [crate::project_slice::extrusion_islands::RegionToolUsage],
    ) -> Result<Vec<Option<Self>>, SliceError> {
        let mut areas = (0..traversal.objects.len())
            .map(|_| None)
            .collect::<Vec<_>>();
        let mut no_brim_area = Vec::new();
        // Brim.cpp:576,662-679: collect every owner's no-brim geometry
        // before subtracting it from any owner, independently of brim ownership.
        for index in owner::generation_order(traversal) {
            if let Some((area, forbidden)) =
                Self::prepare_area(traversal, support, first_layer_regions(index), index)?
            {
                no_brim_area.extend(forbidden);
                areas[index] = Some(area);
            }
        }
        // Brim.cpp:718-722,726-766 then :953-958: the masks/retention run
        // per entry, then the ordered cumulative claimed-area subtraction
        // iterates the map in key order (ObjectInstanceID{ObjectID,
        // instance_idx} ascending) — earlier keys keep overlapping area —
        // before the per-entry infill (:1004-1010).
        let mut claimed = Vec::new();
        // Brim.cpp:953-958: the ordered cumulative subtraction runs only
        // when brims are not combined; under combine_brims the map keeps
        // whole areas and the group assembly unions contacting clusters.
        let combine = traversal.resolved.views.full.process.print.combine_brims.0;
        for area in areas.iter_mut() {
            let Some(area) = area else {
                continue;
            };
            if combine {
                continue;
            }
            area.mask_entries(traversal, &no_brim_area, tower_exclusions)?;
            for entry in &mut area.entries {
                let assigned = crate::geometry::difference_ex(&entry.area, &claimed)
                    .map_err(brim_geometry_error)?;
                claimed.extend(assigned.iter().cloned());
                entry.area = assigned;
            }
        }
        areas
            .into_iter()
            .enumerate()
            .map(|(index, area)| match area {
                Some(area) => area.finish(traversal, index, &no_brim_area, tower_exclusions),
                None => Ok(None),
            })
            .collect()
    }

    fn prepare_area(
        traversal: &PreparedPostClassicTraversal,
        support: &[Vec<InstalledSupportLayer>],
        first_layer_regions: &[crate::project_slice::extrusion_islands::RegionToolUsage],
        object_index: usize,
    ) -> Result<Option<(owner::Area, Vec<crate::geometry::ExPolygon>)>, SliceError> {
        let (resolved, _) = traversal.print_object_config(object_index);
        let options = &resolved.object;
        if !matches!(
            options.brim_type,
            ProcessBrimType::AutoBrim
                | ProcessBrimType::OuterOnly
                | ProcessBrimType::InnerOnly
                | ProcessBrimType::OuterAndInner
                | ProcessBrimType::BrimEars
                | ProcessBrimType::Painted
        ) || (options.brim_type != ProcessBrimType::AutoBrim && options.brim_width.0 <= 0.0)
        {
            return Ok(None);
        }
        let source = traversal
            .objects
            .get(object_index)
            .and_then(|object| {
                object
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object
                    .object
                    .as_parts()
                    .1
                    .first()
            })
            .cloned()
            .unwrap_or_default();
        if source.is_empty() {
            return Ok(None);
        }
        let raw = source.clone();
        let volume_slices = traversal.objects[object_index]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .object
            .volume_slices()
            .to_vec();
        let grouping_resolution = scaled_f64(
            traversal.scale,
            traversal.resolved.views.full.process.print.resolution.0,
        )?;
        let offset_value = scaled_f64(traversal.scale, 0.05)?;
        // libslic3r.h:76-77; Brim.cpp:534,543,569,597 use fixed
        // SCALED_RESOLUTION for brim offsets.
        let resolution = scaled_f64(traversal.scale, 0.0125)?;
        let volume_groups =
            group_volume_slices(volume_slices, &raw, grouping_resolution, offset_value)?;
        // Brim.cpp:364-367/:468-472: the EFC-adjusted bottom outline is
        // computed once under the gate and shared by the base-slices swap
        // and the painted-ear snap.
        let efc_bottom = if outline::efc_enabled(options) {
            Some(outline::bottom_regions(
                &traversal.objects[object_index]
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object
                    .object,
                traversal.scale,
            )?)
        } else {
            None
        };
        let source = outline::select(raw.clone(), efc_bottom.as_deref())?;
        let full = &traversal.resolved.views.full;
        let print = &full.process.print;
        // Print::brim_flow (Print.cpp:2007-2025) selects Print-wide
        // defaults, independently of the object whose brim is generated.
        let region = traversal
            .resolved
            .objects
            .iter()
            .flat_map(|object| &object.all_regions)
            .next()
            .expect("Print retains its first registered region");
        let (first_object, _) = traversal.print_object_config(0);
        let width = if print.initial_layer_line_width.is_non_positive() {
            if region.inner_wall_line_width.is_non_positive() {
                first_object.object.line_width
            } else {
                region.inner_wall_line_width
            }
        } else {
            print.initial_layer_line_width
        };
        let nozzles = &full.project.print.nozzle_diameter.0;
        let nozzle = nozzles
            .get(region.outer_wall_filament_id.0 as usize - 1)
            .unwrap_or(&nozzles[0])
            .0 as f32;
        let flow =
            build_nonbridging_flow(width, print.initial_layer_print_height.0 as f32, nozzle)?;
        let scale = traversal.scale;
        // Flow.hpp:71, Brim.cpp:496–498: truncate spacing once. Derive the
        // double millimeters and float scaled spacing from that integer.
        let spacing_units = geometry::scaled_coord(scale, f64::from(flow.spacing))?;
        let spacing_mm = scale.unscale(spacing_units);
        let brim_width_mm = geometry::even_width_mm(options.brim_width.0, spacing_mm);
        if brim_width_mm <= 0.0 && options.brim_type != ProcessBrimType::AutoBrim {
            return Ok(None);
        }
        let gap = scaled_f32(scale, options.brim_object_gap.0)?;
        // Brim.cpp:497, libslic3r.h:106: double scale_ directly to float.
        let brim_width = scale.scaled_delta(brim_width_mm) as f32;
        let spacing = spacing_units as f32;
        let mut object_islands = Vec::new();
        // Brim.cpp:590-634 via :576/:682-689: support contours join the
        // no-brim mask for every brim type with a ring on the first layer
        // (outer ring types, InnerOnly's in-hole ring, and the ears ring;
        // tree: first-layer slice contours; inner-normal: fill spacing
        // coverage).
        let support_mask = if matches!(
            options.brim_type,
            ProcessBrimType::OuterOnly
                | ProcessBrimType::OuterAndInner
                | ProcessBrimType::InnerOnly
                | ProcessBrimType::BrimEars
                | ProcessBrimType::Painted
        ) {
            match support.get(object_index).and_then(|layers| layers.first()) {
                Some(first) => super::footprint::support_no_brim(first.inner_tree, first, scale)
                    .map_err(brim_geometry_error)?,
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let (brim_area, inner) = if options.brim_type == ProcessBrimType::InnerOnly {
            let area = crate::brims::inner::area(&source, gap, brim_width, spacing, resolution)?;
            object_islands = area.object_islands;
            // :558-560 adds no hole-mask for InnerOnly; the support
            // no-brim mask is the separate :590-634 source.
            let mut forbidden = area.forbidden;
            forbidden.extend(support_mask);
            (area.area, forbidden)
        } else if options.brim_type == ProcessBrimType::OuterAndInner {
            let outer = geometry::outer_width(brim_width, spacing);
            let area = crate::brims::combined::area(&source, gap, (outer, brim_width), resolution)?;
            object_islands = area.object_islands;
            // Both predicates are true: only objectIsland is forbidden,
            // without the no-inner/no-outer masks (Brim.cpp:560-570); the
            // support no-brim mask is a separate :590-634 source.
            let mut forbidden = object_islands.clone();
            forbidden.extend(support_mask);
            (area.area, forbidden)
        } else if options.brim_type == ProcessBrimType::Painted {
            // Brim.cpp:456-458/:537: the painted call site replaces the
            // auto ears with per-point discs (make_brim_ears :354-412;
            // empty point set keeps the plan empty, :359-361).
            let points = owner::painted_points(traversal, object_index);
            if points.is_empty() {
                return Ok(None);
            }
            // Brim.cpp:363-367: raw lslices for island matching and the
            // lazily-built EFC outline (:112-118) for the painted-ear
            // snap (:392-411); :468-472 already swapped `source` to the
            // EFC outline via outline::select.
            let raw_outline = raw.clone();
            let efc_outline = efc_bottom.clone();
            let area = crate::brims::ears::painted(
                &source,
                crate::brims::ears::PaintedParameters {
                    gap,
                    width: brim_width,
                    resolution,
                    scale,
                    points,
                    raw_outline,
                    efc_outline,
                },
            )?;
            object_islands = area.1.clone();
            let mut forbidden = area.1;
            forbidden.extend(support_mask);
            (area.0, forbidden)
        } else if options.brim_type == ProcessBrimType::BrimEars {
            let (area, mut forbidden) = crate::brims::ears::area(
                &source,
                crate::brims::ears::AreaParameters {
                    gap,
                    width: brim_width,
                    spacing,
                    resolution,
                    detection_length: scale
                        .checked_scale(options.brim_ears_detection_length.0)
                        .expect("validated brim detection length"),
                    max_angle: options.brim_ears_max_angle.0,
                },
            )?;
            forbidden.extend(support_mask);
            (area, forbidden)
        } else if options.brim_type == ProcessBrimType::AutoBrim {
            // Brim.cpp:565–576 applies brim_width_mod independently to each
            // ExPolygon before make_brim, so islands do not share a clamp.
            let mut area = Vec::new();
            let mut forbidden = Vec::new();
            let volume_materials = volume_materials(
                traversal,
                object_index,
                &full.filament.gcode.filament_type.0,
            );
            let auto_context = AutoGroupContext {
                traversal,
                object_index,
                max_speed: speed::find_max_speed(traversal, object_index),
                adhesion: groups::object_adhesion(
                    traversal,
                    object_index,
                    first_layer_regions,
                    &volume_materials,
                ),
                spacing_mm,
                scale,
                gap,
                spacing,
                resolution,
                volume_materials: &volume_materials,
                efc_bottom: efc_bottom.as_deref(),
            };
            for group in &volume_groups {
                let (group_area, group_forbidden, group_islands) =
                    auto_group_area(&auto_context, group)?;
                area.extend(group_area);
                forbidden.extend(group_forbidden);
                object_islands.extend(group_islands);
            }
            (area, forbidden)
        } else {
            let outer_width = geometry::outer_width(brim_width, spacing);
            let area = crate::brims::outer::area(&source, gap, outer_width, spacing, resolution)?;
            object_islands = area.object_islands;
            let mut forbidden = area.forbidden;
            forbidden.extend(support_mask);
            (area.area, forbidden)
        };
        // Print.cpp:4188-4193, GCode.cpp:5991-6082: generation is
        // per-instance for every print sequence — the by-object emission
        // consumes the same instance-keyed map.
        let (entries, forbidden) = owner::append(
            traversal,
            object_index,
            (&brim_area, &inner, &object_islands),
        )?;
        Ok(Some((
            owner::Area {
                entries,
                flow,
                spacing,
                resolution: grouping_resolution,
                logical_filament: region.outer_wall_filament_id.0 as usize - 1,
                retain_contact: matches!(
                    options.brim_type,
                    ProcessBrimType::OuterOnly
                        | ProcessBrimType::InnerOnly
                        | ProcessBrimType::OuterAndInner
                ),
            },
            forbidden,
        )))
    }
}

use auto::{AutoGroupContext, auto_group_area};
pub(super) use geometry::{geometry_error as brim_geometry_error, scaled_f32, scaled_f64};

mod geometry;
mod groups;
mod types;
use groups::{group_volume_slices, volume_materials};
pub(super) use types::BrimPlan;
mod auto;
mod bed_area;
mod combined;
mod speed;
pub(super) use combined::{GroupBrim, group_brims};
mod emit;
mod outline;
mod owner;
