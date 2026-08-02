use super::{
    super::onion::RawShellDepth,
    types::{LoopBuckets, PerimeterGeneratorLoop},
};
use crate::geometry::ExPolygon;

pub(super) fn materialize(effective_loop_number: i32, shells: &[RawShellDepth]) -> LoopBuckets {
    if effective_loop_number < 0 {
        return LoopBuckets {
            contours: Vec::new(),
            holes: Vec::new(),
        };
    }
    let count = effective_loop_number as usize + 1;
    let mut buckets = LoopBuckets {
        contours: vec![Vec::new(); count],
        holes: vec![Vec::new(); count],
    };
    for shell in shells {
        append_expolygons(&mut buckets, shell.depth, &shell.normal, false);
        append_expolygons(&mut buckets, shell.depth, &shell.smaller_width, true);
    }
    buckets
}

fn append_expolygons(
    buckets: &mut LoopBuckets,
    depth: i32,
    expolygons: &[ExPolygon],
    smaller: bool,
) {
    let index = depth as usize;
    for expolygon in expolygons {
        buckets.contours[index].push(PerimeterGeneratorLoop {
            polygon: expolygon.contour().clone(),
            is_contour: true,
            is_smaller_width_perimeter: smaller,
            depth: depth as u16,
            children: Vec::new(),
        });
        buckets.holes[index].extend(expolygon.holes().iter().cloned().map(|polygon| {
            PerimeterGeneratorLoop {
                polygon,
                is_contour: false,
                is_smaller_width_perimeter: smaller,
                depth: depth as u16,
                children: Vec::new(),
            }
        }));
    }
}

#[cfg(test)]
mod tests;
