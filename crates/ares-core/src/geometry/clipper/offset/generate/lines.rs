use crate::geometry::{Point, Polygon};

use super::{
    Corner, JoinType, PreparedOffset, UnitNormal, do_round, generate_corner,
    offset_point_with_normal, unit_normal,
};

pub(super) fn generate_open(
    source: &[Point],
    join_type: JoinType,
    round_caps: bool,
    prepared: &PreparedOffset,
) -> Polygon {
    let mut normals = source
        .windows(2)
        .map(|edge| unit_normal(edge[0], edge[1]))
        .collect::<Vec<_>>();
    normals.push(normals[normals.len() - 1]);

    let mut output = Vec::new();
    let mut previous = 0;
    for current in 1..source.len() - 1 {
        let corner = Corner::new(source[current], normals[previous], normals[current]);
        if generate_corner(corner, join_type, prepared, &mut output) {
            previous = current;
        }
    }
    let end = source.len() - 1;
    if round_caps {
        let previous = end - 1;
        normals[end] = negated(normals[end]);
        do_round(
            Corner::new(source[end], normals[previous], normals[end]),
            prepared,
            &mut output,
        );
    } else {
        output.push(offset_point_with_normal(
            source[end],
            normals[end],
            prepared.delta,
        ));
        output.push(offset_point_with_normal(
            source[end],
            negated(normals[end]),
            prepared.delta,
        ));
    }

    for index in (1..normals.len()).rev() {
        normals[index] = negated(normals[index - 1]);
    }
    normals[0] = negated(normals[1]);
    previous = end;
    for current in (1..end).rev() {
        let corner = Corner::new(source[current], normals[previous], normals[current]);
        if generate_corner(corner, join_type, prepared, &mut output) {
            previous = current;
        }
    }
    if round_caps {
        do_round(
            Corner::new(source[0], normals[1], normals[0]),
            prepared,
            &mut output,
        );
    } else {
        output.push(offset_point_with_normal(
            source[0],
            negated(normals[0]),
            prepared.delta,
        ));
        output.push(offset_point_with_normal(
            source[0],
            normals[0],
            prepared.delta,
        ));
    }
    Polygon::new(output)
}

pub(super) fn generate_closed_line(
    source: &[Point],
    join_type: JoinType,
    prepared: &PreparedOffset,
) -> [Polygon; 2] {
    let mut normals = Vec::with_capacity(source.len());
    for edge in source.windows(2) {
        normals.push(unit_normal(edge[0], edge[1]));
    }
    normals.push(unit_normal(source[source.len() - 1], source[0]));

    let mut first = Vec::new();
    let mut previous = source.len() - 1;
    for current in 0..source.len() {
        let corner = Corner::new(source[current], normals[previous], normals[current]);
        if generate_corner(corner, join_type, prepared, &mut first) {
            previous = current;
        }
    }

    let last_normal = normals[source.len() - 1];
    for index in (1..source.len()).rev() {
        normals[index] = negated(normals[index - 1]);
    }
    normals[0] = negated(last_normal);

    let mut second = Vec::new();
    previous = 0;
    for current in (0..source.len()).rev() {
        let corner = Corner::new(source[current], normals[previous], normals[current]);
        if generate_corner(corner, join_type, prepared, &mut second) {
            previous = current;
        }
    }
    [Polygon::new(first), Polygon::new(second)]
}

fn negated(normal: UnitNormal) -> UnitNormal {
    UnitNormal {
        x: -normal.x,
        y: -normal.y,
    }
}
