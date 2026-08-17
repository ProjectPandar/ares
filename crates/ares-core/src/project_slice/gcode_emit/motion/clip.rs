pub(super) fn clip_end(points: &mut Vec<(i64, i64)>, distance: f64) {
    if distance <= 0.0 {
        return;
    }
    let mut remaining = distance;
    while points.len() > 1 {
        let last = points[points.len() - 1];
        let previous = points[points.len() - 2];
        let dx = (previous.0 - last.0) as f64;
        let dy = (previous.1 - last.1) as f64;
        let segment_length = dx.hypot(dy);
        if segment_length > remaining {
            let ratio = remaining / segment_length;
            let endpoint = (
                (last.0 as f64 + dx * ratio) as i64,
                (last.1 as f64 + dy * ratio) as i64,
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
        let mut points = vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 0)];

        super::clip_end(&mut points, 1.0);

        assert_eq!(points, vec![(0, 0), (4, 0), (4, 4), (0, 4), (0, 1)]);
    }

    #[test]
    fn clips_across_short_terminal_segments() {
        let mut points = vec![(0, 0), (2, 0), (2, 2), (0, 2), (0, 0)];

        super::clip_end(&mut points, 3.0);

        assert_eq!(points, vec![(0, 0), (2, 0), (2, 2), (1, 2)]);
    }
}
