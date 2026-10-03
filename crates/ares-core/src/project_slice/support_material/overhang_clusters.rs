//! `OverhangCluster`, `add_overhang` and small-cluster removal in SupportMaterial.cpp.

use super::{
    overhangs::{SharpTails, overlaps},
    sharp_tail_propagation::extent_size,
};
use crate::geometry::{
    ClipperError, ExPolygon, JoinType, Polygon, offset_expolygon, offset_expolygons,
    union_expolygons,
};

struct OverhangCluster {
    // Stable source polygon indices replace pointers into per-layer vectors.
    members: Vec<(usize, usize)>,
    merged_dilated: Vec<ExPolygon>,
    min_layer: usize,
    max_layer: usize,
    offset_scaled: f64,
    is_cantilever: bool,
}

impl OverhangCluster {
    fn intersects(&self, polygon: &ExPolygon, layer_index: usize) -> Result<bool, ClipperError> {
        if layer_index < 1 || layer_index + 1 < self.min_layer || layer_index > self.max_layer + 1 {
            return Ok(false);
        }
        let expanded = offset_expolygon(polygon, self.offset_scaled as f32, JoinType::Miter, 3.0)?;
        overlaps(&expanded, &self.merged_dilated)
    }

    fn insert(
        &mut self,
        polygon: &ExPolygon,
        layer: usize,
        index: usize,
    ) -> Result<(), ClipperError> {
        self.members.push((layer, index));
        let expanded = if self.offset_scaled > 1e-4 {
            offset_expolygon(polygon, self.offset_scaled as f32, JoinType::Miter, 3.0)?
        } else {
            vec![polygon.clone()]
        };
        if !expanded.is_empty() {
            self.merged_dilated = union_expolygons(&self.merged_dilated, &expanded)?;
        }
        self.min_layer = self.min_layer.min(layer);
        self.max_layer = self.max_layer.max(layer);
        Ok(())
    }
}

pub(super) fn remove_small_overhangs(
    overhangs: &mut [Vec<ExPolygon>],
    sharp_tails: &[SharpTails],
    cantilevers: &[Vec<ExPolygon>],
    start_layer: usize,
    extrusion_width_scaled: f64,
) -> Result<(), ClipperError> {
    let mut clusters: Vec<OverhangCluster> = Vec::new();
    for (layer_index, layer) in overhangs.iter().enumerate().skip(start_layer) {
        for (polygon_index, polygon) in layer.iter().enumerate() {
            let index = add_overhang(
                &mut clusters,
                polygon,
                (layer_index, polygon_index),
                extrusion_width_scaled,
            )?;
            let cluster = &mut clusters[index];
            if overlaps(std::slice::from_ref(polygon), &cantilevers[layer_index])? {
                cluster.is_cantilever = true;
            }
        }
    }
    for cluster in clusters {
        let mut is_sharp_tail = false;
        for tails in &sharp_tails[cluster.min_layer..=cluster.max_layer] {
            if overlaps(&tails.polygons, &cluster.merged_dilated)? {
                is_sharp_tail = true;
                break;
            }
        }
        if is_sharp_tail || cluster.is_cantilever {
            continue;
        }
        let eroded = offset_expolygons(
            &cluster.merged_dilated,
            (-extrusion_width_scaled) as f32,
            JoinType::Miter,
            3.0,
        )?;
        let size = extent_size(&eroded);
        if size.x() as f64 >= 2.0 * extrusion_width_scaled
            && size.y() as f64 >= 2.0 * extrusion_width_scaled
        {
            continue;
        }
        for (layer, polygon) in cluster.members {
            // Clear the original slot, as upstream does, without compressing
            // the per-layer indices used throughout contact detection.
            overhangs[layer][polygon] = ExPolygon::new(Polygon::new(Vec::new()), Vec::new());
        }
    }
    Ok(())
}

fn add_overhang(
    clusters: &mut Vec<OverhangCluster>,
    polygon: &ExPolygon,
    (layer_index, polygon_index): (usize, usize),
    extrusion_width_scaled: f64,
) -> Result<usize, ClipperError> {
    for (index, cluster) in clusters.iter_mut().enumerate() {
        if cluster.intersects(polygon, layer_index)? {
            cluster.insert(polygon, layer_index, polygon_index)?;
            return Ok(index);
        }
    }
    let index = clusters.len();
    let mut cluster = OverhangCluster {
        members: Vec::new(),
        merged_dilated: Vec::new(),
        min_layer: 10_000_000,
        max_layer: 0,
        offset_scaled: extrusion_width_scaled,
        is_cantilever: false,
    };
    cluster.insert(polygon, layer_index, polygon_index)?;
    clusters.push(cluster);
    Ok(index)
}
