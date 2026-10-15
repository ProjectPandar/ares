//! Explicit outer/inner brim per PrintObject, rewriting the simple
//! `outer_inner_brim_area` + `makeBrimInfillImpl` path
//! (`Brim.cpp:421-570,819-878`).

use crate::{
    ProcessBrimType, SliceError,
    geometry::{CoordinateScale, Point},
    project_slice::perimeters::{
        classic::traversal::PreparedPostClassicTraversal, flow::build_nonbridging_flow,
    },
    project_slice::support_material::InstalledSupportLayer,
};

pub(super) struct BrimPlan {
    pub(super) paths: Vec<Vec<Point>>,
    pub(super) covered_hull: Vec<Point>,
    pub(super) width: f32,
    pub(super) spacing: f32,
    pub(super) height: f32,
    pub(super) mm3_per_mm: f64,
}

fn painted_points(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
) -> Vec<([f32; 2], f32)> {
    // Brim.cpp:359-361: no painted points means no painted brim at all.
    let source_index = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity()
        .0;
    traversal.project.objects()[source_index].brim_ear_centers()
}

impl BrimPlan {
    pub(super) fn generate(
        traversal: &PreparedPostClassicTraversal,
        support: &[Vec<InstalledSupportLayer>],
        object_index: usize,
    ) -> Result<Option<Self>, SliceError> {
        let (resolved, _) = traversal.print_object_config(object_index);
        let options = &resolved.object;
        if !matches!(
            options.brim_type,
            ProcessBrimType::OuterOnly
                | ProcessBrimType::InnerOnly
                | ProcessBrimType::OuterAndInner
                | ProcessBrimType::BrimEars
                | ProcessBrimType::Painted
        ) || options.brim_width.0 <= 0.0
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
        let source = outline::select(
            &traversal.objects[object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .object,
            options,
            traversal.scale,
            source,
        )?;
        let full = &traversal.resolved.views.full;
        let print = &full.process.print;
        let region = &resolved
            .layer_candidates
            .first()
            .and_then(|layer| layer.model_parts.first())
            .expect("brim object has a resolved model-part region")
            .region;
        let width = if print.initial_layer_line_width.is_non_positive() {
            if region.inner_wall_line_width.is_non_positive() {
                options.line_width
            } else {
                region.inner_wall_line_width
            }
        } else {
            print.initial_layer_line_width
        };
        let nozzle = full
            .project
            .print
            .nozzle_diameter
            .0
            .first()
            .map_or(0.4, |diameter| diameter.0) as f32;
        let flow =
            build_nonbridging_flow(width, print.initial_layer_print_height.0 as f32, nozzle)?;
        let scale = traversal.scale;
        // Upstream quantizes the flow width through the scaled lattice:
        // `flowWidth = print.brim_flow().scaled_spacing() * SCALING_FACTOR`
        // (Brim.cpp:463) — an integer round-trip, not the f32 spacing
        // value directly.
        let spacing_mm = f64::from(scaled_f32(scale, f64::from(flow.spacing))?) * scale.factor();
        let brim_width_mm = (options.brim_width.0 / spacing_mm / 2.0).floor() * spacing_mm * 2.0;
        if brim_width_mm <= 0.0 {
            return Ok(None);
        }
        let gap = scaled_f32(scale, options.brim_object_gap.0)?;
        let brim_width = scaled_f32(scale, brim_width_mm)?;
        let spacing = scaled_f32(scale, spacing_mm)?;
        let resolution = scaled_f64(scale, 0.0125)?;
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
            let area = crate::brims::combined::area(&source, gap, brim_width, spacing, resolution)?;
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
            let points = painted_points(traversal, object_index);
            if points.is_empty() {
                return Ok(None);
            }
            let area = crate::brims::ears::painted(
                &source,
                crate::brims::ears::PaintedParameters {
                    gap,
                    width: brim_width,
                    spacing,
                    resolution,
                    flow_width: spacing_mm,
                    scale,
                    points,
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
        } else {
            let area = crate::brims::outer::area(&source, gap, brim_width, spacing, resolution)?;
            object_islands = area.object_islands;
            let mut forbidden = area.forbidden;
            forbidden.extend(support_mask);
            (area.area, forbidden)
        };
        let brim_area = bed_area::clip(
            traversal,
            object_index,
            region.outer_wall_filament_id.0 as usize - 1,
            (brim_area, inner),
            resolution,
        )?;
        let brim_area = if matches!(
            options.brim_type,
            ProcessBrimType::OuterOnly
                | ProcessBrimType::InnerOnly
                | ProcessBrimType::OuterAndInner
        ) {
            crate::brims::inner::retain_contact(brim_area, &object_islands, spacing, resolution)?
        } else {
            brim_area
        };
        if brim_area.is_empty() {
            return Ok(None);
        }
        let covered_hull = crate::geometry::convex_hull(
            &brim_area
                .iter()
                .flat_map(|expolygon| expolygon.contour().points().iter().copied())
                .collect::<Vec<_>>(),
        );

        let paths = crate::brims::ears::paths::from_area(brim_area, spacing, resolution)?;
        if paths.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            paths,
            covered_hull,
            width: flow.width,
            spacing: flow.spacing,
            height: flow.height,
            mm3_per_mm: flow.mm3_per_mm * options.brim_flow_ratio.0,
        }))
    }
}

pub(super) fn scaled_f32(scale: CoordinateScale, value: f64) -> Result<f32, SliceError> {
    scale
        .checked_scale(value)
        .map(|value| value as f32)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".to_owned()))
}

pub(super) fn scaled_f64(scale: CoordinateScale, value: f64) -> Result<f64, SliceError> {
    scale
        .checked_scale(value)
        .map(|value| value as f64)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".to_owned()))
}

pub(super) fn brim_geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".to_owned())
}

mod bed_area;
mod emit;
mod outline;

#[cfg(test)]
mod tests;
