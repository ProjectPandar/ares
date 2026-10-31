//! Brim.cpp:36-45,573-581: instance areas collected under one PrintObject ID.
use super::bed_area::translate;
use super::{BrimPlan, bed_area};
use crate::{
    geometry::ExPolygon,
    project_slice::{
        gcode_emit::footprint, perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(super) struct Area {
    pub(super) area: Vec<ExPolygon>,
    pub(super) object_islands: Vec<ExPolygon>,
    pub(super) owner_collection: bool,
    pub(super) flow: crate::project_slice::perimeters::types::Flow,
    pub(super) spacing: f32,
    pub(super) resolution: f64,
    pub(super) logical_filament: usize,
    pub(super) retain_contact: bool,
}

impl Area {
    pub(super) fn finish(
        self,
        traversal: &PreparedPostClassicTraversal,
        object_index: usize,
        no_brim_area: &[ExPolygon],
        tower_exclusions: &[ExPolygon],
    ) -> Result<Option<BrimPlan>, crate::SliceError> {
        let brim_area = if self.owner_collection {
            bed_area::clip_plate(
                traversal,
                self.logical_filament,
                (self.area, no_brim_area.to_vec()),
                tower_exclusions,
                self.resolution,
            )?
        } else {
            bed_area::clip(
                traversal,
                object_index,
                self.logical_filament,
                (self.area, no_brim_area.to_vec(), tower_exclusions),
                self.resolution,
            )?
        };
        let brim_area = if self.retain_contact {
            crate::brims::inner::retain_contact(
                brim_area,
                &self.object_islands,
                self.spacing,
                self.resolution,
            )?
        } else {
            brim_area
        };
        if brim_area.is_empty() {
            return Ok(None);
        }
        let covered_hull = crate::geometry::convex_hull(
            &brim_area
                .iter()
                .flat_map(|expolygon| expolygon.contour().points().iter().copied())
                .collect::<Vec<_>>(),
        );
        let paths = if self.owner_collection {
            // Brim.cpp:820 keeps the fractional lattice resolution.
            let paths = crate::brims::ears::paths::from_area(
                brim_area.clone(),
                self.spacing,
                traversal.scale.scaled_delta(0.0125),
                traversal.scale,
            )?;
            if paths.is_empty() {
                return Ok(None);
            }
            paths
        } else {
            Vec::new()
        };
        Ok(Some(BrimPlan {
            area: brim_area,
            global: false,
            owner_collection: self.owner_collection,
            paths,
            covered_hull,
            width: self.flow.width,
            spacing: self.flow.spacing,
            height: self.flow.height,
            mm3_per_mm: self.flow.mm3_per_mm,
        }))
    }
}

pub(super) fn generation_order(traversal: &PreparedPostClassicTraversal) -> Vec<usize> {
    let mut owners = Vec::new();
    let mut centers = Vec::new();
    for index in 0..traversal.objects.len() {
        let (resolved, group) = traversal.print_object_config(index);
        let object = &traversal.project.objects()[resolved.source_object_index];
        let center = crate::project_slice::centering::center(object, group, traversal.scale)
            .expect("sliced object retains its raw center");
        for &member in &group.instance_indices {
            let shift =
                footprint::instance_shift(&object.instances()[member], center, traversal.scale);
            owners.push(index);
            centers.push((shift.x(), shift.y()));
        }
    }
    // Print.cpp:2520-2530: unseeded instance chain, first occurrence of each owner.
    let mut seen = vec![false; traversal.objects.len()];
    super::super::layers::object_order::chain_without_start(&centers)
        .into_iter()
        .map(|instance| owners[instance])
        .filter(|&owner| !std::mem::replace(&mut seen[owner], true))
        .collect()
}

type OwnerAreas = (Vec<ExPolygon>, Vec<ExPolygon>, Vec<ExPolygon>);

pub(super) fn expand(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    (area, forbidden, islands): (&[ExPolygon], &[ExPolygon], &[ExPolygon]),
    accumulated: &mut Vec<ExPolygon>,
) -> Result<OwnerAreas, crate::SliceError> {
    let (resolved, group) = traversal.print_object_config(object_index);
    let object = &traversal.project.objects()[resolved.source_object_index];
    let center = crate::project_slice::centering::center(object, group, traversal.scale)
        .expect("sliced object retains its raw center");
    let mut owner_area = Vec::new();
    let mut owner_forbidden = Vec::new();
    let mut owner_islands = Vec::new();
    for &member in &group.instance_indices {
        let shift = footprint::instance_shift(&object.instances()[member], center, traversal.scale);
        // Brim.cpp:36-45,573-581: assign only area not already owned,
        // accumulating before the later no-brim/bed masks and infill.
        let assigned =
            crate::geometry::difference_ex(&translate(area.to_vec(), shift), accumulated)
                .map_err(super::brim_geometry_error)?;
        owner_area.extend(assigned);
        owner_forbidden.extend(translate(forbidden.to_vec(), shift));
        owner_islands.extend(translate(islands.to_vec(), shift));
    }
    accumulated.extend_from_slice(&owner_area);
    Ok((owner_area, owner_forbidden, owner_islands))
}

pub(super) fn painted_points(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
) -> Vec<([f32; 2], f32)> {
    let source_index = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity()
        .0;
    traversal.project.objects()[source_index].brim_ear_centers()
}
