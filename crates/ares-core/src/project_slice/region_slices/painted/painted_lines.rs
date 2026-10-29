//! MultiMaterialSegmentation.cpp:514–520,614–803 (painted input lines).

use crate::geometry::{Line, Point, fixed_gcc_sort_by};

#[derive(Clone, Copy)]
pub(super) struct PaintedLine {
    pub(super) contour: usize,
    pub(super) segment: usize,
    pub(super) line: Line,
    pub(super) color: usize,
}

#[derive(Clone, Copy)]
pub(super) struct ColoredLine {
    pub(super) line: Line,
    pub(super) color: usize,
    pub(super) contour: usize,
    pub(super) segment: usize,
}

pub(super) fn distance2(a: Point, b: Point) -> f64 {
    let x = (a.x() - b.x()) as f64;
    let y = (a.y() - b.y()) as f64;
    x * x + y * y
}

pub(super) fn segments(lines: &[ColoredLine]) -> Vec<(usize, usize)> {
    let count = lines.len();
    let mut end = 0;
    while end + 1 < count && lines[end].color == lines[end + 1].color {
        end += 1;
    }
    if end == count - 1 {
        return vec![(0, count - 1)];
    }
    let first = (end + 1) % count;
    let mut offset = 0;
    let mut output = Vec::new();
    while offset < count {
        let start = (first + offset) % count;
        let mut end = start;
        while offset + 1 < count && lines[start].color == lines[(first + offset + 1) % count].color
        {
            end = (first + offset + 1) % count;
            offset += 1;
        }
        output.push((start, end));
        offset += 1;
    }
    output
}

fn filter(line: Line, input: &[PaintedLine], epsilon: f64) -> Vec<PaintedLine> {
    let mut output = vec![input[0]];
    for &current in &input[1..] {
        let previous = output.last_mut().unwrap();
        if previous.line.b == line.b {
            break;
        }
        let length = previous.line.length();
        let start_distance = distance2(current.line.a, previous.line.a).sqrt();
        let gap = start_distance - length;
        if gap >= 0.0 {
            if previous.color == current.color && gap <= epsilon {
                previous.line.b = current.line.b;
            } else {
                output.push(current);
            }
        } else if distance2(current.line.b, previous.line.a).sqrt() > length {
            if previous.color == current.color {
                previous.line.b = current.line.b;
            } else {
                let end = previous.line.b;
                output.push(PaintedLine {
                    line: Line::new(end, current.line.b),
                    ..current
                });
            }
        }
    }
    if distance2(output[0].line.a, line.a).sqrt() <= epsilon {
        output[0].line.a = line.a;
    }
    let last = output.last_mut().unwrap();
    if distance2(last.line.b, line.b).sqrt() <= epsilon {
        last.line.b = line.b;
    }
    output
}

pub(super) fn post_process(
    contours: &[Vec<Point>],
    mut input: Vec<PaintedLine>,
    epsilon: f64,
) -> Vec<Vec<PaintedLine>> {
    if input.is_empty() {
        return Vec::new();
    }
    fixed_gcc_sort_by(&mut input, |a, b| {
        let origin = contours[a.contour][a.segment];
        a.contour < b.contour
            || (a.contour == b.contour
                && (a.segment < b.segment
                    || (a.segment == b.segment
                        && (distance2(a.line.a, origin) < distance2(b.line.a, origin)
                            || (distance2(a.line.a, origin) == distance2(b.line.a, origin)
                                && distance2(a.line.a, a.line.b)
                                    < distance2(b.line.a, b.line.b))))))
    });
    let mut output = vec![Vec::new(); contours.len()];
    let mut first = 0;
    while first < input.len() {
        let key = input[first];
        let last = first
            + input[first..]
                .partition_point(|line| line.contour == key.contour && line.segment == key.segment);
        let contour = &contours[key.contour];
        let line = Line::new(
            contour[key.segment],
            contour[(key.segment + 1) % contour.len()],
        );
        output[key.contour].extend(filter(line, &input[first..last], epsilon));
        first = last;
    }
    output
}

pub(super) fn colorize(
    line: Line,
    input: &[PaintedLine],
    (epsilon, short): (f64, f64),
) -> Vec<ColoredLine> {
    let first = input[0];
    let make = |line, color| ColoredLine {
        line,
        color,
        contour: first.contour,
        segment: first.segment,
    };
    let mut output = Vec::new();
    if distance2(first.line.a, line.a).sqrt() > epsilon {
        output.push(make(Line::new(line.a, first.line.a), 0));
    }
    output.push(make(first.line, first.color));
    for current in &input[1..] {
        let previous = output.last_mut().unwrap();
        let gap = distance2(current.line.a, previous.line.b).sqrt();
        if gap <= epsilon {
            if previous.color == current.color {
                previous.line.b = current.line.b;
            } else {
                previous.line.b = current.line.a;
                output.push(make(current.line, current.color));
            }
        } else {
            let end = previous.line.b;
            output.push(make(Line::new(end, current.line.a), 0));
            output.push(make(current.line, current.color));
        }
    }
    let end = output.last().unwrap().line.b;
    if distance2(end, line.b).sqrt() > epsilon {
        output.push(make(Line::new(end, line.b), 0));
    }
    for i in 2..output.len() {
        if output[i - 2].color == output[i].color
            && output[i - 2].color != output[i - 1].color
            && output[i - 1].line.length() <= short
        {
            output[i - 1].color = output[i - 2].color;
        }
    }
    let mut merged = vec![output[0]];
    for line in output.into_iter().skip(1) {
        let previous = merged.last_mut().unwrap();
        if previous.color == line.color {
            previous.line.b = line.line.b;
        } else {
            merged.push(line);
        }
    }
    if merged.len() > 1 && merged[0].color != merged[1].color && merged[0].line.length() <= short {
        merged[1].line.a = merged[0].line.a;
        merged.remove(0);
    }
    let n = merged.len();
    if n > 1 && merged[n - 1].color != merged[n - 2].color && merged[n - 1].line.length() <= short {
        merged[n - 2].line.b = merged[n - 1].line.b;
        merged.pop();
    }
    merged
}
