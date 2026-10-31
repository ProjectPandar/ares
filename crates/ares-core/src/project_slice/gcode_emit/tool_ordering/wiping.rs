//! `ToolOrdering.cpp::WipingExtrusions` original-collection/copy overrides.
use crate::ExtrusionRole;
use std::collections::HashMap;

mod assign;
mod source;
pub(in crate::project_slice::gcode_emit) use source::objects;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::project_slice::gcode_emit) enum CollectionId {
    Perimeter(usize),
    Infill(usize),
}

pub(super) struct Collection {
    pub(super) id: CollectionId,
    pub(super) tool: usize,
    pub(super) role: ExtrusionRole,
    pub(super) volume: f64,
}

pub(super) struct Region {
    pub(super) infill_first: bool,
    pub(super) wall_tool: usize,
    pub(super) sparse_tool: usize,
    pub(super) infills: Vec<Collection>,
    pub(super) perimeters: Vec<Collection>,
}

pub(in crate::project_slice::gcode_emit) struct Object {
    pub(super) id: usize,
    pub(super) copies: usize,
    pub(super) flush_objects: bool,
    pub(super) flush_infill: bool,
    pub(super) flush_support: bool,
    pub(super) support_tools: [usize; 2],
    pub(super) support_interface_not_for_body: bool,
    pub(super) regions: Option<Vec<Region>>,
    pub(super) support: Option<Vec<(ExtrusionRole, f64)>>,
}

pub(in crate::project_slice::gcode_emit) struct Layer<'a> {
    pub(in crate::project_slice::gcode_emit) objects: &'a [Object],
    pub(in crate::project_slice::gcode_emit) tools: &'a [usize],
    pub(in crate::project_slice::gcode_emit) soluble: &'a [bool],
    pub(in crate::project_slice::gcode_emit) support_filaments: &'a [bool],
}

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct WipingExtrusions {
    entities: HashMap<(usize, CollectionId), Vec<Option<usize>>>,
    support: HashMap<usize, usize>,
    interface: HashMap<usize, usize>,
    something_overridable: bool,
}

impl WipingExtrusions {
    pub(in crate::project_slice::gcode_emit) fn new(layer: &Layer<'_>) -> Self {
        let something_overridable = layer.objects.iter().any(|object| {
            object
                .regions
                .iter()
                .flatten()
                .flat_map(|region| region.infills.iter().chain(&region.perimeters))
                .any(|collection| overridable(collection, object, layer))
                || object.flush_support
                    && object
                        .support
                        .iter()
                        .flatten()
                        .any(|&(role, _)| support_overridable(role, object.support_tools))
        });
        Self {
            something_overridable,
            ..Self::default()
        }
    }

    pub(in crate::project_slice::gcode_emit) fn has_overrides(&self) -> bool {
        !self.entities.is_empty() || !self.support.is_empty() || !self.interface.is_empty()
    }

    pub(in crate::project_slice::gcode_emit) fn entity(
        &self,
        object: usize,
        collection: CollectionId,
        copy: usize,
    ) -> Option<usize> {
        self.entities
            .get(&(object, collection))
            .and_then(|copies| copies[copy])
    }

    pub(in crate::project_slice::gcode_emit) fn support_tool(
        &self,
        object: usize,
        interface: bool,
    ) -> Option<usize> {
        if interface {
            self.interface.get(&object)
        } else {
            self.support.get(&object)
        }
        .copied()
    }

    fn assign(&mut self, object: &Object, collection: CollectionId, copy: usize, tool: usize) {
        // Native unsigned(-1) becomes the signed -1 "print normally" sentinel.
        self.entities
            .entry((object.id, collection))
            .or_insert_with(|| vec![None; object.copies])[copy] =
            (tool != usize::MAX).then_some(tool);
    }

    pub(in crate::project_slice::gcode_emit) fn ensure_order(&mut self, layer: &Layer<'_>) {
        if !self.something_overridable {
            return;
        }
        let eligible = |tool: &&usize| !layer.soluble[**tool] && !layer.support_filaments[**tool];
        // The source returns unsigned(-1) when no eligible tool exists.
        let first = layer
            .tools
            .iter()
            .find(eligible)
            .copied()
            .unwrap_or(usize::MAX);
        let last = layer
            .tools
            .iter()
            .rfind(eligible)
            .copied()
            .unwrap_or(usize::MAX);
        for object in layer
            .objects
            .iter()
            .filter(|object| object.flush_infill || object.flush_objects)
        {
            if object.regions.is_none() {
                continue;
            }
            for copy in 0..object.copies {
                self.ensure_copy_order(layer, object, copy, [first, last]);
            }
        }
    }
    fn ensure_copy_order(
        &mut self,
        layer: &Layer<'_>,
        object: &Object,
        copy: usize,
        tools: [usize; 2],
    ) {
        let regions = object.regions.as_ref().unwrap();
        for (collection, tool) in regions
            .iter()
            .flat_map(|region| normal_tools(region, layer, tools))
        {
            if overridable(collection, object, layer)
                && self.entity(object.id, collection.id, copy).is_none()
            {
                self.assign(object, collection.id, copy, tool);
            }
        }
    }
}

fn support_overridable(role: ExtrusionRole, tools: [usize; 2]) -> bool {
    match role {
        ExtrusionRole::Mixed => tools.contains(&0),
        ExtrusionRole::SupportMaterial | ExtrusionRole::SupportTransition => tools[0] == 0,
        ExtrusionRole::SupportMaterialInterface => tools[1] == 0,
        _ => false,
    }
}

fn normal_tools<'a>(
    region: &'a Region,
    layer: &Layer<'_>,
    [first, last]: [usize; 2],
) -> impl Iterator<Item = (&'a Collection, usize)> {
    // Preserve the source parentheses: the second argument of
    // is_extruder_order is this boolean converted to a tool ID.
    let order_tool = usize::from(last != 0 || !layer.tools.contains(&region.sparse_tool));
    let infill = region.infill_first || ordered(layer.tools, region.wall_tool, order_tool);
    let infill_tool = if region.infill_first { first } else { last };
    let perimeter_tool = if region.infill_first { last } else { first };
    region
        .infills
        .iter()
        .filter(move |_| infill)
        .map(move |collection| (collection, infill_tool))
        .chain(
            region
                .perimeters
                .iter()
                .map(move |collection| (collection, perimeter_tool)),
        )
}

fn overridable(collection: &Collection, object: &Object, layer: &Layer<'_>) -> bool {
    !layer.soluble[collection.tool]
        && (object.flush_objects
            || object.flush_infill && collection.role == ExtrusionRole::InternalInfill)
}

fn ordered(tools: &[usize], a: usize, b: usize) -> bool {
    a != b && tools.iter().find(|&&tool| tool == a || tool == b) == Some(&a)
}
