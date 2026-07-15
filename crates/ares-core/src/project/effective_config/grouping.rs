use std::cmp::Ordering;

use crate::{ProjectObject, project::transform::Transform3d};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProjectObjectTransformGroups {
    pub(crate) source_object_index: usize,
    pub(crate) transforms: Vec<Transform3d>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GroupedPrintObjects {
    pub(crate) by_object: Vec<ProjectObjectTransformGroups>,
    pub(crate) effective_print_object_count: usize,
}

pub(crate) fn group_print_object_transforms(objects: &[ProjectObject]) -> GroupedPrintObjects {
    let mut effective_print_object_count = 0;
    let mut by_object = Vec::with_capacity(objects.len());

    for (source_object_index, object) in objects.iter().enumerate() {
        let mut transforms = object
            .instances()
            .iter()
            .filter(|instance| instance.printable())
            .map(|instance| instance.transform().without_xy_translation())
            .collect::<Vec<_>>();
        transforms.sort_by(|left, right| {
            if left.fixed_order_less_than(*right) {
                Ordering::Less
            } else if right.fixed_order_less_than(*left) {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        transforms.dedup_by(|left, right| left.fixed_order_equal(*right));
        effective_print_object_count += transforms.len();
        by_object.push(ProjectObjectTransformGroups {
            source_object_index,
            transforms,
        });
    }

    GroupedPrintObjects {
        by_object,
        effective_print_object_count,
    }
}
