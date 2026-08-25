use crate::{
    arachne::extrusion_line::{ExtrusionJunction, ExtrusionLine},
    geometry::Point,
};

use super::super::SkeletalTrapezoidation;

impl SkeletalTrapezoidation<'_> {
    pub(super) fn generate_local_maxima_single_beads(&mut self) {
        let nodes = self.graph.active_nodes().collect::<Vec<_>>();
        for node in nodes {
            let Some(beading_storage) = self.graph.node(node).data.beading() else {
                continue;
            };
            let beading = beading_storage.borrow();
            if beading.beading.bead_widths.len() % 2 == 0
                || !self.graph.node_is_local_maximum(node, true)
                || self.graph.node_is_central(node)
            {
                continue;
            }
            let inset_index = beading.beading.bead_widths.len() / 2;
            let width = beading.beading.bead_widths[inset_index];
            drop(beading);
            if inset_index >= self.generated_toolpaths.len() {
                self.generated_toolpaths
                    .resize_with(inset_index + 1, Vec::new);
            }
            let mut line = ExtrusionLine::new(inset_index, true);
            let center = self.graph.node(node).point;
            let radius = width / 8;
            for segment in 0..6 {
                let angle = (2.0 * std::f64::consts::PI / 6.0 * segment as f64) as f32;
                let angle = f64::from(angle);
                let offset = Point::new(
                    (radius as f64 * angle.cos()).round() as i64,
                    (radius as f64 * angle.sin()).round() as i64,
                );
                line.push(ExtrusionJunction::new(
                    Point::new(center.x() + offset.x(), center.y() + offset.y()),
                    width,
                    inset_index,
                ));
            }
            self.generated_toolpaths[inset_index].push(line);
        }
    }
}

#[cfg(test)]
mod tests;
