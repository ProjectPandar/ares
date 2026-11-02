use super::{
    super::onion::RawShellDepth,
    types::{LoopBuckets, PerimeterGeneratorLoop},
};
use crate::geometry::ExPolygon;

pub(super) fn materialize(
    effective_loop_number: i32,
    shells: &[RawShellDepth],
    thin_wall_holes: &[bool],
    narrow_wall_holes: &[bool],
) -> LoopBuckets {
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
    let mut hole_flags = thin_wall_holes
        .iter()
        .copied()
        .zip(narrow_wall_holes.iter().copied());
    for shell in shells {
        append_expolygons(
            &mut buckets,
            shell.depth,
            &shell.normal,
            false,
            &mut hole_flags,
        );
        append_expolygons(
            &mut buckets,
            shell.depth,
            &shell.smaller_width,
            true,
            &mut hole_flags,
        );
    }
    buckets
}

fn append_expolygons(
    buckets: &mut LoopBuckets,
    depth: i32,
    expolygons: &[ExPolygon],
    smaller: bool,
    hole_flags: &mut impl Iterator<Item = (bool, bool)>,
) {
    let index = depth as usize;
    for expolygon in expolygons {
        buckets.contours[index].push(PerimeterGeneratorLoop {
            polygon: expolygon.contour().clone(),
            is_contour: true,
            is_thin_wall_hole: false,
            is_narrow_wall_hole: false,
            is_smaller_width_perimeter: smaller,
            depth: depth as u16,
            children: Vec::new(),
        });
        buckets.holes[index].extend(expolygon.holes().iter().cloned().map(|polygon| {
            let (is_thin_wall_hole, is_narrow_wall_hole) = if depth == 0 {
                hole_flags.next().unwrap()
            } else {
                (false, false)
            };
            PerimeterGeneratorLoop {
                polygon,
                is_contour: false,
                is_thin_wall_hole,
                is_narrow_wall_hole,
                is_smaller_width_perimeter: smaller,
                depth: depth as u16,
                children: Vec::new(),
            }
        }));
    }
}
