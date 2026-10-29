//! MultiMaterialSegmentation.cpp:805–956,1942–1953.

use super::painted_lines::{ColoredLine, PaintedLine, colorize, segments};
use crate::geometry::{CoordinateScale, Line, Point};

fn segment_length(lines: &[ColoredLine], (first, last): (usize, usize)) -> f64 {
    let mut length = 0.0;
    let mut index = first;
    while index != last {
        length += lines[index].line.length();
        index = (index + 1) % lines.len();
    }
    length + lines[last].line.length()
}

fn recolor(lines: &mut [ColoredLine], (first, last): (usize, usize), color: usize) {
    let mut index = first;
    while index != last {
        lines[index].color = color;
        index = (index + 1) % lines.len();
    }
    lines[last].color = color;
}

fn filter(lines: &mut [ColoredLine], [one, two, five]: [f64; 3]) {
    for i in 2..lines.len() {
        let color = lines[i - 2].color;
        if color == lines[i].color
            && color != lines[i - 1].color
            && color >= 1
            && lines[i - 1].line.length() <= five
        {
            lines[i - 1].color = color;
        }
    }
    for i in 3..lines.len() {
        let color = lines[i - 3].color;
        if color == lines[i].color
            && color >= 1
            && (color != lines[i - 2].color || color != lines[i - 1].color)
            && lines[i - 2].line.length() + lines[i - 1].line.length() <= five
        {
            lines[i - 2].color = color;
            lines[i - 1].color = color;
        }
    }
    let runs = segments(lines);
    if runs.len() >= 2 {
        for i in 0..runs.len() {
            let next = (i + 1) % runs.len();
            let color = lines[runs[i].0].color;
            if color != lines[runs[next].0].color
                && segment_length(lines, runs[i]) >= one
                && segment_length(lines, runs[next]) <= two
            {
                recolor(lines, runs[next], color);
            }
        }
    }
    let runs = segments(lines);
    if runs.len() >= 2 {
        for i in 0..runs.len() {
            let next = (i + 1) % runs.len();
            let color = lines[runs[i].0].color;
            if color >= 1
                && color != lines[runs[next].0].color
                && segment_length(lines, runs[next]) <= two
            {
                recolor(lines, runs[next], color);
            }
        }
    }
    let runs = segments(lines);
    if runs.len() >= 3 {
        for i in 0..runs.len() {
            let next = (i + 1) % runs.len();
            let after = (next + 1) % runs.len();
            let color = lines[runs[i].0].color;
            if color > 0
                && color == lines[runs[after].0].color
                && color != lines[runs[next].0].color
                && segment_length(lines, runs[next]) <= five
            {
                // Upstream deliberately writes next_next, not the middle run.
                recolor(lines, runs[after], color);
            }
        }
    }
}

fn contour(points: &[Point], input: &[PaintedLine], scale: CoordinateScale) -> Vec<ColoredLine> {
    let limits = [0.1, 0.2, 0.5].map(|mm| scale.scaled_delta(mm));
    let epsilon = scale.scaled_delta(f64::from(0.1_f32)) as i64 as f64;
    let make = |index| ColoredLine {
        line: Line::new(points[index], points[(index + 1) % points.len()]),
        color: 0,
        contour: 0,
        segment: 0,
    };
    if input.is_empty() {
        return (0..points.len()).map(make).collect();
    }
    let mut output = (0..input[0].segment).map(make).collect::<Vec<_>>();
    let mut first = 0;
    while first < input.len() {
        let segment = input[first].segment;
        let end = first + input[first..].partition_point(|line| line.segment == segment);
        output.extend(colorize(
            make(segment).line,
            &input[first..end],
            (epsilon, limits[1]),
        ));
        if end < input.len() {
            output.extend((segment + 1..input[end].segment).map(make));
        }
        first = end;
    }
    output.extend((input.last().unwrap().segment + 1..points.len()).map(make));
    filter(&mut output, limits);
    output
}

pub(super) fn contours(
    points: &[Vec<Point>],
    painted: &[Vec<PaintedLine>],
    scale: CoordinateScale,
) -> Vec<Vec<ColoredLine>> {
    points
        .iter()
        .zip(painted)
        .enumerate()
        .map(|(contour_index, (points, painted))| {
            let mut lines = contour(points, painted, scale);
            for (segment, line) in lines.iter_mut().enumerate() {
                line.contour = contour_index;
                line.segment = segment;
            }
            lines
        })
        .collect()
}

pub(super) fn single_color(contours: &[Vec<ColoredLine>]) -> bool {
    let color = contours[0][0].color;
    contours.iter().flatten().all(|line| line.color == color)
}
