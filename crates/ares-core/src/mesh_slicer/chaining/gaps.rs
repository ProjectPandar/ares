use crate::geometry::{Coord, Polygon};

use super::{OpenPolyline, open_length, signed_area};
use spatial::{EndpointKey, EndpointSide, Nearest, RadiusGrid, distance_squared_inside};

mod spatial;

pub(super) fn chain_open_polylines_close_gaps(
    open_polylines: &mut [OpenPolyline],
    polygons: &mut Vec<Polygon>,
    max_gap_scaled: Coord,
    try_connect_reversed: bool,
) {
    let sorted_indices = sorted_gap_indices(open_polylines);
    let mut grid = RadiusGrid::new(max_gap_scaled);
    for &polyline_index in &sorted_indices {
        insert_polyline(
            &mut grid,
            polyline_index,
            &open_polylines[polyline_index],
            try_connect_reversed,
        );
    }

    for seed_index in sorted_indices {
        if open_polylines[seed_index].consumed {
            continue;
        }
        if try_connect_reversed {
            assert!(grid.remove(endpoint_key(seed_index, EndpointSide::End)));
        }
        open_polylines[seed_index].consumed = true;
        let mut segments_joined = 1;

        loop {
            let current_end = *open_polylines[seed_index].points.last().unwrap();
            let next = grid.find(current_end, |candidate| !open_polylines[candidate].consumed);
            let closing_distance_squared = distance_squared_inside(
                current_end,
                open_polylines[seed_index].points[0],
                max_gap_scaled,
            );

            if should_close(
                &open_polylines[seed_index].points,
                closing_distance_squared,
                next,
            ) {
                assert!(grid.remove(endpoint_key(seed_index, EndpointSide::Start)));
                close_seed(
                    &mut open_polylines[seed_index],
                    polygons,
                    closing_distance_squared.unwrap(),
                    try_connect_reversed,
                    segments_joined,
                );
                break;
            }

            let Some(next) = next else {
                restore_open_seed(open_polylines, &mut grid, seed_index, try_connect_reversed);
                break;
            };
            attach_candidate(
                open_polylines,
                &mut grid,
                seed_index,
                next.key,
                try_connect_reversed,
            );
            segments_joined += 1;
        }
    }
}

fn restore_open_seed(
    open_polylines: &mut [OpenPolyline],
    grid: &mut RadiusGrid,
    seed_index: usize,
    try_connect_reversed: bool,
) {
    open_polylines[seed_index].consumed = false;
    if try_connect_reversed {
        grid.insert(
            endpoint_key(seed_index, EndpointSide::End),
            *open_polylines[seed_index].points.last().unwrap(),
        );
    }
}

fn sorted_gap_indices(open_polylines: &mut [OpenPolyline]) -> Vec<usize> {
    for polyline in open_polylines
        .iter_mut()
        .filter(|polyline| !polyline.consumed)
    {
        polyline.length = open_length(&polyline.points);
    }
    let mut indices = open_polylines
        .iter()
        .enumerate()
        .filter_map(|(index, polyline)| (!polyline.consumed).then_some(index))
        .collect::<Vec<_>>();
    indices.sort_by(|&left, &right| {
        open_polylines[right]
            .length
            .total_cmp(&open_polylines[left].length)
            .then_with(|| left.cmp(&right))
    });
    indices
}

fn insert_polyline(
    grid: &mut RadiusGrid,
    polyline_index: usize,
    polyline: &OpenPolyline,
    try_connect_reversed: bool,
) {
    grid.insert(
        endpoint_key(polyline_index, EndpointSide::Start),
        polyline.points[0],
    );
    if try_connect_reversed {
        grid.insert(
            endpoint_key(polyline_index, EndpointSide::End),
            *polyline.points.last().unwrap(),
        );
    }
}

fn should_close(
    points: &[crate::geometry::Point],
    closing_distance_squared: Option<u128>,
    next: Option<Nearest>,
) -> bool {
    let Some(closing_distance_squared) = closing_distance_squared else {
        return false;
    };
    match next {
        Some(next) if closing_distance_squared < next.distance_squared => {
            (closing_distance_squared as f64).sqrt() < 0.3 * open_length(points)
        }
        _ => true,
    }
}

fn close_seed(
    seed: &mut OpenPolyline,
    polygons: &mut Vec<Polygon>,
    closing_distance_squared: u128,
    try_connect_reversed: bool,
    segments_joined: usize,
) {
    if closing_distance_squared == 0 {
        seed.points.pop();
    }
    if seed.points.len() >= 3 {
        if try_connect_reversed && segments_joined > 1 && signed_area(&seed.points) < 0.0 {
            seed.points.reverse();
        }
        polygons.push(Polygon::new(std::mem::take(&mut seed.points)));
    } else {
        seed.points.clear();
    }
    seed.consumed = true;
}

fn attach_candidate(
    open_polylines: &mut [OpenPolyline],
    grid: &mut RadiusGrid,
    seed_index: usize,
    candidate_key: EndpointKey,
    try_connect_reversed: bool,
) {
    let candidate_index = candidate_key.original_index;
    assert!(grid.remove(endpoint_key(candidate_index, EndpointSide::Start)));
    if try_connect_reversed {
        assert!(grid.remove(endpoint_key(candidate_index, EndpointSide::End)));
    }
    let candidate_points = std::mem::take(&mut open_polylines[candidate_index].points);
    open_polylines[candidate_index].consumed = true;

    let seed_points = &mut open_polylines[seed_index].points;
    let current_end = *seed_points.last().unwrap();
    match candidate_key.side {
        EndpointSide::Start => {
            let skip = usize::from(candidate_points[0] == current_end);
            seed_points.extend(candidate_points.into_iter().skip(skip));
        }
        EndpointSide::End => {
            let skip = usize::from(*candidate_points.last().unwrap() == current_end);
            seed_points.extend(candidate_points.into_iter().rev().skip(skip));
        }
    }
}

const fn endpoint_key(original_index: usize, side: EndpointSide) -> EndpointKey {
    EndpointKey {
        original_index,
        side,
    }
}

#[cfg(test)]
mod tests;
