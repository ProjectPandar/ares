use crate::{arachne::extrusion_line::ExtrusionLine, geometry::Point};

pub(super) fn remove_small_lines(
    toolpaths: &mut [Vec<ExtrusionLine>],
    min_length_factor: f64,
    is_top_or_bottom_layer: bool,
) {
    for inset in toolpaths {
        let mut line_index = 0;
        while line_index < inset.len() {
            let line = &inset[line_index];
            let min_width = line
                .junctions
                .iter()
                .map(|junction| junction.width)
                .min()
                .unwrap();
            let threshold = if is_top_or_bottom_layer {
                min_width / 2
            } else {
                (min_width as f64 * min_length_factor) as i64
            };
            if line.is_odd && !line.is_closed && shorter_than(line, threshold) {
                inset.swap_remove(line_index);
            } else {
                line_index += 1;
            }
        }
    }
}

fn shorter_than(line: &ExtrusionLine, threshold: i64) -> bool {
    let mut previous = line.junctions.last().unwrap().point;
    let mut length = 0;
    for junction in &line.junctions {
        length += distance(previous, junction.point);
        if length >= threshold {
            return false;
        }
        previous = junction.point;
    }
    true
}

fn distance(left: Point, right: Point) -> i64 {
    let dx = i128::from(left.x() - right.x());
    let dy = i128::from(left.y() - right.y());
    ((dx * dx + dy * dy) as f64).sqrt() as i64
}

#[cfg(test)]
mod tests {
    use crate::{
        arachne::{ExtrusionJunction, ExtrusionLine},
        geometry::Point,
    };

    use super::remove_small_lines;

    fn odd_line(length: i64) -> ExtrusionLine {
        let mut line = ExtrusionLine::new(0, true);
        line.push(ExtrusionJunction::new(Point::new(0, 0), 100, 0));
        line.push(ExtrusionJunction::new(Point::new(length, 0), 100, 0));
        line
    }

    #[test]
    fn task22o197_removes_only_odd_open_line_below_width_factor() {
        let mut toolpaths = vec![vec![odd_line(20), odd_line(30)]];

        remove_small_lines(&mut toolpaths, 0.5, false);

        assert_eq!(toolpaths[0], vec![odd_line(30)]);
    }
}
