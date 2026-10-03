//! `TreeSupport3D.cpp`'s line status conversion and next-layer splitting.

use super::super::model_volumes::{AvoidanceType, TreeModelVolumes};
use super::TreeSettings;
use crate::geometry::{ClipperError, Coord, Point, Polygon, Polyline, point_in_polygon};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LineStatus {
    Invalid,
    ToModel,
    ToModelGracious,
    ToModelGraciousSafe,
    ToBuildplate,
    ToBuildplateSafe,
}

pub(super) type LineInformation = Vec<(Point, LineStatus)>;
pub(super) type LineInformations = Vec<LineInformation>;

pub(super) fn avoidance_status(
    point: Point,
    radius: Coord,
    layer: usize,
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
) -> Result<LineStatus, ClipperError> {
    let minimum_xy = config.xy_distance > config.xy_min_distance;
    for (kind, to_model, status) in [
        (AvoidanceType::FastSafe, false, LineStatus::ToBuildplateSafe),
        (AvoidanceType::Fast, false, LineStatus::ToBuildplate),
        (
            AvoidanceType::FastSafe,
            true,
            LineStatus::ToModelGraciousSafe,
        ),
        (AvoidanceType::Fast, true, LineStatus::ToModelGracious),
    ] {
        if to_model && !config.support_rests_on_model {
            continue;
        }
        if !contains(
            volumes.avoidance((radius, layer), kind, to_model, minimum_xy)?,
            point,
        ) {
            return Ok(status);
        }
    }
    Ok(
        if config.support_rests_on_model
            && !contains(volumes.collision(radius, layer, minimum_xy)?, point)
        {
            LineStatus::ToModel
        } else {
            LineStatus::Invalid
        },
    )
}

pub(super) fn convert(
    polylines: &[Polyline],
    layer: usize,
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
) -> Result<LineInformations, ClipperError> {
    let mut result = Vec::new();
    for line in polylines {
        let mut segment = Vec::new();
        for &point in line.points() {
            let status = avoidance_status(point, config.radius(0, 0.0), layer, volumes, config)?;
            if status != LineStatus::Invalid {
                segment.push((point, status));
            } else if !segment.is_empty() {
                result.push(std::mem::take(&mut segment));
            }
        }
        if !segment.is_empty() {
            result.push(segment);
        }
    }
    Ok(result)
}

pub(super) fn valid_on_next_layer(
    data: &(Point, LineStatus),
    current_layer: usize,
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
) -> Result<bool, ClipperError> {
    let (point, status) = *data;
    let minimum_xy = config.xy_distance > config.xy_min_distance;
    let radius = config.radius(0, 0.0);
    let key = (radius, current_layer - 1);
    let kind = if status == LineStatus::ToBuildplateSafe {
        AvoidanceType::FastSafe
    } else {
        AvoidanceType::Fast
    };
    if !contains(volumes.avoidance(key, kind, false, minimum_xy)?, point) {
        return Ok(true);
    }
    if !config.support_rests_on_model {
        return Ok(false);
    }
    let forbidden = match status {
        LineStatus::ToModelGraciousSafe => {
            volumes.avoidance(key, AvoidanceType::FastSafe, true, minimum_xy)?
        }
        LineStatus::ToModelGracious => {
            volumes.avoidance(key, AvoidanceType::Fast, true, minimum_xy)?
        }
        LineStatus::ToModel | LineStatus::Invalid => {
            volumes.collision(radius, key.1, minimum_xy)?
        }
        LineStatus::ToBuildplate | LineStatus::ToBuildplateSafe => return Ok(false),
    };
    Ok(!contains(forbidden, point))
}

pub(super) fn split(
    lines: &[LineInformation],
    mut evaluate: impl FnMut(&(Point, LineStatus)) -> Result<bool, ClipperError>,
) -> Result<(LineInformations, LineInformations), ClipperError> {
    let mut groups = [Vec::new(), Vec::new()];
    for line in lines {
        let mut current_keep = true;
        let mut segment = Vec::new();
        for data in line {
            let changed = evaluate(data)? != current_keep;
            if changed && !segment.is_empty() {
                groups[usize::from(!current_keep)].push(std::mem::take(&mut segment));
            }
            if changed {
                current_keep = !current_keep;
            }
            segment.push(*data);
        }
        if !segment.is_empty() {
            groups[usize::from(!current_keep)].push(segment);
        }
    }
    let [keep, free] = groups;
    Ok((keep, free))
}

pub(super) fn contains(polygons: &[Polygon], point: Point) -> bool {
    let mut inside = 0;
    for polygon in polygons {
        let result = point_in_polygon(point, polygon.points());
        if result == -1 {
            return true;
        }
        inside += result;
    }
    inside % 2 == 1
}
