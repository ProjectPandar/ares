//! `TreeSupport.cpp:2538-2622` descending hole state and Roof1 expansion.

use std::collections::HashMap;

use super::super::{
    contacts::ContactNodes, data::TreeSupportData, heights::LayerHeightData, movement::normal,
};
use super::{areas::LayerAreas, contour_intersection::intersects_contour, groups::AreaKind};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, difference_ex,
    offset_paths, union_polygons_paths_pair,
};

type HoleId = (usize, AreaKind, usize, usize);

#[derive(Clone, Copy)]
struct PropagationInfo {
    depth: i32,
    direction: Point,
    far_point: Point,
}

pub(super) struct HolePropagation<'a> {
    pub(super) contacts: &'a ContactNodes,
    pub(super) heights: &'a [LayerHeightData],
    pub(super) data: &'a mut TreeSupportData,
    pub(super) line_width_scaled: f64,
    pub(super) scale: CoordinateScale,
}

impl HolePropagation<'_> {
    pub(super) fn run(&mut self, layers: &mut [LayerAreas]) -> Result<(), ClipperError> {
        let mut infos = HashMap::new();
        for layer in (1..self.contacts.layers.len()).rev() {
            if self.contacts.layers[layer].is_empty()
                || layers[layer].height < 0.0001
                || layers[layer].groups.is_empty()
            {
                continue;
            }
            let Some(lower) = layers[..layer]
                .iter()
                .rposition(|area| !area.groups.is_empty())
            else {
                continue;
            };
            let (below, above) = layers.split_at_mut(layer);
            self.propagate_layer(
                (layer, &mut above[0]),
                (lower, &mut below[lower]),
                &mut infos,
            )?;
        }
        Ok(())
    }

    fn propagate_layer(
        &mut self,
        current: (usize, &mut LayerAreas),
        lower: (usize, &mut LayerAreas),
        infos: &mut HashMap<HoleId, PropagationInfo>,
    ) -> Result<(), ClipperError> {
        let (layer, current) = current;
        let (lower_layer, lower) = lower;
        for group in &current.groups {
            if group.kind != AreaKind::Base {
                continue;
            }
            for (hole_index, hole) in current.base[group.index].holes().iter().enumerate() {
                let id = (layer, group.kind, group.index, hole_index);
                self.propagate_hole(hole, id, (lower_layer, lower), infos)?;
                expand_roof_first(
                    hole,
                    &mut current.roof_first,
                    &current.roof,
                    self.data,
                    self.heights[layer].object_layer,
                )?;
            }
        }
        Ok(())
    }

    fn propagate_hole(
        &self,
        hole: &Polygon,
        id: HoleId,
        lower: (usize, &mut LayerAreas),
        infos: &mut HashMap<HoleId, PropagationInfo>,
    ) -> Result<(), ClipperError> {
        let (lower_layer, lower) = lower;
        for group_index in 0..lower.groups.len() {
            let group = &lower.groups[group_index];
            let (kind, index) = (group.kind, group.index);
            let area = area_mut(lower, kind, index);
            let mut contact = None;
            if area.contour().contains(&hole.points()[0]) {
                contact = Some(intersects_contour(hole, area, self.scale)?);
            }
            let propagation = if let Some(contact) = contact.filter(|contact| !contact.intersects) {
                Some((
                    PropagationInfo {
                        depth: 25,
                        direction: normal(
                            Point::new(
                                contact.on_area.x() - contact.on_hole.x(),
                                contact.on_area.y() - contact.on_hole.y(),
                            ),
                            self.line_width_scaled / 2.0,
                            self.scale,
                        ),
                        far_point: contact.far_on_hole,
                    },
                    -self.line_width_scaled / 4.0,
                ))
            } else {
                continued_propagation(infos.get(&id).copied(), area, self.line_width_scaled)
            };
            let Some((info, offset)) = propagation else {
                continue;
            };
            if let Some(hole_index) = append_hole(hole, area, offset, info)? {
                infos
                    .entry((lower_layer, kind, index, hole_index))
                    .or_insert(info);
            }
            break;
        }
        Ok(())
    }
}

fn continued_propagation(
    info: Option<PropagationInfo>,
    area: &ExPolygon,
    line_width_scaled: f64,
) -> Option<(PropagationInfo, f64)> {
    let info = info?;
    if info.depth <= 0 || !area.contour().contains(&info.far_point) {
        return None;
    }
    Some((
        PropagationInfo {
            depth: info.depth - 1,
            direction: info.direction,
            far_point: Point::new(
                info.far_point.x() + info.direction.x() * 2,
                info.far_point.y() + info.direction.y() * 2,
            ),
        },
        line_width_scaled / 2.0,
    ))
}

fn append_hole(
    hole: &Polygon,
    area: &mut ExPolygon,
    offset: f64,
    info: PropagationInfo,
) -> Result<Option<usize>, ClipperError> {
    let translated = Polygon::new(
        hole.points()
            .iter()
            .map(|point| {
                Point::new(
                    point.x() + info.direction.x(),
                    point.y() + info.direction.y(),
                )
            })
            .collect(),
    );
    let expanded = offset_paths(
        std::slice::from_ref(&translated),
        offset as f32,
        JoinType::Square,
        3.0,
    )?;
    if let Some(first) = expanded.into_iter().next() {
        let index = area.holes().len();
        area.holes_mut().push(first);
        return Ok(Some(index));
    }
    Ok(None)
}

fn area_mut(layer: &mut LayerAreas, kind: AreaKind, index: usize) -> &mut ExPolygon {
    match kind {
        AreaKind::Base => &mut layer.base[index],
        AreaKind::Roof => &mut layer.roof[index],
        AreaKind::Floor => &mut layer.floor[index],
        AreaKind::RoofFirst => &mut layer.roof_first[index],
    }
}

fn expand_roof_first(
    hole: &Polygon,
    roofs_first: &mut [ExPolygon],
    roofs: &[ExPolygon],
    data: &mut TreeSupportData,
    object_layer: usize,
) -> Result<(), ClipperError> {
    for roof in roofs_first {
        if !roof
            .contour()
            .points()
            .iter()
            .all(|point| hole.contains(point))
        {
            continue;
        }
        let mut reoriented = hole.clone();
        if roof.contour().is_counter_clockwise() != reoriented.is_counter_clockwise() {
            reoriented.reverse();
        }
        let merged = union_polygons_paths_pair(
            std::slice::from_ref(roof.contour()),
            std::slice::from_ref(&reoriented),
        )?;
        if let Some(contour) = merged.into_iter().next() {
            *roof = ExPolygon::new(contour, std::mem::take(roof.holes_mut()));
        }
        let trimmed = difference_ex(
            &difference_ex(
                std::slice::from_ref(roof),
                data.collision(0.0, object_layer)?,
            )?,
            roofs,
        )?;
        if let Some(first) = trimmed.into_iter().next() {
            *roof = first;
        }
        break;
    }
    Ok(())
}
