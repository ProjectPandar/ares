//! `WipingExtrusions::mark_wiping_extrusions` (`ToolOrdering.cpp:1620–1755`).
use super::{Collection, Layer, Object, Region, WipingExtrusions, ordered, overridable};
use crate::ExtrusionRole;

struct Pass<'a> {
    layer: &'a Layer<'a>,
    tools: (usize, usize),
    perimeters_done: bool,
}

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
            let pass = Pass {
                layer,
                tools: (old, new),
                perimeters_done,
            };
            volume = self.mark_pass(&pass, &objects, volume);
            if volume <= 0.0 {
                return 0.0;
            }
        }
        volume
    }

    fn mark_pass(&mut self, pass: &Pass<'_>, objects: &[&Object], mut volume: f32) -> f32 {
        for &object in objects
            .iter()
            .filter(|object| pass.perimeters_done || object.flush_objects)
        {
            volume = self.mark_object(pass, object, volume);
            if volume <= 0.0 {
                return 0.0;
            }
        }
        volume
    }

    fn mark_object(&mut self, pass: &Pass<'_>, object: &Object, mut volume: f32) -> f32 {
        if object.regions.is_none() {
            return volume;
        }
        for copy in 0..object.copies {
            volume = self.mark_copy(pass, object, copy, volume);
            if volume <= 0.0 {
                return 0.0;
            }
        }
        volume
    }

    fn mark_copy(&mut self, pass: &Pass<'_>, object: &Object, copy: usize, mut volume: f32) -> f32 {
        let regions = object.regions.as_ref().unwrap();
        for collection in regions
            .iter()
            .flat_map(|region| collections(region, object, pass))
        {
            if !overridable(collection, object, pass.layer)
                || self.entity(object.id, collection.id, copy).is_some()
            {
                continue;
            }
            self.assign(object, collection.id, copy, pass.tools.1);
            volume -= collection.volume as f32;
            if volume <= 0.0 {
                return 0.0;
            }
        }
        self.mark_support(pass, object, volume)
    }

    fn mark_support(&mut self, pass: &Pass<'_>, object: &Object, mut volume: f32) -> f32 {
        if !object.flush_support {
            return volume;
        }
        let Some(support) = &object.support else {
            return volume;
        };
        let (old, new) = pass.tools;
        let body = object.support_tools[0] == 0;
        let interface = object.support_tools[1] == 0;
        let excluded = object.support_interface_not_for_body
            && !interface
            && (new == object.support_tools[1] - 1 || old == object.support_tools[1] - 1);
        // These two source maps are per object, not per copy.
        if body && !self.support.contains_key(&object.id) && !excluded {
            self.support.insert(object.id, new);
            volume = support_volume(support, volume, false);
            if volume <= 0.0 {
                return 0.0;
            }
        }
        if interface && !self.interface.contains_key(&object.id) {
            self.interface.insert(object.id, new);
            volume = support_volume(support, volume, true);
        }
        volume
    }
}

fn collections<'a>(
    region: &'a Region,
    object: &Object,
    pass: &Pass<'_>,
) -> impl Iterator<Item = &'a Collection> {
    let enabled = object.flush_infill || object.flush_objects || object.flush_support;
    let infill_only = !object.flush_objects && object.flush_infill;
    let infill = enabled
        && (region.infill_first != pass.perimeters_done || infill_only)
        && (!infill_only
            || region.infill_first
            || ordered(pass.layer.tools, region.wall_tool, pass.tools.1));
    let perimeter = enabled && object.flush_objects && region.infill_first == pass.perimeters_done;
    region
        .infills
        .iter()
        .filter(move |_| infill)
        .chain(region.perimeters.iter().filter(move |_| perimeter))
        .filter(|collection| collection.volume > 0.0)
}

fn support_volume(support: &[(ExtrusionRole, f64)], mut volume: f32, interface: bool) -> f32 {
    for &(role, amount) in support {
        let consume = match role {
            ExtrusionRole::SupportMaterial | ExtrusionRole::SupportTransition => !interface,
            ExtrusionRole::SupportMaterialInterface => interface,
            _ => false,
        };
        if consume {
            volume = (f64::from(volume) - amount) as f32;
        }
        if volume <= 0.0 {
            return 0.0;
        }
    }
    volume
}
