use crate::{
    SliceError,
    geometry::{CoordinateScale, Point, ThickPolyline},
    project_slice::perimeters::{classic::gap_extrusion::GapFillEntity, types::Flow},
};

use super::FillExtrusionEntity;

#[derive(Clone, Copy)]
struct Item {
    entry: Point,
    exit: Point,
    gap: Option<usize>,
    reversed: bool,
}

// FillBase.cpp:232-302: fixed-order gap insertion, before variable_width.
pub(super) fn append(
    gaps: &[ThickPolyline],
    flow: Flow,
    scale: CoordinateScale,
    entities: &mut Vec<FillExtrusionEntity>,
) -> Result<(), SliceError> {
    if entities.is_empty() {
        return super::append_polylines(gaps, flow, scale, entities);
    }
    let mut items = Vec::with_capacity(entities.len() + gaps.len());
    items.extend(entities.iter().map(entity_item));
    let mut appended = items.clone();
    for (index, gap) in gaps.iter().enumerate() {
        appended.push(Item {
            entry: gap.points[0],
            exit: gap.points[gap.points.len() - 1],
            gap: Some(index),
            reversed: false,
        });
        let (slot, item) = insertion(gap, index, &items);
        items.insert(slot, item);
    }
    if travel(&appended, gaps) <= travel(&items, gaps) {
        items = appended;
    }

    let mut out = Vec::with_capacity(entities.len() + gaps.len());
    let mut existing = std::mem::take(entities).into_iter();
    for item in items {
        if let Some(index) = item.gap {
            let mut gap = gaps[index].clone();
            if item.reversed {
                gap.reverse();
            }
            super::append_polylines(std::slice::from_ref(&gap), flow, scale, &mut out)?;
        } else {
            out.push(existing.next().unwrap());
        }
    }
    *entities = out;
    Ok(())
}

fn insertion(gap: &ThickPolyline, index: usize, items: &[Item]) -> (usize, Item) {
    let mut best = f64::MAX;
    let mut best_slot = 0;
    let mut best_item = Item {
        entry: Point::new(0, 0),
        exit: Point::new(0, 0),
        gap: Some(0),
        reversed: false,
    };
    for slot in 0..=items.len() {
        let prev = slot.checked_sub(1).map(|i| items[i].exit);
        let next = items.get(slot).map(|item| item.entry);
        let mut consider = |entry, exit, reversed| {
            let added = prev.map_or(0.0, |p| distance(p, entry))
                + next.map_or(0.0, |n| distance(exit, n))
                - prev.zip(next).map_or(0.0, |(p, n)| distance(p, n));
            if added < best {
                best = added;
                best_slot = slot;
                best_item = Item {
                    entry,
                    exit,
                    gap: Some(index),
                    reversed,
                };
            }
        };
        if closed(gap) {
            let entry = closest_point(gap, prev.or(next).unwrap());
            consider(entry, entry, false);
        } else {
            let first = gap.points[0];
            let last = gap.points[gap.points.len() - 1];
            consider(first, last, false);
            consider(last, first, true);
        }
    }
    (best_slot, best_item)
}

fn travel(items: &[Item], gaps: &[ThickPolyline]) -> f64 {
    let mut sum = 0.0;
    let mut at = items[0].exit;
    for item in &items[1..] {
        let closed_gap = item.gap.filter(|&index| closed(&gaps[index]));
        let entry = closed_gap.map_or(item.entry, |index| closest_point(&gaps[index], at));
        sum += distance(at, entry);
        at = if closed_gap.is_some() {
            entry
        } else {
            item.exit
        };
    }
    sum
}

fn closed(gap: &ThickPolyline) -> bool {
    gap.points[0] == gap.points[gap.points.len() - 1]
}

// MultiPoint.hpp:62-77: nearest vertex, Euclidean norm and first strict minimum.
fn closest_point(gap: &ThickPolyline, at: Point) -> Point {
    let mut point = gap.points[0];
    let mut best = distance(point, at);
    for &candidate in &gap.points[1..] {
        let value = distance(at, candidate);
        if value < best {
            best = value;
            point = candidate;
        }
    }
    point
}

fn distance(a: Point, b: Point) -> f64 {
    let x = (b.x() - a.x()) as f64;
    let y = (b.y() - a.y()) as f64;
    (x * x + y * y).sqrt()
}

fn entity_item(entity: &FillExtrusionEntity) -> Item {
    let (first, last) = match entity {
        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => {
            let points = &path.polyline.points;
            (points[0], points[points.len() - 1])
        }
        FillExtrusionEntity::VariableWidth(GapFillEntity::Path(path)) => {
            let points = &path.polyline.points;
            (points[0], points[points.len() - 1])
        }
        FillExtrusionEntity::VariableWidth(GapFillEntity::Loop(paths)) => {
            // ExtrusionLoop::last_point (ExtrusionEntity.hpp:495) equals first_point.
            let first = paths[0].polyline.points[0];
            (first, first)
        }
    };
    Item {
        entry: Point::new(first.x, first.y),
        exit: Point::new(last.x, last.y),
        gap: None,
        reversed: false,
    }
}
