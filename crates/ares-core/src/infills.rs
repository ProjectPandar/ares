use crate::{
    InfillOptions, InfillPattern, Layer, LayerContours, Point2, SliceError,
    bridge_support::fully_unsupported_layer, options::InfillLayerRole,
};

mod area;
mod calibration;
mod combination;
mod concentric;
mod elephant_foot;
mod internal_bridge;
mod multiline;
mod narrow_internal;
mod overlap;
mod rotation;
mod scanline;
mod spacing;
mod spiral_vase;
mod symmetry;
mod top_surface;

use area::filled_area_mm2;
use rotation::{InfillLayerPosition, InfillPasses};
#[cfg(test)]
use scanline::compare_points;
use scanline::{
    ScanlineBasis, Vector2, anchored_segment, clip_contours, compare_candidates, scanline_bounds,
    transform_contour,
};
use symmetry::{mirror_axis_x, mirror_contour_x, mirror_point_x};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfillRole {
    Sparse,
    Solid,
    BottomSurface,
    TopSurface,
    InternalBridge,
}

impl InfillRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sparse => "sparse",
            Self::Solid | Self::BottomSurface | Self::TopSurface => "solid",
            Self::InternalBridge => "internal_bridge",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InfillPath {
    role: InfillRole,
    points: Vec<Point2>,
    effective_layer_height_mm: f64,
}

impl InfillPath {
    pub fn new(
        role: InfillRole,
        points: Vec<Point2>,
        effective_layer_height_mm: f64,
    ) -> Result<Self, SliceError> {
        if points.len() != 2 {
            return Err(SliceError::InvalidInput(
                "infill path must contain exactly two points".to_owned(),
            ));
        }
        Ok(Self {
            role,
            points,
            effective_layer_height_mm,
        })
    }

    pub const fn role(&self) -> InfillRole {
        self.role
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }

