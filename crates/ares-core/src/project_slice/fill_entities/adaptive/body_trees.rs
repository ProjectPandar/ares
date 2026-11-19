use std::{collections::BTreeMap, sync::Arc};

use crate::{
    SliceError,
    fill::adaptive::octree::{self, Octree},
    geometry::{ClipperError, ExPolygon, triangulate_expolygon_3d},
    project_slice::perimeters::{
        classic::traversal::types::PreparedPostClassicTraversal, types::RegionLayerIndex,
    },
};

use super::{body_layout::BodyLayout, body_mesh::Mesh, preparation::SupportPreparation};

pub(super) struct BodyTrees {
    layout: BodyLayout,
    sets: BTreeMap<u64, Vec<Option<Arc<Octree>>>>,
}

impl BodyTrees {
    pub(super) fn for_fill(
        &self,
        spacing: f64,
        layer: usize,
        islands: &[ExPolygon],
        surface: &ExPolygon,
    ) -> Result<Arc<Octree>, ClipperError> {
        let trees = &self.sets[&spacing.to_bits()];
        let body = self.layout.fill_body(layer, islands, surface)?;
        // FillAdaptive.hpp:43-60: body, whole object, any prepared body.
        Ok(Arc::clone(
            body.and_then(|body| trees[body].as_ref())
                .or_else(|| trees[self.layout.count].as_ref())
                .or_else(|| trees.iter().flatten().next())
                .expect("active spacing has a prepared tree"),
        ))
    }
}

pub(super) fn prepare(context: &SupportPreparation<'_>) -> Result<(), SliceError> {
    let SupportPreparation {
        traversal,
        object_index,
        candidates,
        horizontal,
        prelude,
        spacing,
    } = context;
    if !spacing.adaptive.iter().any(|&spacing| spacing > 0.0) {
        return Ok(());
    }
    let (regions, islands) = prelude.object.as_parts();
    let (plan, _, _) = regions.as_parts();
    let layout = BodyLayout::connected(islands).map_err(super::clipper_error)?;
    if layout.count <= 1 {
        return Ok(());
    }
    let scale = traversal.scale;
    let (source, group) = super::object_input(traversal, *object_index);
    let transform = super::centered_transform(source, group, scale)?;
    let mesh = Mesh::rotated(source, transform);
    let (config, _) = traversal.print_object_config(*object_index);
    let Some(bodies) = mesh.split_by_body(
        &layout,
        (&plan.layers, islands),
        config.object.layer_height.0,
        scale,
    ) else {
        return Ok(());
    };

    // PrintObject.cpp:1200-1249: rotate the bridge payload once, keep an
    // object-wide copy, and attach each original candidate to its body.
    let mut overhangs = vec![Vec::new(); layout.count + 1];
    let rotation = octree::octree_rotation_to_octree();
    for candidate in candidates.surfaces_by_layer.values().flatten() {
        let source = candidate.source;
        let layer = source.layer_index;
        let slot = prelude.region_layer_record_index(RegionLayerIndex {
            region_index: source.region_index,
            layer_index: layer,
        });
        let record = horizontal.records[slot].as_ref().unwrap();
        let surface = record.fill_surfaces[source.surface_index].as_parts().1;
        let bottom = plan.layers[layer].print_z - plan.layers[layer].height;
        let triangles = triangulate_expolygon_3d(surface, f64::from(bottom as f32), false, scale)
            .into_iter()
            .map(|point| octree::rotate_point(rotation, point));
        if let Some(body) =
            layout.body_at(layer, &islands[layer], surface.contour().points()[0], scale)
        {
            overhangs[body].extend(triangles.clone());
        }
        overhangs[layout.count].extend(triangles);
    }

    let mut to_build = BTreeMap::new();
    for (region_index, &distance) in spacing
        .adaptive
        .iter()
        .enumerate()
        .filter(|&(_, spacing)| *spacing > 0.0)
    {
        let trees = to_build
            .entry(distance.to_bits())
            .or_insert_with(|| vec![false; layout.count + 1]);
        for (layer_index, islands) in islands.iter().enumerate() {
            let slot = prelude.region_layer_record_index(RegionLayerIndex {
                region_index,
                layer_index,
            });
            if let Some(record) = &horizontal.records[slot] {
                for fill in &record.fill_surfaces {
                    let p = fill.as_parts().1.contour().points()[0];
                    let body = layout
                        .body_at(layer_index, islands, p, scale)
                        .filter(|&id| !bodies[id].is_empty())
                        .unwrap_or(layout.count);
                    trees[body] = true;
                }
            }
        }
    }
    let sets = to_build
        .into_iter()
        .map(|(distance, active)| {
            let trees = active
                .into_iter()
                .enumerate()
                .map(|(body, build)| {
                    build.then(|| {
                        let mesh = if body == layout.count {
                            &mesh
                        } else {
                            &bodies[body]
                        };
                        Arc::new(octree::build_octree(
                            mesh.input(),
                            &overhangs[body],
                            f64::from_bits(distance),
                            false,
                            [0.0; 3],
                        ))
                    })
                })
                .collect();
            (distance, trees)
        })
        .collect();
    assert!(spacing.body_trees.set(BodyTrees { layout, sets }).is_ok());
    Ok(())
}

pub(super) fn layer_islands(
    traversal: &PreparedPostClassicTraversal,
    object: usize,
    layer: usize,
) -> &[ExPolygon] {
    let prelude = &traversal.objects[object]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    &prelude.object.as_parts().1[layer]
}
