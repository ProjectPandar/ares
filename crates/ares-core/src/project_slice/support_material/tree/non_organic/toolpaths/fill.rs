//! `TreeSupport.cpp:1210-1242` with source `Fill::new_from_type` kernels.

use crate::{
    ExtrusionRole,
    fill::{
        honeycomb::{HoneycombFillParams, fill_honeycomb},
        multiline::{MultilineFillParams, Sweep},
        rectilinear::{MonotonicFillParams, fill_rectilinear_surface},
    },
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polyline,
        offset_expolygon,
    },
    project_slice::{
        perimeters::types::Flow,
        raft::{
            fill_params::RaftFillPattern,
            fills::{concentric, support_base},
        },
        support_material::SupportExtrusionEntity,
    },
};

pub(super) struct Filler {
    pub(super) scale: CoordinateScale,
    pub(super) pattern: RaftFillPattern,
    pub(super) spacing: f64,
    pub(super) angle: f32,
    pub(super) reference: Option<Point>,
}

#[derive(Clone, Copy)]
pub(super) struct FillSettings {
    pub(super) density: f64,
    pub(super) dont_sort: bool,
    pub(super) flow: Flow,
    pub(super) role: ExtrusionRole,
}

impl Filler {
    pub(super) fn append(
        &self,
        destination: &mut Vec<SupportExtrusionEntity>,
        areas: &[ExPolygon],
        settings: FillSettings,
    ) -> Result<Vec<BoundingBox>, ClipperError> {
        let mut boxes = Vec::with_capacity(areas.len());
        for area in areas {
            let lines = self.fill_surface(area, settings)?;
            boxes.push(emitted_bounds(&lines));
            SupportExtrusionEntity::append_paths(destination, lines, settings.role, settings.flow);
        }
        Ok(boxes)
    }

    fn fill_surface(
        &self,
        area: &ExPolygon,
        settings: FillSettings,
    ) -> Result<Vec<Polyline>, ClipperError> {
        let scale = self.scale;
        let reference = self.reference.unwrap_or_else(|| {
            BoundingBox::from_expolygon(area)
                .expect("support fill has a contour")
                .center()
        });
        let density = settings.density as f32;
        match self.pattern {
            RaftFillPattern::Rectilinear => Ok(fill_rectilinear_surface(
                area,
                MonotonicFillParams {
                    dont_alternate_fill_direction: false,
                    layer_angle_fixed: false,
                    spacing: self.spacing,
                    overlap: 0.0,
                    density,
                    angle: self.angle,
                    layer_index: 0,
                    thickness_layers: 1,
                    fixed_angle: true,
                    bridge_angle: None,
                    reference_point: reference,
                    dont_adjust: true,
                    anchor_length_max: 1000.0,
                    link_max_length: 0.0,
                    consistent_pattern: false,
                    horiz_move: 0.0,
                    extended_object_min_x: None,
                    extended_object_max_x: None,
                    locked_zag: false,
                },
                scale,
            )?
            .polylines),
            RaftFillPattern::Grid => {
                let mut lines = crate::fill::multiline::fill_surface(
                    area,
                    MultilineFillParams {
                        spacing: self.spacing,
                        overlap: 0.0,
                        angle: self.angle + std::f32::consts::FRAC_PI_2,
                        reference,
                        density,
                        multiline: 1,
                        anchor_length: 1000.0,
                        anchor_length_max: 1000.0,
                        dont_sort: settings.dont_sort,
                    },
                    &[
                        Sweep {
                            angle: 0.0,
                            shift: 0.0,
                        },
                        Sweep {
                            angle: std::f32::consts::FRAC_PI_2,
                            shift: 0.0,
                        },
                    ],
                    scale,
                )?;
                // The source support filler retains layer_id = size_t(-1).
                for line in &mut lines {
                    line.reverse();
                }
                Ok(lines)
            }
            RaftFillPattern::SupportBase => support_base::fill_surface(
                area,
                support_base::SupportBaseFillParams {
                    spacing: self.spacing,
                    density,
                    angle: self.angle,
                    reference,
                    reference_angle: self.angle + std::f32::consts::FRAC_PI_2,
                    outer_offset: scale.scaled_delta(-0.5 * self.spacing) as f32,
                },
                scale,
            ),
            RaftFillPattern::Honeycomb => {
                let mut lines = Vec::new();
                for component in offset_expolygon(
                    area,
                    scale.scaled_delta(-0.5 * self.spacing) as f32,
                    JoinType::Miter,
                    3.0,
                )? {
                    lines.extend(fill_honeycomb(
                        &component,
                        HoneycombFillParams {
                            dont_alternate_fill_direction: false,
                            spacing: self.spacing,
                            density,
                            multiline: 1,
                            smooth_factor: 0.0,
                            resolution: 0.0125,
                            angle: self.angle,
                            layer_index: 0,
                            thickness_layers: 1,
                            fixed_angle: true,
                            bridge_angle: None,
                            anchor_length: 1000.0,
                            anchor_length_max: 1000.0,
                        },
                        scale,
                    )?);
                }
                Ok(lines)
            }
            RaftFillPattern::Concentric => concentric(area, self.spacing, density, scale),
            RaftFillPattern::SpiralInset => {
                crate::fill::spiral_inset::support(area, self.spacing, density, scale)
            }
        }
    }
}

fn emitted_bounds(lines: &[Polyline]) -> BoundingBox {
    let bounds = |line: &Polyline| {
        let mut bounds = BoundingBox::new(line.points()[0], line.points()[0]);
        for &point in &line.points()[1..] {
            bounds.merge_point(point);
        }
        bounds
    };
    let Some(first) = lines.first() else {
        return BoundingBox::new(Point::new(0, 0), Point::new(0, 0));
    };
    let mut output = bounds(first);
    let defined = |bounds: BoundingBox| {
        bounds.min().x() < bounds.max().x() && bounds.min().y() < bounds.max().y()
    };
    for line in &lines[1..] {
        let next = bounds(line);
        if !defined(next) {
            continue;
        }
        if defined(output) {
            output.merge_point(next.min());
            output.merge_point(next.max());
        } else {
            output = next;
        }
    }
    output
}
