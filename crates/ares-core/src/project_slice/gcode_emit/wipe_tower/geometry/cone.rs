//! `WipeTower2::generate_support_cone_wall` and `get_wipe_tower_cone_base`.
use super::super::writer::Writer;
use super::{BoxCoordinates, Shape, scaled, unscaled};
use crate::{
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface},
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
        difference_ex_polygons, offset_expolygon, offset_paths,
    },
};
use std::f64::consts::PI;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Corner,
    Arc,
    End,
}

pub(in super::super) fn cone_base(width: f64, height: f64, depth: f64, angle: f64) -> (f64, f64) {
    let radius = (angle / 2.0 * PI / 180.0).tan() * height;
    let width = 0.66 * width;
    let diagonal = (width / 2.0).hypot(depth / 2.0);
    let mut support_scale = 1.0;
    if radius > diagonal {
        let sin = 0.5 * depth / diagonal;
        let tan = depth / width;
        let t = (radius - diagonal) * sin;
        support_scale = (width / 2.0 + t / tan + t * tan) / (width / 2.0);
    }
    (radius, support_scale)
}

impl Shape {
    pub(in super::super) fn cone_wall(
        &self,
        bounds: BoxCoordinates,
        writer: &mut Writer,
        options: ConeOptions,
    ) -> Result<Polygon, ClipperError> {
        let ConeOptions {
            angle,
            z,
            spacing,
            infill,
            feedrate,
            scale,
        } = options;
        let (_, stretch) = cone_base(
            f64::from(self.width),
            f64::from(self.height),
            f64::from(self.depth),
            f64::from(angle),
        );
        let radius = f64::from((std::f32::consts::PI * (angle / 2.0) / 180.0).tan())
            * (f64::from(self.height) - z);
        let center = [
            (bounds.lower[0] + bounds.upper[0]) / 2.0,
            (bounds.lower[1] + bounds.upper[1]) / 2.0,
        ];
        let width = f64::from(bounds.upper[1] - bounds.lower[1]);
        let mut points = vec![(bounds.upper, Kind::Corner)];
        let alpha_start = ((0.5 * width) / radius).asin();
        if !alpha_start.is_nan() && radius > 0.5 * width + 0.01 {
            let mut alpha = alpha_start;
            while alpha < PI - alpha_start + 0.001 {
                points.push((
                    [
                        (f64::from(center[0]) + radius * alpha.cos() / stretch) as f32,
                        (f64::from(center[1]) + radius * alpha.sin()) as f32,
                    ],
                    Kind::Arc,
                ));
                alpha += (PI - 2.0 * alpha_start) / 40.0;
            }
            points.last_mut().unwrap().1 = Kind::End;
        }
        points.push(([bounds.lower[0], bounds.upper[1]], Kind::Corner));
        points.push((bounds.lower, Kind::Corner));
        for index in (1..=points.len() - 3).rev() {
            let point = points[index].0;
            points.push((
                [point[0], 2.0 * center[1] - point[1]],
                if index == 1 { Kind::End } else { Kind::Arc },
            ));
        }
        points.push(([bounds.upper[0], bounds.lower[1]], Kind::Corner));
        let polygon = Polygon::new(
            points
                .iter()
                .map(|&(point, _)| scaled(point, scale))
                .collect(),
        );
        let paths = if infill && self.width > 2.0 * spacing && self.depth > 2.0 * spacing {
            cone_infill(&polygon, bounds, spacing, scale)?
        } else {
            Vec::new()
        };
        let start = points
            .iter()
            .enumerate()
            .filter(|(_, (_, kind))| *kind == Kind::Corner)
            .min_by(|(_, (a, _)), (_, (b, _))| {
                let distance = |p: [f32; 2]| {
                    let x = p[0] - writer.position[0];
                    let y = p[1] - writer.position[1];
                    x * x + y * y
                };
                distance(*a).total_cmp(&distance(*b))
            })
            .unwrap()
            .0;
        writer.travel(points[start].0, 0.0);
        for step in 1..points.len() {
            let (point, kind) = points[(start + step) % points.len()];
            writer.extrude(point, feedrate);
            if kind == Kind::End && !paths.is_empty() {
                emit_cone_infill(writer, &paths, center, point, scale);
            }
        }
        writer.extrude(points[start].0, feedrate);
        Ok(polygon)
    }
}

pub(in super::super) struct ConeOptions {
    pub(in super::super) angle: f32,
    pub(in super::super) z: f64,
    pub(in super::super) spacing: f32,
    pub(in super::super) infill: bool,
    pub(in super::super) feedrate: f32,
    pub(in super::super) scale: CoordinateScale,
}

fn emit_cone_infill(
    writer: &mut Writer,
    paths: &[Polyline],
    center: [f32; 2],
    point: [f32; 2],
    scale: CoordinateScale,
) {
    let mirror = f64::from(point[1] - center[1])
        * (scale.unscale(paths[0].points()[0].y()) - f64::from(center[1]))
        < 0.0;
    for path in paths {
        let transform = |p| {
            let p = unscaled(p, scale);
            let sign = if mirror { 1.0 } else { -1.0 };
            [
                center[0] - sign * (p[0] - center[0]),
                center[1] - sign * (p[1] - center[1]),
            ]
        };
        writer.travel(transform(path.points()[0]), 0.0);
        for &p in path.points() {
            writer.extrude(transform(p), 0.0);
        }
    }
    writer.travel(point, 0.0);
}

fn cone_infill(
    polygon: &Polygon,
    bounds: BoxCoordinates,
    spacing: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let delta = (-f64::from(spacing) / 2.0 / scale.factor()) as f32;
    let rectangle = offset_paths(
        &[bounds.polygon_new_scale(scale)],
        delta,
        JoinType::Miter,
        3.0,
    )?;
    let contour = offset_expolygon(
        &ExPolygon::new(polygon.clone(), Vec::new()),
        delta,
        JoinType::Miter,
        3.0,
    )?;
    let areas = difference_ex_polygons(&contour[..1], &rectangle[..1])?;
    if areas.len() != 2 {
        return Ok(Vec::new());
    }
    let bottom = if areas[0].contour().points()[0].y() < areas[1].contour().points()[0].y() {
        &areas[0]
    } else {
        &areas[1]
    };
    let points = bottom.contour().points();
    let min_x = points.iter().map(|p| p.x()).min().unwrap();
    let max_x = points.iter().map(|p| p.x()).max().unwrap();
    let min_y = points.iter().map(|p| p.y()).min().unwrap();
    let max_y = points.iter().map(|p| p.y()).max().unwrap();
    let params = MonotonicFillParams {
        dont_alternate_fill_direction: false,
        layer_angle_fixed: false,
        spacing: f64::from(spacing),
        overlap: 0.0,
        density: 1.0,
        angle: std::f32::consts::PI * 45.0 / 180.0,
        layer_index: 0,
        thickness_layers: 1,
        fixed_angle: true,
        bridge_angle: None,
        reference_point: Point::new((min_x + max_x) / 2, (min_y + max_y) / 2),
        dont_adjust: true,
        anchor_length_max: 0.0,
        link_max_length: 0.0,
        consistent_pattern: false,
        horiz_move: 0.0,
        extended_object_min_x: None,
        extended_object_max_x: None,
        locked_zag: false,
    };
    let mut paths = fill_monotonic_surface(bottom, params, scale)?.polylines;
    if !paths.is_empty()
        && paths[0].points()[0].x() > paths.last().unwrap().points().last().unwrap().x()
    {
        paths.reverse();
        for path in &mut paths {
            path.reverse();
        }
    }
    Ok(paths)
}
