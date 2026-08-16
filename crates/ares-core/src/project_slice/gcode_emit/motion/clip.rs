pub(super) fn clip_end(points: &mut Vec<(f64, f64)>, distance: f64) {
    if distance <= 0.0 {
        return;
    }
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points[points.len() - 1];
        let previous = points[points.len() - 2];
        let segment_length = (last.0 - previous.0).hypot(last.1 - previous.1);
        if segment_length > remaining {
            let ratio = remaining / segment_length;
            let endpoint = (
                last.0 + (previous.0 - last.0) * ratio,
                last.1 + (previous.1 - last.1) * ratio,
            );
            *points.last_mut().expect("the path has an endpoint") = endpoint;
            return;
        }
        points.pop();
        remaining -= segment_length;
        if remaining <= f64::EPSILON {
            return;
        }
    }
    points.clear();
}

#[cfg(test)]
mod tests {
    #[test]
    fn clips_a_closed_loop_from_its_end() {
        let mut points = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)];

        super::clip_end(&mut points, 0.25);

        assert_eq!(
            points,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.25)]
        );
    }

    #[test]
    fn clips_across_short_terminal_segments() {
        let mut points = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)];

        super::clip_end(&mut points, 1.5);

        assert_eq!(points, vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.5, 1.0)]);
    }
}
