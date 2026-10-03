//! Zero-loop per-object occupied groups (`Print.cpp:2785–2905`).

use crate::{
    SliceError,
    geometry::{JoinType, Point, Polygon, intersection_polygons_paths, offset_paths},
};

pub(super) fn merge(
    hulls: Vec<Vec<Point>>,
    distance: f32,
    arc_tolerance: f64,
) -> Result<Vec<Polygon>, SliceError> {
    let mut hulls = hulls.into_iter().map(Polygon::new).collect::<Vec<_>>();
    loop {
        let envelopes = hulls
            .iter()
            .map(|hull| {
                offset_paths(
                    std::slice::from_ref(hull),
                    distance,
                    JoinType::Round,
                    arc_tolerance,
                )
                .map(|polygons| polygons.into_iter().next())
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(geometry_error)?;
        let mut parent = (0..hulls.len()).collect::<Vec<_>>();
        if !merge_collisions(&envelopes, &mut parent)? {
            return Ok(hulls);
        }
        let mut groups = vec![Vec::new(); hulls.len()];
        for (i, hull) in hulls.into_iter().enumerate() {
            groups[root(&mut parent, i)].extend(hull.into_points());
        }
        hulls = groups
            .into_iter()
            .filter(|points| !points.is_empty())
            .map(|points| Polygon::new(super::convex_hull(&points)))
            .collect();
    }
}

fn merge_collisions(
    envelopes: &[Option<Polygon>],
    parent: &mut [usize],
) -> Result<bool, SliceError> {
    let mut changed = false;
    for i in 0..envelopes.len() {
        for j in i + 1..envelopes.len() {
            let root_i = root(parent, i);
            let root_j = root(parent, j);
            if root_i == root_j {
                continue;
            }
            let (Some(a), Some(b)) = (&envelopes[i], &envelopes[j]) else {
                continue;
            };
            if !intersection_polygons_paths(std::slice::from_ref(a), std::slice::from_ref(b))
                .map_err(geometry_error)?
                .is_empty()
            {
                parent[root_j] = root_i;
                changed = true;
            }
        }
    }
    Ok(changed)
}

fn root(parent: &mut [usize], mut index: usize) -> usize {
    while parent[index] != index {
        parent[index] = parent[parent[index]];
        index = parent[index];
    }
    index
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("skirt group geometry failed".to_owned())
}
