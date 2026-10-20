use crate::{
    ProjectVolumeType, SliceError,
    geometry::{self},
    project_slice::{
        compensation::RawBrimVolume, perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

use super::brim_geometry_error;

pub(super) struct BrimVolumeGroup {
    pub(super) volume_ids: Vec<usize>,
    pub(super) slices: Vec<geometry::ExPolygon>,
}

pub(super) fn volume_materials(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    filament_types: &[String],
) -> Vec<(usize, String)> {
    let (source_index, _) = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    let object = &traversal.project.objects()[source_index];
    let object_extruder = object
        .region_overrides()
        .extruder
        .map_or(1, |value| value.0);
    object
        .volumes()
        .iter()
        .enumerate()
        .filter_map(|(index, volume)| {
            let extruder = volume
                .region_overrides()
                .extruder
                .map_or(object_extruder, |value| value.0);
            (extruder > 0).then(|| {
                (
                    index,
                    filament_types
                        .get(extruder as usize - 1)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
        })
        .collect()
}

pub(super) fn object_adhesion(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    first_layer_regions: &[crate::project_slice::extrusion_islands::RegionToolUsage],
    volume_materials: &[(usize, String)],
) -> f64 {
    let input = &traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let regions = input.as_parts().0.as_parts().0.as_parts().2;
    // Brim.cpp:120-148: has_extrusions gates the region, but its external
    // perimeter extruder is selected even when only fill entities exist.
    let extruders = first_layer_regions
        .iter()
        .zip(regions)
        .filter(|(usage, _)| {
            usage.perimeters || usage.internal_solid || usage.top || usage.bottom || usage.infill
        })
        .map(|(_, region)| region.options.outer_wall_filament_id.0);
    let object = &traversal.project.objects()[input.identity().0];
    let object_extruder = object
        .region_overrides()
        .extruder
        .map_or(1, |value| value.0);
    // volume_materials retains model order. Last matching volume wins;
    // unused configured filaments and non-extruding regions do not vote.
    volume_materials
        .iter()
        .rfind(|(index, _)| {
            let extruder = object.volumes()[*index]
                .region_overrides()
                .extruder
                .map_or(object_extruder, |value| value.0);
            extruders.clone().any(|used| used == extruder)
        })
        .map_or(1.0, |(_, name)| {
            super::auto::material_terms(std::iter::once(name.as_str())).0
        })
}

pub(super) fn group_volume_slices(
    slices: Vec<RawBrimVolume>,
    layer_slices: &[geometry::ExPolygon],
    resolution: f64,
    offset_value: f64,
) -> Result<Vec<BrimVolumeGroup>, SliceError> {
    use geometry::{JoinType, offset_expolygons};
    // PrintObjectSlice.cpp:517-596: simplify every volume slice, then use
    // the fixed +0.05mm safety offset for doesVolumeIntersect.
    let offset = offset_value;
    let mut positive = Vec::new();
    let mut negative = Vec::new();
    for volume in slices {
        let mut raw = volume.first_layer;
        for expolygon in &mut raw {
            expolygon.douglas_peucker(resolution);
        }
        let expanded = offset_expolygons(&raw, offset as f32, JoinType::Miter, 3.0)
            .map_err(brim_geometry_error)?;
        let entry = (volume.source_volume_index, raw, expanded);
        if volume.volume_type == ProjectVolumeType::NegativeVolume {
            negative.push(entry);
        } else if volume.volume_type == ProjectVolumeType::ModelPart {
            positive.push(entry);
        }
    }
    // PrintObjectSlice.cpp:517-596 doesVolumeIntersect: union-find over
    // volume indices, retaining the first volume as the group representative.
    let mut parent: Vec<usize> = (0..positive.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for (i, left) in positive.iter().enumerate() {
        for (j, right) in positive.iter().enumerate().skip(i + 1) {
            let intersects = !geometry::intersection_ex(&left.2, &right.2)
                .map_err(brim_geometry_error)?
                .is_empty();
            if !intersects {
                continue;
            }
            let ri = root(&mut parent, i);
            let rj = root(&mut parent, j);
            if ri == rj {
                continue;
            }
            parent[rj] = ri;
        }
    }
    let mut groups = Vec::<BrimVolumeGroup>::new();
    let mut group_index = std::collections::BTreeMap::<usize, usize>::new();
    for (i, positive_volume) in positive.iter().enumerate() {
        let r = root(&mut parent, i);
        let index = if let Some(&index) = group_index.get(&r) {
            index
        } else {
            let index = groups.len();
            groups.push(BrimVolumeGroup {
                volume_ids: Vec::new(),
                slices: Vec::new(),
            });
            group_index.insert(r, index);
            index
        };
        groups[index].volume_ids.push(positive_volume.0);
        groups[index].slices.extend(positive_volume.1.clone());
    }
    // :517-596 unions the group's expanded polygons and contracts by the
    // offset before the final Douglas-Peucker pass.
    for group in &mut groups {
        let mut expanded = offset_expolygons(&group.slices, offset as f32, JoinType::Miter, 3.0)
            .map_err(brim_geometry_error)?;
        expanded = geometry::union_expolygons(&expanded, &[]).map_err(brim_geometry_error)?;
        group.slices = offset_expolygons(&expanded, -(offset as f32), JoinType::Miter, 3.0)
            .map_err(brim_geometry_error)?;
        for expolygon in &mut group.slices {
            expolygon.douglas_peucker(resolution);
        }
    }
    // PrintObjectSlice.cpp:608-626 applyNegtiveVolumes: append negative
    // first-layer slices, difference them from each group, then simplify.
    let mut neg_total = Vec::new();
    for (_, raw, _) in negative {
        neg_total.extend(raw);
    }
    if !neg_total.is_empty() {
        for group in &mut groups {
            group.slices =
                geometry::difference_ex(&group.slices, &neg_total).map_err(brim_geometry_error)?;
            for expolygon in &mut group.slices {
                expolygon.douglas_peucker(resolution);
            }
        }
    }
    // PrintObjectSlice.cpp:628-660 reGroupingLayerPolygons: the actual
    // first-layer lslices are assigned to the group with the smallest
    // difference area, then replace the temporary union with those lslices.
    // Keep this assignment separate from the union used only for overlap
    // testing, matching the upstream groupVector order.
    let mut assigned = groups
        .iter()
        .map(|group| BrimVolumeGroup {
            volume_ids: group.volume_ids.clone(),
            slices: Vec::new(),
        })
        .collect::<Vec<_>>();
    let comparison_slices = groups
        .iter()
        .map(|group| {
            let mut slices = group.slices.clone();
            for polygon in &mut slices {
                polygon.douglas_peucker(resolution);
            }
            slices
        })
        .collect::<Vec<_>>();
    for expolygon in layer_slices.iter().cloned() {
        assign_polygon(expolygon, &comparison_slices, &mut assigned, resolution)?;
    }
    Ok(assigned)
}

fn assign_polygon(
    expolygon: geometry::ExPolygon,
    comparison_slices: &[Vec<geometry::ExPolygon>],
    assigned: &mut [BrimVolumeGroup],
    resolution: f64,
) -> Result<(), SliceError> {
    let mut simplified = expolygon.clone();
    simplified.douglas_peucker(resolution);
    let mut min_area = simplified.area();
    if min_area <= 0.0 {
        return Ok(());
    }
    let mut best = None;
    for (index, group_slices) in comparison_slices.iter().enumerate() {
        let diff = geometry::difference_ex(std::slice::from_ref(&simplified), group_slices)
            .map_err(brim_geometry_error)?;
        let area = diff.iter().map(geometry::ExPolygon::area).sum::<f64>();
        if area < min_area {
            min_area = area;
            best = Some(index);
        }
    }
    let Some(index) = best else {
        return Ok(());
    };
    assigned[index].slices.push(expolygon);
    Ok(())
}

pub(super) fn model_mesh_bbox(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    volume_ids: Option<&[usize]>,
) -> [f64; 3] {
    let (source_index, transform_index) = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    let object = &traversal.project.objects()[source_index];
    let instance = object.instances()[transform_index].transform();
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for (source_index, volume) in object.volumes().iter().enumerate() {
        if volume.volume_type() != ProjectVolumeType::ModelPart {
            continue;
        }
        if let Some(ids) = volume_ids
            && !ids.contains(&source_index)
        {
            continue;
        }
        let bbox = crate::project::region_graph::bounds::transformed_mesh_bbox(
            volume.mesh(),
            instance.then(volume.transform()),
            0.0,
        );
        for axis in 0..3 {
            min[axis] = min[axis].min(bbox.min()[axis]);
            max[axis] = max[axis].max(bbox.max()[axis]);
        }
    }
    [
        f64::from(max[0] - min[0]),
        f64::from(max[1] - min[1]),
        f64::from(max[2] - min[2]),
    ]
}
