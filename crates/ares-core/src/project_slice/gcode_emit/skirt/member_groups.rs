//! Print.cpp:2784-2900: stable membership and fixed-point skirt envelopes.

use super::*;

pub(super) struct Group {
    pub(super) members: Vec<usize>,
    pub(super) hull: Polygon,
}

struct Item {
    members: Vec<usize>,
    points: Vec<Point>,
    hull: Polygon,
    /// `emits_skirt` (Print.cpp:2785-2789): obstacle items such as the
    /// wipe tower never emit a skirt and do not expand their envelope
    /// (`:2874-2880`); groups without an emitting member produce no
    /// skirt (`:2901+`).
    emits: bool,
}

pub(super) fn merge(
    items: Vec<(usize, Vec<Point>)>,
    obstacle: Option<&[Point]>,
    distance: f32,
    tolerance: f64,
) -> Result<Vec<Group>, SliceError> {
    let mut groups = items
        .into_iter()
        .map(|(member, points)| Item {
            hull: Polygon::new(convex_hull(&points)),
            members: vec![member],
            points,
            emits: true,
        })
        .chain(obstacle.map(|points| Item {
            hull: Polygon::new(convex_hull(points)),
            members: Vec::new(),
            points: points.to_vec(),
            emits: false,
        }))
        .collect::<Vec<_>>();
    loop {
        let envelopes = groups
            .iter()
            .map(|item| envelope(item, distance, tolerance))
            .collect::<Result<Vec<_>, _>>()?;
        let mut parent = (0..groups.len()).collect::<Vec<_>>();
        let changed = merge_collisions(&envelopes, &mut parent)?;
        if !changed {
            break;
        }
        groups = merged_items(groups, &mut parent);
    }
    Ok(groups
        .into_iter()
        .filter(|item| item.emits)
        .map(|item| Group {
            members: item.members,
            hull: item.hull,
        })
        .collect())
}

/// `:2874-2880`: only emitting groups expand to their final skirt
/// reach; obstacle envelopes stay the raw occupied hull.
fn envelope(item: &Item, distance: f32, tolerance: f64) -> Result<Vec<Polygon>, SliceError> {
    if !item.emits {
        return Ok(vec![item.hull.clone()]);
    }
    offset_paths(
        std::slice::from_ref(&item.hull),
        distance,
        JoinType::Round,
        tolerance,
    )
    .map_err(|_| SliceError::InvalidInput("skirt group geometry failed".into()))
}

fn merged_items(groups: Vec<Item>, parent: &mut [usize]) -> Vec<Item> {
    let mut merged = (0..groups.len())
        .map(|_| Option::<Item>::None)
        .collect::<Vec<_>>();
    for (index, mut item) in groups.into_iter().enumerate() {
        let owner = root(parent, index);
        let slot = &mut merged[owner];
        match slot {
            Some(slot) => {
                slot.points.append(&mut item.points);
                slot.members.append(&mut item.members);
                slot.emits = slot.emits || item.emits;
            }
            None => *slot = Some(item),
        }
    }
    merged
        .into_iter()
        .flatten()
        .map(|mut item| {
            item.hull = Polygon::new(convex_hull(&item.points));
            item
        })
        .collect()
}

fn merge_collisions(envelopes: &[Vec<Polygon>], parent: &mut [usize]) -> Result<bool, SliceError> {
    let mut changed = false;
    for i in 0..envelopes.len() {
        for j in i + 1..envelopes.len() {
            let a = root(parent, i);
            let b = root(parent, j);
            if a != b
                && !crate::geometry::intersection_polygons_paths(&envelopes[i], &envelopes[j])
                    .map_err(|_| SliceError::InvalidInput("skirt group geometry failed".into()))?
                    .is_empty()
            {
                parent[b] = a;
                changed = true;
            }
        }
    }
    Ok(changed)
}

fn root(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}
