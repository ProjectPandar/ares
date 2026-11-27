//! `BeltBrim.cpp::make_belt_brim` for an admitted single belt object.

mod bands;
mod geometry;

use self::geometry::Frame;
use super::floor_context::BeltFloorContext;
use crate::{
    ObjectOptions, ProcessBrimType, ProjectSettings, SliceError,
    geometry::{
        BoundingBox, CoordinateScale, ExPolygon, Polyline, intersection_ex_polygons,
        union_expolygons,
    },
    project_slice::{
        belt::EPSILON,
        perimeters::{
            classic::traversal::PreparedPostClassicTraversal, flow::build_nonbridging_flow,
        },
    },
};

pub(in crate::project_slice) struct Band {
    pub(in crate::project_slice) print_z: f64,
    pub(in crate::project_slice) height: f64,
    pub(in crate::project_slice) paths: Vec<BandPath>,
    pub(in crate::project_slice) areas: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct BandPath {
    pub(in crate::project_slice) polyline: Polyline,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
}

pub(in crate::project_slice) struct BeltBrimPlan {
    pub(in crate::project_slice) by_layer: Vec<Band>,
    pub(in crate::project_slice) prologue: Vec<Band>,
}

impl BeltBrimPlan {
    pub(in crate::project_slice) fn generate(
        traversal: &PreparedPostClassicTraversal,
    ) -> Result<Option<Self>, SliceError> {
        if traversal.objects.len() != 1
            || !traversal
                .resolved
                .views
                .full
                .printer
                .remaining
                .belt_printer
                .0
        {
            return Ok(None);
        }
        generate(traversal)
            .map_err(|_| SliceError::InvalidInput("invalid belt brim geometry".into()))
    }
}

pub(super) fn enabled(settings: &ProjectSettings, options: &ObjectOptions) -> bool {
    let angle = settings.printer.remaining.belt_slice_rotation_angle.0;
    let leading = settings.process.object.leading_brim_length.0;
    let lateral = settings.process.object.extra_brim_width.0;
    // PrintObject::has_belt_brim excludes the marked mesh independently of purge enable.
    (1.0..=85.0).contains(&angle.abs())
        && !options.belt_purge_tower_object.0
        && options.brim_type != ProcessBrimType::NoBrim
        && (options.brim_width.0 > 0.0
            || (options.brim_type != ProcessBrimType::InnerOnly
                && (leading > 0.0 || lateral > 0.0)))
}

fn generate(
    traversal: &PreparedPostClassicTraversal,
) -> Result<Option<BeltBrimPlan>, crate::geometry::ClipperError> {
    let full = &traversal.resolved.views.full;
    let printer = &full.printer.remaining;
    let object = &traversal.objects[0];
    let options = &traversal.print_object_config(0).0.object;
    let leading = full.process.object.leading_brim_length.0;
    let lateral = full.process.object.extra_brim_width.0;
    if !enabled(full, options) {
        return Ok(None);
    }
    let Some(floor) = object.belt_floor() else {
        return Ok(None);
    };
    let floor_context = BeltFloorContext::new(floor, printer.belt_support_floor_offset.0);
    let frame = Frame {
        axis: floor.axis,
        shear: floor.shear,
    };
    let scale = traversal.scale;
    let scaled = |mm| scale.scaled_delta(mm) as i64;
    let layers = object.physical_layers();
    let mut contact = Vec::new();
    let mut first_contact = None;
    let mut wall_width = vec![None; layers.len()];
    for (i, layer) in layers.iter().enumerate() {
        let Some(slices) = object.slices(i) else {
            continue;
        };
        let Some(bounds) = BoundingBox::from_expolygons(slices) else {
            continue;
        };
        let (u0, u1) = cutoff_band(floor_context, (layer.print_z, layer.height));
        let Some(band) = frame.band_box(bounds, (scaled(u0), scaled(u1)), scaled(1.0)) else {
            continue;
        };
        let intersection = intersection_ex_polygons(slices, std::slice::from_ref(&band))?;
        if !intersection.is_empty() {
            first_contact.get_or_insert(layer.print_z);
            contact.extend(intersection);
            // Admission allows only one model volume and one print region.
            wall_width[i] = object.records[i]
                .as_ref()
                .map(|r| f64::from(r.ext_perimeter_flow.width));
        }
    }
    let footprint = union_expolygons(&contact, &[])?;
    let Some(footprint_box) = BoundingBox::from_expolygons(&footprint) else {
        return Ok(None);
    };
    let print = &full.process.print;
    let region = &traversal.print_object_config(0).0.all_regions[0];
    let mut width = print.initial_layer_line_width;
    if width.is_non_positive() {
        width = region.inner_wall_line_width;
    }
    if width.is_non_positive() {
        width = options.line_width;
    }
    let flow = build_nonbridging_flow(
        width,
        print.initial_layer_print_height.0 as f32,
        full.project.print.nozzle_diameter.0[0].0 as f32,
    )
    .expect("validated initial layer flow");
    let spacing = scale.unscale(flow.scaled_spacing(scale));
    let mut region = geometry::region(
        frame,
        footprint,
        options.brim_type,
        (
            scaled((options.brim_width.0 / spacing / 2.0).floor() * spacing * 2.0),
            scaled(options.brim_object_gap.0),
            scaled(leading),
            scaled(lateral),
        ),
        scale.scaled_delta(0.0125),
    )?;
    if options.brim_type == ProcessBrimType::LeadingEdgeOnly
        && let Some(bounds) = BoundingBox::from_expolygons(&region)
        && let Some(z) = first_contact
    {
        let u = scaled(floor_context.cutoff_u(z));
        let margin = scaled(1.0);
        let range = if frame.low_side() {
            (frame.u(bounds.min()) - margin, u)
        } else {
            (u, frame.u(bounds.max()) + margin)
        };
        region = if let Some(keep) = frame.band_box(bounds, range, margin) {
            intersection_ex_polygons(&region, &[keep])?
        } else {
            Vec::new()
        };
    }
    let Some(bounds) = BoundingBox::from_expolygons(&region) else {
        return Ok(None);
    };
    let pitch = ((flow.scaled_spacing(scale) as f64 * frame.stretch().recip()) as i64).max(1);
    let context = bands::Context {
        floor: floor_context,
        frame,
        bounds,
        scale,
        pitch,
        region: geometry::rings(&region).cloned().collect(),
        anchor: frame.leading_u(footprint_box),
    };
    let mut width = wall_width.iter().copied().flatten().next().unwrap_or(0.0);
    let first_inset = 0.5 * width - options.xy_contour_compensation.0;
    let mut by_layer = Vec::with_capacity(layers.len());
    for (layer, contact_width) in layers.iter().zip(wall_width) {
        if let Some(contact_width) = contact_width {
            width = contact_width;
        }
        by_layer.push(context.band(
            object,
            (layer.print_z, layer.height),
            0.5 * width - options.xy_contour_compensation.0,
        )?);
    }
    let mut prologue = Vec::new();
    let h = layers[0].height;
    let limit = floor_context.floor_print_z(scale.unscale(frame.leading_u(bounds))) - h;
    let mut z = layers[0].print_z - h;
    if h > EPSILON {
        while z > limit {
            let band = context.band(object, (z, h), first_inset)?;
            if !band.paths.is_empty() {
                prologue.push(band);
            }
            z -= h;
        }
    }
    prologue.reverse();
    Ok(Some(BeltBrimPlan { by_layer, prologue }))
}

fn cutoff_band(floor: BeltFloorContext, (z, height): (f64, f64)) -> (f64, f64) {
    let [a, b] = [floor.cutoff_u(z - height), floor.cutoff_u(z)];
    (a.min(b), a.max(b))
}
