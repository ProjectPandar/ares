//! Print.cpp:2784-2900: stable membership and fixed-point skirt envelopes.

use super::*;

pub(super) struct Group {
    pub(super) members: Vec<usize>,
    pub(super) hull: Polygon,
}

pub(super) fn merge(
    items: Vec<(usize, Vec<Point>)>,
    distance: f32,
    tolerance: f64,
) -> Result<Vec<Group>, SliceError> {
    let mut groups = items
        .into_iter()
        .map(|(member, points)| Group {
            members: vec![member],
            hull: Polygon::new(points),
        })
        .collect::<Vec<_>>();
    loop {
        let envelopes = groups
            .iter()
            .map(|group| {
                offset_paths(
                    std::slice::from_ref(&group.hull),
                    distance,
                    JoinType::Round,
                    tolerance,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SliceError::InvalidInput("skirt group geometry failed".into()))?;
        let mut parent = (0..groups.len()).collect::<Vec<_>>();
        let changed = merge_collisions(&envelopes, &mut parent)?;
        if !changed {
            return Ok(groups);
        }
        let mut points = vec![Vec::new(); groups.len()];
        let mut members = vec![Vec::new(); groups.len()];
        for (i, group) in groups.into_iter().enumerate() {
            let owner = root(&mut parent, i);
            points[owner].extend(group.hull.into_points());
            members[owner].extend(group.members);
        }
        groups = points
            .into_iter()
            .zip(members)
            .filter(|(_, members)| !members.is_empty())
            .map(|(points, members)| Group {
                members,
                hull: Polygon::new(convex_hull(&points)),
            })
            .collect();
    }
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
