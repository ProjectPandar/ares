use crate::{
    SliceError,
    geometry::{CoordinateScale, Line, LineDistanceTree, Point, ThickPolyline},
    project_slice::perimeters::{
        classic::{gap_extrusion::GapFillEntity, materialize::Point3},
        types::Flow,
    },
};

use super::FillExtrusionEntity;

#[derive(Clone, Copy)]
struct Item {
    entry: Point,
    exit: Point,
    gap: Option<usize>,
    reversed: bool,
}

// FillBase.cpp:232-337: fixed-order gap insertion, before variable_width.
pub(super) fn append(
    gaps: &[ThickPolyline],
    flow: Flow,
    scale: CoordinateScale,
    entities: &mut Vec<FillExtrusionEntity>,
) -> Result<(), SliceError> {
    if entities.is_empty() {
        return super::append_polylines(gaps, flow, scale, entities);
    }
    let last_owner = bordering_entities(gaps, flow.scaled_spacing(scale) as f64, entities);
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
        let (slot, item) = insertion(gap, index, &items, last_owner[index]);
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

fn insertion(
    gap: &ThickPolyline,
    index: usize,
    items: &[Item],
    last_owner: Option<usize>,
) -> (usize, Item) {
    let mut best = f64::MAX;
    let mut best_slot = 0;
    let mut best_item = Item {
        entry: Point::new(0, 0),
        exit: Point::new(0, 0),
        gap: Some(0),
        reversed: false,
    };
    let mut extrusions_before = 0;
    for slot in 0..=items.len() {
        if slot > 0 && items[slot - 1].gap.is_none() {
            extrusions_before += 1;
        }
        if last_owner.is_some_and(|owner| extrusions_before <= owner) {
            continue;
        }
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

// FillBase.cpp:255-282: sample raw gaps against the original fill's line index.
fn bordering_entities(
    gaps: &[ThickPolyline],
    spacing: f64,
    entities: &[FillExtrusionEntity],
) -> Vec<Option<usize>> {
    let mut lines = Vec::new();
    let mut owners = Vec::new();
    for (index, entity) in entities.iter().enumerate() {
        let mut append = |points: &[Point3]| {
            for pair in points.windows(2) {
                lines.push(Line::new(
                    Point::new(pair[0].x, pair[0].y),
                    Point::new(pair[1].x, pair[1].y),
                ));
                owners.push(index);
            }
        };
        match entity {
            FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => {
                append(&path.polyline.points);
            }
            FillExtrusionEntity::VariableWidth(GapFillEntity::Path(path)) => {
                append(&path.polyline.points);
            }
            FillExtrusionEntity::VariableWidth(GapFillEntity::Loop(paths)) => {
                for path in paths {
                    append(&path.polyline.points);
                }
            }
        }
    }
    let tree = LineDistanceTree::new(&lines);
    let mut after = vec![None; gaps.len()];
    for (index, gap) in gaps.iter().enumerate() {
        let Some(maximum_width) = gap.width.iter().copied().reduce(f64::max) else {
            continue;
        };
        let radius = 0.5 * maximum_width + spacing;
        for pair in gap.points.windows(2) {
            let a = [pair[0].x() as f64, pair[0].y() as f64];
            let delta = [pair[1].x() as f64 - a[0], pair[1].y() as f64 - a[1]];
            let length = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
            let steps = ((length / (0.5 * spacing)).ceil() as usize).max(1);
            for sample in 0..=steps {
                let fraction = sample as f64 / steps as f64;
                let point = Point::new(
                    (a[0] + fraction * delta[0]) as i64,
                    (a[1] + fraction * delta[1]) as i64,
                );
                after[index] = after[index].max(
                    tree.all_lines_in_radius(point, radius)
                        .into_iter()
                        .map(|line| owners[line])
                        .max(),
                );
            }
        }
    }
    after
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
