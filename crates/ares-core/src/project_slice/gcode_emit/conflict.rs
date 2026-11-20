//! `ConflictChecker.cpp/.hpp`: the activated Type2 `Print::process` consumer.
mod collect;
mod intersect;

use super::layers::Schedule;
use crate::{
    geometry::{Line, Point},
    project_slice::{
        island_print_order::OrderedExtrusionLayer,
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(super) struct Layer {
    bottom: f32,
    lines: Vec<(Line, bool)>,
}
impl Layer {
    pub(super) fn new(bottom: f32) -> Self {
        Self {
            bottom,
            lines: Vec::new(),
        }
    }
    pub(super) fn path(&mut self, points: impl Iterator<Item = Point>, support: bool) {
        let mut previous = None;
        for point in points {
            if let Some(start) = previous {
                self.lines.push((Line::new(start, point), support));
            }
            previous = Some(point);
        }
    }
}

struct Bucket {
    source: usize,
    owner: usize,
    shift: Point,
    index: usize,
    bottom: f32,
    queued: bool,
}
impl Bucket {
    fn new(source: usize, owner: usize, shift: Point) -> Self {
        Self {
            source,
            owner,
            shift,
            index: 0,
            bottom: 0.0,
            queued: true,
        }
    }
    fn raise(&mut self, layers: &[Layer]) {
        let previous = self.bottom;
        if let Some(layer) = layers.get(self.index) {
            let bottom = layer.bottom;
            while self.index < layers.len() && layers[self.index].bottom == bottom {
                self.index += 1;
            }
            self.bottom = layers
                .get(self.index)
                .unwrap_or_else(|| layers.last().unwrap())
                .bottom;
        }
        self.queued = self.bottom != previous && self.index < layers.len();
    }
}

pub(super) fn check<'a>(
    traversal: &'a PreparedPostClassicTraversal,
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &[Vec<InstalledSupportLayer>],
    schedule: &Schedule,
    check_without_tower: bool,
) -> Option<[&'a str; 2]> {
    let mut sources = Vec::new();
    let mut buckets = Vec::new();
    let mut names = Vec::new();
    if let Some(tower) = &schedule.tower {
        sources.push(tower.fake_layers(
            &traversal.resolved.views.full,
            traversal.scale,
            super::wipe_tower::plate_position(traversal),
        ));
        buckets.push(Bucket::new(0, 0, Point::new(0, 0)));
        names.push("WipeTower");
    } else if !check_without_tower || schedule.instances.iter().map(Vec::len).sum::<usize>() <= 1 {
        // ConflictChecker.cpp:241–246: no pair can conflict with fewer than two owners.
        return None;
    }
    for (index, copies) in schedule.instances.iter().enumerate() {
        let source = sources.len();
        sources.push(collect::object(&objects[index], &traversal.objects[index]));
        sources.push(collect::support(
            &support[index],
            schedule
                .raft
                .as_ref()
                .map(|raft| (raft.plans[index].as_slice(), raft.layers[index].as_slice())),
        ));
        let source_index = traversal.objects[index]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .identity()
            .0;
        for copy in copies {
            let owner = names.len();
            names.push(traversal.project.objects()[source_index].name());
            buckets.push(Bucket::new(source, owner, copy.shift));
            buckets.push(Bucket::new(source + 1, owner, copy.shift));
        }
    }
    let mut lines = Vec::new();
    loop {
        // Source getCurLines already scans every bucket: find the queue minimum
        // in that same pass, without cloning object geometry for every copy.
        lines.clear();
        let mut minimum = f32::MAX;
        for bucket in &buckets {
            if bucket.queued {
                minimum = minimum.min(bucket.bottom);
            }
            append_current(&mut lines, bucket, &sources[bucket.source]);
        }
        if minimum == f32::MAX {
            break;
        }
        if let Some((mut first, mut second)) = intersect::find(&lines, traversal.scale) {
            if schedule.tower.is_some() && second == 0 {
                std::mem::swap(&mut first, &mut second);
            }
            return Some([names[first], names[second]]);
        }
        for bucket in &mut buckets {
            if bucket.queued && f64::from((bucket.bottom - minimum).abs()) < 1e-4 {
                bucket.raise(&sources[bucket.source]);
            }
        }
    }
    None
}

fn append_current(output: &mut Vec<intersect::OwnedLine>, bucket: &Bucket, layers: &[Layer]) {
    let Some(current) = layers.get(bucket.index) else {
        return;
    };
    for layer in layers[bucket.index..]
        .iter()
        .take_while(|layer| layer.bottom == current.bottom)
    {
        for &(line, support) in &layer.lines {
            let shift = |point: Point| {
                Point::new(point.x() + bucket.shift.x(), point.y() + bucket.shift.y())
            };
            output.push(intersect::OwnedLine {
                line: Line::new(shift(line.a), shift(line.b)),
                owner: bucket.owner,
                support,
            });
        }
    }
}
