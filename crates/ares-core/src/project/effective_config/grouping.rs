use std::cmp::Ordering;

use crate::{ProjectObject, project::transform::Transform3d};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProjectObjectTransformGroups {
    pub(crate) source_object_index: usize,
    pub(crate) transforms: Vec<Transform3d>,
}

impl ProjectObjectTransformGroups {
    /// `PrintApply.cpp::print_objects_from_model_object`: each group's members
    /// retain ModelObject instance order, independently of transform ordering.
    pub(crate) fn instance_indices(&self, object: &ProjectObject) -> Vec<Vec<usize>> {
        let mut members = vec![Vec::new(); self.transforms.len()];
        for (index, instance) in object
            .instances()
            .iter()
            .enumerate()
            .filter(|(_, instance)| instance.printable())
        {
            let transform = instance.transform().without_xy_translation();
            let group = self
                .transforms
                .binary_search_by(|candidate| transform_order(candidate, &transform))
                .expect("printable instance retains its transform group");
            members[group].push(index);
        }
        members
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GroupedPrintObjects {
    pub(crate) by_object: Vec<ProjectObjectTransformGroups>,
    pub(crate) effective_print_object_count: usize,
}

fn transform_order(left: &Transform3d, right: &Transform3d) -> Ordering {
    if left.fixed_order_less_than(*right) {
        Ordering::Less
    } else if right.fixed_order_less_than(*left) {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
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
        transforms.sort_by(transform_order);
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