    pub const fn effective_layer_height_mm(&self) -> f64 {
        self.effective_layer_height_mm
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerInfills {
    layer_id: usize,
    print_z: f64,
    paths: Vec<InfillPath>,
}

impl LayerInfills {
    pub fn new(layer_id: usize, print_z: f64, paths: Vec<InfillPath>) -> Self {
        Self {
            layer_id,
            print_z,
            paths,
        }
    }

    pub const fn layer_id(&self) -> usize {
        self.layer_id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub fn paths(&self) -> &[InfillPath] {
        &self.paths
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct InfillBridgeContext<'a> {
    layer_contours: &'a [LayerContours],
    bridge_no_support: bool,
    extra_bridge_layer: crate::bridges::ExtraBridgeLayer,
    counterbore_hole_bridging: crate::bridges::CounterboreHoleBridging,
}

impl<'a> InfillBridgeContext<'a> {
    pub(crate) const fn new(
        layer_contours: &'a [LayerContours],
        bridge_no_support: bool,
        extra_bridge_layer: crate::bridges::ExtraBridgeLayer,
        counterbore_hole_bridging: crate::bridges::CounterboreHoleBridging,
    ) -> Self {
        Self {
            layer_contours,
            bridge_no_support,
            extra_bridge_layer,
            counterbore_hole_bridging,
        }
    }

    fn overrides_bottom_surface(self, layer_index: usize) -> bool {
        self.bridge_no_support
            && !self
                .counterbore_hole_bridging
                .preserves_bridge_surfaces_for_solid_detection()
            && fully_unsupported_layer(self.layer_contours, layer_index)
    }

    fn overrides_extra_external_bridge_layer(self, layer_index: usize) -> bool {
        layer_index > 0
            && self.bridge_no_support
            && self.extra_bridge_layer.applies_to_external_bridge()
            && fully_unsupported_layer(self.layer_contours, layer_index - 1)
            && !fully_unsupported_layer(self.layer_contours, layer_index)
    }
}

pub fn generate_infills(
    print_layers: &[Layer],
    layers: &[LayerContours],
    options: InfillOptions,
) -> Result<Vec<LayerInfills>, SliceError> {
    generate_infills_with_bridge_context(print_layers, layers, options, None)
}

pub(crate) fn generate_infills_with_bridge_context(
    print_layers: &[Layer],
    layers: &[LayerContours],
    options: InfillOptions,
    bridge_context: Option<InfillBridgeContext<'_>>,
) -> Result<Vec<LayerInfills>, SliceError> {
    let options = &options;
    if print_layers.len() != layers.len() {
        return Err(SliceError::InvalidInput(
            "layer and infill contour metadata must match".to_owned(),
        ));
    }
    for (print_layer, layer) in print_layers.iter().zip(layers.iter()) {
        if print_layer.id() != layer.layer_id() || print_layer.print_z() != layer.print_z() {
            return Err(SliceError::InvalidInput(
                "layer and infill contour metadata must match".to_owned(),
            ));
        }
    }

    let spiral_base_layer_count = options.spiral_base_layer_count(layers.len());
    if options.sparse_density_percent() == 0.0 && spiral_base_layer_count == 0 {
        return Ok(layers
            .iter()
            .map(|layer| LayerInfills::new(layer.layer_id(), layer.print_z(), Vec::new()))
            .collect());
    }

    let sparse_spacing = spiral_vase::sparse_spacing(options);

    let layer_count = layers.len();
    let mut infills = layers
        .iter()
        .zip(print_layers.iter())
        .enumerate()
        .map(|(layer_index, (layer, print_layer))| {
            let role = options.layer_role_for_layers(print_layers, layer_index);
            if let Some(empty) = spiral_vase::empty_if_zero_density_sparse(layer, options, role) {
                return Ok(empty);
            }
            let bridge_override =
                internal_bridge::bridge_infill_override(role, layer_index, options, bridge_context);
            generate_layer_infills(LayerInfillInput {
                layer,
                options,
                role,
                layer_position: InfillLayerPosition {
                    index: layer_index,
                    count: layer_count,
                    id: print_layer.id(),
                },
                bridge_override,
                sparse_spacing,
                effective_layer_height_mm: print_layer.height(),
            })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    combination::apply(print_layers, &mut infills, options);
    Ok(infills)
}

struct LayerInfillInput<'a> {
    layer: &'a LayerContours,
    options: &'a InfillOptions,
    role: InfillLayerRole,
    layer_position: InfillLayerPosition,
    bridge_override: Option<internal_bridge::BridgeInfillOverride>,
    sparse_spacing: f64,
    effective_layer_height_mm: f64,
}

fn generate_layer_infills(input: LayerInfillInput<'_>) -> Result<LayerInfills, SliceError> {
    let LayerInfillInput {
        layer,
        options,
        role,
        layer_position,
        bridge_override,
        sparse_spacing,
        effective_layer_height_mm,
    } = input;
    let adjusted = top_surface::filter_contours(
        role,
        overlap::adjusted_contours(
            layer,
            role,
            layer_position.index,
            layer_position.count,
            options,
        ),
        options,
    );
    let mut contours = Vec::new();
    for points in &adjusted {
        if points.len() < 3 {
            return Err(SliceError::InvalidInput(
                "infill contour has fewer than three points".to_owned(),
            ));
        }
        contours.push(points.as_slice());
    }
    if filled_area_mm2(&contours) <= options.minimum_sparse_infill_area_mm2() {
        return Ok(LayerInfills::new(
            layer.layer_id(),
            layer.print_z(),
            Vec::new(),
        ));
    }

    let uses_internal_bridge_density = internal_bridge::uses_density(role, options, &contours);
    let spacing = spacing::for_role(spacing::SpacingRequest {
        role,
        options,
        sparse_spacing,
        bridge_override,
        uses_internal_bridge_density,
        layer_index: layer_position.index,
    });
    let Some(spacing) = spacing else {
        return Ok(LayerInfills::new(
            layer.layer_id(),
            layer.print_z(),
            Vec::new(),
        ));
    };
    let fixed_angle_degrees = internal_bridge::fixed_angle_degrees(bridge_override, &contours)
        .or_else(|| internal_bridge::angle_override(role, options, &contours));
    let passes = InfillPasses::new(role, layer_position, options, fixed_angle_degrees);
    let path_role = spiral_vase::path_role(role, options, uses_internal_bridge_density);

    if !uses_internal_bridge_density
        && let Some((InfillPattern::ConcentricInternal, bounds)) =
            narrow_internal::concentric_internal_override(role, &contours, options)
    {
        return Ok(LayerInfills::new(
            layer.layer_id(),
            layer.print_z(),
            narrow_internal::concentric_internal_segments(
                bounds,
                options.solid_line_width(),
                effective_layer_height_mm,
            )?,
        ));
    }
    if let Some(layer_infills) = concentric::try_layer(
        concentric::LayerInput {
            layer_id: layer.layer_id(),
            print_z: layer.print_z(),
            role,
            path_role,
            contours: &contours,
            spacing,
            effective_layer_height_mm,
        },
        options,
    )? {
        return Ok(layer_infills);
    }

    let mut candidates = Vec::new();
    let pattern = role.pattern(options);
    let mirror_axis_x = mirror_axis_x(pattern, options, &contours);
    for &angle_degrees in &passes.angles_degrees {
        let angle = angle_degrees.to_radians();
        let u = Vector2::new(-angle.sin(), angle.cos());
        let v = Vector2::new(angle.cos(), angle.sin());
        let basis = ScanlineBasis::new(u, v);
        let transformed = contours
            .iter()
            .map(|points| {
                let mirrored;
                let points = if let Some(axis_x) = mirror_axis_x {
                    mirrored = mirror_contour_x(points, axis_x);
                    mirrored.as_slice()
                } else {
                    points
                };
                transform_contour(points, u, v)
            })
            .collect::<Vec<_>>();
        let bounds = scanline_bounds(&transformed);
        let mut clipped = clip_contours(
            &transformed,
            basis,
            multiline::source_spacing(role, pattern, spacing, options),
            passes.scanline_shift_mm,
            passes.normalize_segments,
        );
        clipped = multiline::expand_candidates(
            clipped,
            multiline::Expansion {
                normal: v,
                bounds,
                role,
                pattern,
                options,
            },
        );
        if let Some(axis_x) = mirror_axis_x {
            for candidate in &mut clipped {
                candidate.start = mirror_point_x(candidate.start, axis_x);
                candidate.end = mirror_point_x(candidate.end, axis_x);
            }
        }
        candidates.extend(clipped);
    }
    candidates.sort_by(compare_candidates);

    let anchor_length = if role.is_sparse() {
        options.infill_anchor_length_mm()
    } else {
        0.0
    };
    let paths = candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let (start, end) = anchored_segment(candidate.start, candidate.end, anchor_length);
            let already_reversed = passes.alternate_segments && index % 2 == 1;
            let points = if calibration::reverse_segment(role, options, already_reversed) {
                vec![end, start]
            } else {
                vec![start, end]
            };
            InfillPath::new(path_role, points, effective_layer_height_mm)
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(LayerInfills::new(layer.layer_id(), layer.print_z(), paths))
}

#[cfg(test)]
mod tests;
