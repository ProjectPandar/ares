//! Reachable `WipeTowerWriter2::generate_path` linear branch.
// The source constructor leaves m_enable_arc_fitting=false (cpp 577,1169).

use super::Writer;
use crate::geometry::{CoordinateScale, Polyline};

impl Writer {
    pub(in super::super) fn generate_path(
        &mut self,
        paths: &[Polyline],
        feedrate: f32,
        retract: (f32, f32),
        scale: CoordinateScale,
    ) {
        let factor = scale.factor() as f32;
        let segments = paths
            .iter()
            .flat_map(|path| path.points().windows(2))
            .map(|pair| {
                [pair[0], pair[1]]
                    .map(|point| [point.x() as f32 * factor, point.y() as f32 * factor])
            })
            .collect::<Vec<_>>();
        let nearest = segments
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let distance = |p: [f32; 2]| {
                    let x = p[0] - self.position[0];
                    let y = p[1] - self.position[1];
                    x * x + y * y
                };
                distance(a[0]).total_cmp(&distance(b[0]))
            })
            .expect("tower wall has segments")
            .0;
        self.travel(segments[nearest][0], 0.0);
        self.extrude(segments[nearest][1], feedrate);
        for step in 1..segments.len() {
            let [start, end] = segments[(nearest + step) % segments.len()];
            let dx = start[0] - self.position[0];
            let dy = start[1] - self.position[1];
            if (dx * dx + dy * dy).sqrt() > 1e-4 {
                self.load(-retract.0, retract.1);
                self.travel(start, 600.0);
                self.load(retract.0, retract.1);
            }
            self.extrude(end, feedrate);
        }
    }
}
