use std::cmp::Ordering;

use crate::{Point3d, ProjectObject, project::transform::Transform3d};

use super::types::ResolvedPrintObjectConfig;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ProjectObjectTransformGroups {
    pub(crate) source_object_index: usize,
    pub(crate) groups: Vec<ResolvedPrintObjectConfig>,
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

pub(crate) fn group_print_object_transforms(
    objects: &[ProjectObject],
    belt_shrinkage: Option<f64>,
) -> GroupedPrintObjects {
    let mut effective_print_object_count = 0;
    let mut by_object = Vec::with_capacity(objects.len());

    for (source_object_index, object) in objects.iter().enumerate() {
        let mut instances = object
            .instances()
            .iter()
            .enumerate()
            .filter(|(_, instance)| instance.is_printable())
            .enumerate()
            .map(|(ordinal, (index, instance))| {
                let transform = instance.transform().without_xy_translation();
                // PrintApply.cpp:200–218 separates belt copies and retains bed-drop Z.
                let transform = belt_shrinkage.map_or(transform, |factor| {
                    transform
                        .with_z_shrinkage_compensation(factor)
                        .pretranslated(Point3d::new(0., 0., 1e-10 * (ordinal + 1) as f64))
                });
                (transform, index)
            })
            .collect::<Vec<_>>();
        instances.sort_by(|left, right| transform_order(&left.0, &right.0));
        let mut groups: Vec<ResolvedPrintObjectConfig> = Vec::new();
        for (transform, index) in instances {
            if belt_shrinkage.is_none()
                && let Some(last) = groups.last_mut()
                && last.transform.fixed_order_equal(transform)
            {
                last.instance_indices.push(index);
            } else {
                groups.push(ResolvedPrintObjectConfig {
                    transform,
                    instance_indices: vec![index],
                });
            }
        }
        effective_print_object_count += groups.len();
        by_object.push(ProjectObjectTransformGroups {
            source_object_index,
            groups,
        });
    }

    GroupedPrintObjects {
        by_object,
        effective_print_object_count,
    }
}
