//! Active Tree-local `OverhangCluster` and collection in `TreeSupport.cpp`.

use std::collections::BTreeMap;

use super::{OverhangLayer, SmartOverhangs, overlaps};
use crate::{
    ProcessSupportType,
    geometry::{
        BoundingBox, ClipperError, ExPolygon, JoinType, Point, offset_expolygon, offset_expolygons,
        union_expolygons,
    },
    project_slice::support_material::sharp_tail_propagation::extent_size,
};

struct OverhangCluster {
    members: BTreeMap<usize, usize>,
    merged: Vec<ExPolygon>,
    bounds: Option<BoundingBox>,
    min_layer: usize,
    max_layer: usize,
    is_cantilever: bool,
    is_small: bool,
}

impl OverhangCluster {
    fn new(polygon: &ExPolygon, key: (usize, usize)) -> Result<Self, ClipperError> {
        let expanded = offset_expolygon(polygon, 0.0, JoinType::Miter, 3.0)?;
        let merged = if expanded.is_empty() {
            Vec::new()
        } else {
            union_expolygons(&[], &expanded)?
        };
        Ok(Self {
            members: BTreeMap::from([key]),
            merged,
            bounds: BoundingBox::from_expolygons(&expanded),
            min_layer: 10_000_000.min(key.0),
            max_layer: key.0,
            is_cantilever: false,
            is_small: false,
        })
    }

    fn push_back_if_intersects(
        &mut self,
        polygon: &ExPolygon,
        (layer, polygon_index): (usize, usize),
        layers: &[OverhangLayer],
        offset: f64,
    ) -> Result<bool, ClipperError> {
        if layer < 1 {
            return Ok(false);
        }
        let Some(&previous) = self.members.get(&(layer - 1)) else {
            return Ok(false);
        };
        let expanded = offset_expolygon(polygon, offset as f32, JoinType::Miter, 3.0)?;
        if expanded.is_empty() {
            return Ok(false);
        }
        let bounds = BoundingBox::from_expolygons(&expanded)
            .expect("expanded cluster polygons have nonempty contours");
        // Native undefined BoundingBox retains zero min/max for overlap queries.
        let accumulated = self
            .bounds
            .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)));
        if !bounds_overlap(accumulated, bounds)
            || !overlaps(
                std::slice::from_ref(&layers[layer - 1].raw_overhangs[previous]),
                &expanded,
            )?
        {
            return Ok(false);
        }
        self.members.entry(layer).or_insert(polygon_index);
        self.merged = union_expolygons(&self.merged, &expanded)?;
        self.min_layer = self.min_layer.min(layer);
        self.max_layer = self.max_layer.max(layer);
        match &mut self.bounds {
            Some(accumulated) => {
                accumulated.merge_point(bounds.min());
                accumulated.merge_point(bounds.max());
            }
            None => self.bounds = Some(bounds),
        }
        Ok(true)
    }

    fn filter_small(&mut self, layers: &[OverhangLayer], width: f64) -> Result<(), ClipperError> {
        for layer in &layers[self.min_layer..=self.max_layer] {
            if overlaps(&layer.sharp_tails, &self.merged)? {
                return Ok(());
            }
        }
        if self.is_cantilever {
            return Ok(());
        }
        let eroded = offset_expolygons(&self.merged, (-width) as f32, JoinType::Miter, 3.0)?;
        let size = extent_size(&eroded);
        self.is_small = (size.x() as f64) < 2.0 * width || (size.y() as f64) < 2.0 * width;
        Ok(())
    }
}

impl SmartOverhangs<'_> {
    pub(in super::super) fn collect_cluster_overhangs(&mut self) -> Result<(), ClipperError> {
        let mut clusters = group_clusters(&self.layers, self.extrusion_width_scaled)?;
        if matches!(self.object.support_type, ProcessSupportType::TreeAuto)
            && self.remove_small_overhangs
        {
            for cluster in &mut clusters {
                cluster.filter_small(&self.layers, self.extrusion_width_scaled)?;
            }
        }
        for cluster in clusters.into_iter().filter(|cluster| !cluster.is_small) {
            for (layer, polygon) in cluster.members {
                let polygon = self.layers[layer].raw_overhangs[polygon].clone();
                self.layers[layer].overhangs.push(polygon);
            }
        }
        Ok(())
    }
}

fn group_clusters(
    layers: &[OverhangLayer],
    offset: f64,
) -> Result<Vec<OverhangCluster>, ClipperError> {
    let mut clusters = Vec::new();
    for (layer_index, layer) in layers.iter().enumerate() {
        for (polygon_index, polygon) in layer.raw_overhangs.iter().enumerate() {
            let cluster = find_and_insert_cluster(
                &mut clusters,
                polygon,
                (layer_index, polygon_index),
                layers,
                offset,
            )?;
            clusters[cluster].is_cantilever |=
                overlaps(std::slice::from_ref(polygon), &layer.cantilevers)?;
        }
    }
    Ok(clusters)
}

fn find_and_insert_cluster(
    clusters: &mut Vec<OverhangCluster>,
    polygon: &ExPolygon,
    key: (usize, usize),
    layers: &[OverhangLayer],
    offset: f64,
) -> Result<usize, ClipperError> {
    for (index, cluster) in clusters.iter_mut().enumerate() {
        if cluster.push_back_if_intersects(polygon, key, layers, offset)? {
            return Ok(index);
        }
    }
    let index = clusters.len();
    clusters.push(OverhangCluster::new(polygon, key)?);
    Ok(index)
}

fn bounds_overlap(left: BoundingBox, right: BoundingBox) -> bool {
    !(left.max().x() < right.min().x()
        || left.min().x() > right.max().x()
        || left.max().y() < right.min().y()
        || left.min().y() > right.max().y())
}
