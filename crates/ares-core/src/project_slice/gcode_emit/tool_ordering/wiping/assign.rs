//! `WipingExtrusions::mark_wiping_extrusions` (`ToolOrdering.cpp:1620–1755`).
use super::{Layer, WipingExtrusions, ordered, overridable};
use crate::ExtrusionRole;

impl WipingExtrusions {
    pub(in crate::project_slice::gcode_emit) fn mark(
        &mut self,
        layer: &Layer<'_>,
        (old, new): (usize, usize),
        mut volume: f32,
    ) -> f32 {
        if !self.something_overridable
            || volume <= 0.0
            || layer.soluble[old]
            || layer.soluble[new]
            || layer.support_filaments[old]
            || layer.support_filaments[new]
        {
            return volume.max(0.0);
        }
        let mut objects = layer.objects.iter().collect::<Vec<_>>();
        objects.sort_unstable_by_key(|object| (!object.flush_objects, object.id));
        for perimeters_done in [false, true] {
            for object in &objects {
                if !perimeters_done && !object.flush_objects {
                    continue;
                }
                let Some(regions) = &object.regions else {
                    continue;
                };
                for copy in 0..object.copies {
                    for region in regions {
                        if !object.flush_infill && !object.flush_objects && !object.flush_support {
                            continue;
                        }
                        let infill_only = !object.flush_objects && object.flush_infill;
                        if region.infill_first != perimeters_done || infill_only {
                            for collection in &region.infills {
                                if !overridable(collection, object, layer) {
                                    continue;
                                }
                                if infill_only
                                    && !region.infill_first
                                    && !ordered(layer.tools, region.wall_tool, new)
                                {
                                    continue;
                                }
                                if self.entity(object.id, collection.id, copy).is_none()
                                    && collection.volume > 0.0
                                {
                                    self.assign(object, collection.id, copy, new);
                                    volume -= collection.volume as f32;
                                    if volume <= 0.0 {
                                        return 0.0;
                                    }
                                }
                            }
                        }
                        if object.flush_objects && region.infill_first == perimeters_done {
                            for collection in &region.perimeters {
                                if overridable(collection, object, layer)
                                    && self.entity(object.id, collection.id, copy).is_none()
                                    && collection.volume > 0.0
                                {
                                    self.assign(object, collection.id, copy, new);
                                    volume -= collection.volume as f32;
                                    if volume <= 0.0 {
                                        return 0.0;
                                    }
                                }
                            }
                        }
                    }
                    if object.flush_support
                        && let Some(support) = &object.support
                    {
                        let body = object.support_tools[0] == 0;
                        let interface = object.support_tools[1] == 0;
                        let excluded = object.support_interface_not_for_body
                            && !interface
                            && (new == object.support_tools[1] - 1
                                || old == object.support_tools[1] - 1);
                        // These two source maps are per object, not per copy.
                        if body && !self.support.contains_key(&object.id) && !excluded {
                            self.support.insert(object.id, new);
                            for &(role, amount) in support {
                                if matches!(
                                    role,
                                    ExtrusionRole::SupportMaterial
                                        | ExtrusionRole::SupportTransition
                                ) {
                                    volume = (f64::from(volume) - amount) as f32;
                                }
                                if volume <= 0.0 {
                                    return 0.0;
                                }
                            }
                        }
                        if interface && !self.interface.contains_key(&object.id) {
                            self.interface.insert(object.id, new);
                            for &(role, amount) in support {
                                if role == ExtrusionRole::SupportMaterialInterface {
                                    volume = (f64::from(volume) - amount) as f32;
                                }
                                if volume <= 0.0 {
                                    return 0.0;
                                }
                            }
                        }
                    }
                }
            }
        }
        volume
    }
}
