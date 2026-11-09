//! Shared `PerimeterGenerator.cpp:1728–1929`, Bridges and Filled.
mod detector;
mod filled;
use super::super::types::{PerimeterInputRecord, PostPerimeterInputPrintObject};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, append_simplified_expolygon,
    difference_ex, difference_ex_with_safety_offset, intersection_ex, offset_expolygons,
    offset2_ex, opening_ex, union_ex, union_expolygons,
};
use crate::project_slice::region_slices::RegionSurface;
use crate::{ProcessCounterboreHoleBridging, RegionOptions, SliceError};

struct BridgeGeometry {
    spacing: i64,
    width: i64,
    margin: f64,
    anchor: f64,
}

struct SurfaceContext<'a> {
    lower: &'a [ExPolygon],
    config: &'a RegionOptions,
    model_rotation: f64,
    geometry: BridgeGeometry,
}

struct SurfaceReference {
    capacity: usize,
    valid: bool,
}

impl SurfaceReference {
    fn append(&mut self, len: usize) {
        // Native libstdc++ _M_check_len(1): logical growth, independent of Rust Vec.
        if len == self.capacity {
            self.capacity = len + len.max(1);
            self.valid = false;
        }
    }
}

pub(super) fn process(
    object: &mut PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
    lower: Option<&[ExPolygon]>,
    (spacing, external_width): (i64, i64),
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, SliceError> {
    let config = object.region_options(record);
    if config.counterbore_hole_bridging == ProcessCounterboreHoleBridging::None {
        return Ok(Vec::new());
    }
    let Some(lower) = lower.filter(|lower| !lower.is_empty()) else {
        return Ok(Vec::new());
    };
    let mut surfaces = object.perimeter_surfaces(record).to_vec();
    // libslic3r.h:89 BRIDGE_INFILL_MARGIN = 1 mm.
    let context = SurfaceContext {
        lower,
        config,
        model_rotation: record.model_rotation_rad,
        geometry: BridgeGeometry {
            spacing,
            width: external_width,
            margin: 1.0 / scale.factor(),
            anchor: (1.0 / scale.factor())
                .min(spacing as f64)
                .min(external_width as f64),
        },
    };
    let fill = process_surfaces(&mut surfaces, &context).map_err(geometry_error)?;
    let slot = object.region_layer_record_index(record.current);
    let group = &mut object.groups[object.record_groups[slot].unwrap()];
    if group.owner == record.current {
        group.merged_surfaces = Some(surfaces);
    }
    Ok(fill)
}

fn process_surfaces(
    surfaces: &mut Vec<RegionSurface>,
    context: &SurfaceContext<'_>,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let mut fill = Vec::new();
    let mut index = 0;
    // Surface copy (:1212/:2121) has capacity == size; capture current at :1739.
    let mut reference = SurfaceReference {
        capacity: surfaces.len(),
        valid: true,
    };
    while index < surfaces.len() {
        reference.valid = true;
        let original = surfaces[index].clone();
        let last = process_surface(
            original.as_parts().1,
            context,
            &mut fill,
            (surfaces, &mut index, &mut reference),
        )?;
        let mut parts = last.into_iter();
        let Some(first) = parts.next() else {
            surfaces.remove(index);
            continue;
        };
        // :1922: growth can invalidate the captured Native reference; live slot stays.
        if reference.valid {
            surfaces[index] = original.clone_with_expolygon(first);
        }
        for polygon in parts {
            reference.append(surfaces.len());
            surfaces.push(original.clone_with_expolygon(polygon));
        }
        index += 1;
    }
    Ok(fill)
}

fn process_surface(
    original: &ExPolygon,
    context: &SurfaceContext<'_>,
    fill: &mut Vec<ExPolygon>,
    traversal: (&mut Vec<RegionSurface>, &mut usize, &mut SurfaceReference),
) -> Result<Vec<ExPolygon>, ClipperError> {
    let (surfaces, index, reference) = traversal;
    let mut last = vec![original.clone()];
    let unsupported = difference_ex_with_safety_offset(&last, context.lower)?;
    if unsupported.is_empty() {
        return Ok(last);
    }
    let spacing = context.geometry.spacing;
    let filtered = offset2_ex(
        &unsupported,
        (-spacing) as f32,
        spacing as f32,
        JoinType::Miter,
        3.0,
    )?;
    if filtered.is_empty() {
        return Ok(last);
    }
    let support = difference_ex_with_safety_offset(&last, &unsupported)?;
    let angle = bridge_angle(context);
    let mut bridgeable = Vec::new();
    for polygon in &filtered {
        let coverage = detector::bridgeable(polygon, &support, spacing / 4, angle)?;
        bridgeable.extend(union_ex(&coverage, FillRule::NonZero)?);
    }
    if bridgeable.is_empty() || original.holes().is_empty() {
        return Ok(last);
    }
    let mask = match context.config.counterbore_hole_bridging {
        ProcessCounterboreHoleBridging::SacrificialLayer => filled::mask(
            (surfaces, index, reference),
            &last,
            &filtered,
            &bridgeable,
            context,
        )?,
        ProcessCounterboreHoleBridging::PartiallyBridged => {
            last = difference_ex_with_safety_offset(&last, &filtered)?;
            partial_mask(&last, &filtered, &bridgeable, &context.geometry)?
        }
        ProcessCounterboreHoleBridging::None => {
            unreachable!("None is gated before surface processing")
        }
    };
    if mask.is_empty() {
        return Ok(last);
    }
    fill.extend(mask.iter().cloned());
    last = difference_ex_with_safety_offset(&last, &mask)?;
    let mut rest = Vec::new();
    for polygon in last {
        if intersection_ex(&support, std::slice::from_ref(&polygon))?.is_empty() {
            fill.push(polygon);
        } else {
            rest.push(polygon);
        }
    }
    Ok(rest)
}

fn bridge_angle(context: &SurfaceContext<'_>) -> f64 {
    let config = context.config;
    let custom = config.bridge_angle.0;
    if custom > 0.0 && !config.relative_bridge_angle.0 {
        custom.to_radians()
            + if config.align_infill_direction_to_model.0 {
                context.model_rotation
            } else {
                0.0
            }
    } else {
        0.0
    }
}

// PerimeterGenerator.cpp:1865–1891: preserve operation and safety-offset order.
fn partial_mask(
    last: &[ExPolygon],
    unsupported: &[ExPolygon],
    bridgeable: &[ExPolygon],
    geometry: &BridgeGeometry,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let BridgeGeometry {
        spacing,
        width,
        anchor,
        ..
    } = *geometry;
    let mut filtered = Vec::new();
    for polygon in bridgeable {
        append_simplified_expolygon(polygon.clone(), spacing as f64, &mut filtered)?;
    }
    filtered = opening_ex(&filtered, width as f32, JoinType::Miter, 3.0)?;
    filtered = union_expolygons(
        &offset_expolygons(last, spacing as f32, JoinType::Miter, 3.0)?,
        &filtered,
    )?;
    filtered = offset_expolygons(&filtered, (-spacing) as f32, JoinType::Miter, 3.0)?;
    filtered = difference_ex_with_safety_offset(&filtered, last)?;
    filtered = opening_ex(&filtered, spacing as f32, JoinType::Miter, 3.0)?;
    filtered = offset_expolygons(&filtered, spacing as f32, JoinType::Miter, 3.0)?;
    filtered = difference_ex(
        &filtered,
        &offset_expolygons(last, (-anchor) as f32, JoinType::Miter, 3.0)?,
    )?;
    let anchors = intersection_ex(
        last,
        &offset_expolygons(unsupported, anchor as f32, JoinType::Miter, 3.0)?,
    )?;
    opening_ex(
        &union_expolygons(&filtered, &anchors)?,
        anchor as f32,
        JoinType::Miter,
        3.0,
    )
}

fn geometry_error(_: ClipperError) -> SliceError {
    SliceError::InvalidInput("invalid classic counterbore bridging geometry".to_owned())
}
