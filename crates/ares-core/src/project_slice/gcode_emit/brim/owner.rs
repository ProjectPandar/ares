//! Brim.cpp:619-633,1004-1010: instance areas collected as per-instance
//! map entries under one PrintObject ID.
use super::bed_area::translate;
use super::types::InstanceBrim;
use super::{BrimPlan, bed_area};
use crate::{
    geometry::ExPolygon,
    project_slice::{
        gcode_emit::footprint, perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

/// One `brimAreaMap` entry: `{object_id, instance_idx}` → translated area.
pub(super) struct InstanceEntry {
    pub(super) copy: usize,
    pub(super) area: Vec<ExPolygon>,
    pub(super) islands: Vec<ExPolygon>,
}

/// The generated collections: per-instance infills, the merged-owner path
/// set, and the masked areas (hull source).
type AreaOutput = (
    Vec<InstanceBrim>,
    Vec<Vec<crate::geometry::Point>>,
    Vec<ExPolygon>,
);

pub(super) struct Area {
    pub(super) entries: Vec<InstanceEntry>,
    /// `Print.cpp:3849-3862/:4326-4340, GCode.cpp:5991-6082`: per-instance
    /// generation/emission exists only where the group machinery keeps
    /// per-instance entries — uncombined (`combine_brims` off) by-layer
    /// printing. ByObject consumes the owner-keyed `brimMap` (:1006) and
    /// `combine_brims` merges touching instances' entities; both emit the
    /// owner-merged collection.
    pub(super) per_instance: bool,
    pub(super) owner_collection: bool,
    pub(super) flow: crate::project_slice::perimeters::types::Flow,
    pub(super) spacing: f32,
    pub(super) resolution: f64,
    pub(super) logical_filament: usize,
    pub(super) retain_contact: bool,
}

impl Area {
    pub(super) fn finish(
        mut self,
        traversal: &PreparedPostClassicTraversal,
        object_index: usize,
        no_brim_area: &[ExPolygon],
        tower_exclusions: &[ExPolygon],
    ) -> Result<Option<BrimPlan>, crate::SliceError> {
        let (instances, merged_paths, area_all) = if self.per_instance {
            self.finish_instances(traversal, object_index, no_brim_area, tower_exclusions)?
        } else {
            self.finish_merged(traversal, object_index, no_brim_area, tower_exclusions)?
        };
        if area_all.is_empty()
            || (self.owner_collection && instances.is_empty() && merged_paths.is_empty())
        {
            return Ok(None);
        }
        let covered_hull = crate::geometry::convex_hull(
            &area_all
                .iter()
                .flat_map(|expolygon| expolygon.contour().points().iter().copied())
                .collect::<Vec<_>>(),
        );
        Ok(Some(BrimPlan {
            area: area_all,
            global: false,
            owner_collection: self.owner_collection,
            instances,
            paths: merged_paths,
            covered_hull,
            width: self.flow.width,
            spacing: self.flow.spacing,
            height: self.flow.height,
            mm3_per_mm: self.flow.mm3_per_mm,
        }))
    }

    /// `Brim.cpp:1004-1010`: per map entry — masks and contact retention
    /// (:718-722,726-766) and one `makeBrimInfill` per entry's area.
    fn finish_instances(
        &mut self,
        traversal: &PreparedPostClassicTraversal,
        object_index: usize,
        no_brim_area: &[ExPolygon],
        tower_exclusions: &[ExPolygon],
    ) -> Result<AreaOutput, crate::SliceError> {
        let mut instances = Vec::new();
        let mut area_all = Vec::new();
        for entry in std::mem::take(&mut self.entries) {
            let brim_area = self.masked(
                traversal,
                object_index,
                entry.area,
                &entry.islands,
                no_brim_area,
                tower_exclusions,
            )?;
            if brim_area.is_empty() {
                continue;
            }
            let paths = self.infill(&brim_area, traversal)?;
            if paths.is_empty() {
                continue;
            }
            instances.push(InstanceBrim {
                copy: entry.copy,
                paths,
            });
            area_all.extend(brim_area);
        }
        Ok((instances, Vec::new(), area_all))
    }

    /// Owner-merged collection (the pre-existing scaffold): one
    /// clip/retention/infill pass over the concatenated entry areas.
    fn finish_merged(
        &mut self,
        traversal: &PreparedPostClassicTraversal,
        object_index: usize,
        no_brim_area: &[ExPolygon],
        tower_exclusions: &[ExPolygon],
    ) -> Result<AreaOutput, crate::SliceError> {
        let merged = self
            .entries
            .iter()
            .flat_map(|entry| entry.area.iter().cloned())
            .collect::<Vec<_>>();
        let islands = self
            .entries
            .iter()
            .flat_map(|entry| entry.islands.iter().cloned())
            .collect::<Vec<_>>();
        let brim_area = self.masked(
            traversal,
            object_index,
            merged,
            &islands,
            no_brim_area,
            tower_exclusions,
        )?;
        if brim_area.is_empty() {
            return Ok((Vec::new(), Vec::new(), Vec::new()));
        }
        let merged_paths = if self.owner_collection {
            self.infill(&brim_area, traversal)?
        } else {
            Vec::new()
        };
        Ok((Vec::new(), merged_paths, brim_area))
    }

    /// Brim.cpp:718-722,726-766: the no-brim/tower masks and contact
    /// retention applied to one area (per map entry or the merged owner
    /// collection).
    #[expect(
        clippy::too_many_arguments,
        reason = "the mask pass shares the finish context"
    )]
    fn masked(
        &self,
        traversal: &PreparedPostClassicTraversal,
        object_index: usize,
        area: Vec<ExPolygon>,
        islands: &[ExPolygon],
        no_brim_area: &[ExPolygon],
        tower_exclusions: &[ExPolygon],
    ) -> Result<Vec<ExPolygon>, crate::SliceError> {
        let brim_area = if self.owner_collection {
            bed_area::clip_plate(
                traversal,
                self.logical_filament,
                (area, no_brim_area.to_vec()),
                tower_exclusions,
                self.resolution,
            )?
        } else {
            bed_area::clip(
                traversal,
                object_index,
                self.logical_filament,
                (area, no_brim_area.to_vec(), tower_exclusions),
                self.resolution,
            )?
        };
        if self.retain_contact {
            crate::brims::inner::retain_contact(brim_area, islands, self.spacing, self.resolution)
        } else {
            Ok(brim_area)
        }
    }

    /// `makeBrimInfill` over one area; Brim.cpp:820 keeps the fractional
    /// lattice resolution.
    fn infill(
        &self,
        area: &[ExPolygon],
        traversal: &PreparedPostClassicTraversal,
    ) -> Result<Vec<Vec<crate::geometry::Point>>, crate::SliceError> {
        crate::brims::ears::paths::from_area(
            area.to_vec(),
            self.spacing,
            traversal.scale.scaled_delta(0.0125),
            traversal.scale,
        )
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

type OwnerAreas = (Vec<InstanceEntry>, Vec<ExPolygon>);

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
    let mut entries = Vec::new();
    let mut owner_forbidden = Vec::new();
    for (copy, &member) in group.instance_indices.iter().enumerate() {
        let shift = footprint::instance_shift(&object.instances()[member], center, traversal.scale);
        // Brim.cpp:36-45,573-581 (2.4.2-era scaffold; the beta map-level
        // claimed-area loop :953-958 is the subtraction slice): assign only
        // area not already owned, per instance, keeping each copy's entry
        // separate (:619-633 appends per `{object_id, instance_idx}`).
        // The accumulator extends only after the owner's instance loop —
        // same-owner copies never claim against each other.
        let assigned =
            crate::geometry::difference_ex(&translate(area.to_vec(), shift), accumulated)
                .map_err(super::brim_geometry_error)?;
        owner_forbidden.extend(translate(forbidden.to_vec(), shift));
        entries.push(InstanceEntry {
            copy,
            area: assigned,
            islands: translate(islands.to_vec(), shift),
        });
    }
    accumulated.extend(entries.iter().flat_map(|entry| entry.area.iter().cloned()));
    Ok((entries, owner_forbidden))
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
