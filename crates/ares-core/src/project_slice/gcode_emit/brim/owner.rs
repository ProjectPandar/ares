//! Brim.cpp:619-633,1004-1010: instance areas collected as per-instance
//! map entries under one PrintObject ID.
use super::bed_area::translate;
use super::types::{InstanceArea, InstanceBrim};
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

/// The generated collections: per-instance infills, the per-instance
/// plate-frame areas (skirt grouping input, `Brim.cpp:984-996`), the
/// merged-owner path set, and the masked areas (hull source).
type AreaOutput = (
    Vec<InstanceBrim>,
    Vec<InstanceArea>,
    Vec<Vec<crate::geometry::Point>>,
    Vec<ExPolygon>,
);

pub(super) struct Area {
    pub(super) entries: Vec<InstanceEntry>,
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
        let (instances, instance_areas, merged_paths, area_all) =
            self.finish_instances(traversal, object_index, no_brim_area, tower_exclusions)?;
        if area_all.is_empty() || (instances.is_empty() && merged_paths.is_empty()) {
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
            instances,
            instance_areas,
            paths: merged_paths,
            covered_hull,
            width: self.flow.width,
            spacing: self.flow.spacing,
            height: self.flow.height,
            mm3_per_mm: self.flow.mm3_per_mm,
        }))
    }

    /// `Brim.cpp:718-722,726-766`: masks and contact retention per map
    /// entry — split from the infill so the ordered claimed-area
    /// subtraction (:953-958) can run between them, as in `make_brim`.
    pub(super) fn mask_entries(
        &mut self,
        traversal: &PreparedPostClassicTraversal,
        no_brim_area: &[ExPolygon],
        tower_exclusions: &[ExPolygon],
    ) -> Result<(), crate::SliceError> {
        let entries = std::mem::take(&mut self.entries);
        self.entries = entries
            .into_iter()
            .map(|mut entry| {
                let islands = std::mem::take(&mut entry.islands);
                let area = std::mem::take(&mut entry.area);
                entry.area =
                    self.masked(traversal, (area, &islands, no_brim_area, tower_exclusions))?;
                Ok(entry)
            })
            .collect::<Result<Vec<_>, crate::SliceError>>()?;
        Ok(())
    }

    /// `Brim.cpp:1004-1010`: per map entry — one `makeBrimInfill` per
    /// entry's (already masked and claimed) area.
    fn finish_instances(
        &mut self,
        traversal: &PreparedPostClassicTraversal,
        _object_index: usize,
        _no_brim_area: &[ExPolygon],
        _tower_exclusions: &[ExPolygon],
    ) -> Result<AreaOutput, crate::SliceError> {
        let mut instances = Vec::new();
        let mut instance_areas = Vec::new();
        let mut area_all = Vec::new();
        for entry in std::mem::take(&mut self.entries) {
            if entry.area.is_empty() {
                continue;
            }
            instance_areas.push(InstanceArea {
                copy: entry.copy,
                area: entry.area.clone(),
            });
            let paths = self.infill(&entry.area, traversal)?;
            if paths.is_empty() {
                continue;
            }
            instances.push(InstanceBrim {
                copy: entry.copy,
                paths,
            });
            area_all.extend(entry.area);
        }
        Ok((instances, instance_areas, Vec::new(), area_all))
    }

    /// Brim.cpp:718-722,726-766: the no-brim/tower masks and contact
    /// retention applied to one area (per map entry).
    fn masked(
        &self,
        traversal: &PreparedPostClassicTraversal,
        (area, islands, no_brim_area, tower_exclusions): (
            Vec<ExPolygon>,
            &[ExPolygon],
            &[ExPolygon],
            &[ExPolygon],
        ),
    ) -> Result<Vec<ExPolygon>, crate::SliceError> {
        let brim_area = bed_area::clip_plate(
            traversal,
            self.logical_filament,
            (area, no_brim_area.to_vec()),
            tower_exclusions,
            self.resolution,
        )?;
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

/// `Brim.cpp:619-633`: append each copy's translated area whole — no
/// subtraction at append; the map-level claimed-area loop (:953-958)
/// runs after the masks/retention.
pub(super) fn append(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    (area, forbidden, islands): (&[ExPolygon], &[ExPolygon], &[ExPolygon]),
) -> Result<OwnerAreas, crate::SliceError> {
    let (resolved, group) = traversal.print_object_config(object_index);
    let object = &traversal.project.objects()[resolved.source_object_index];
    let center = crate::project_slice::centering::center(object, group, traversal.scale)
        .expect("sliced object retains its raw center");
    let mut entries = Vec::new();
    let mut owner_forbidden = Vec::new();
    for (copy, &member) in group.instance_indices.iter().enumerate() {
        let shift = footprint::instance_shift(&object.instances()[member], center, traversal.scale);
        owner_forbidden.extend(translate(forbidden.to_vec(), shift));
        entries.push(InstanceEntry {
            copy,
            area: translate(area.to_vec(), shift),
            islands: translate(islands.to_vec(), shift),
        });
    }
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
