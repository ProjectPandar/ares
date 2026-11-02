//! `PartPlate::check_outside` (`PartPlate.cpp:4524-4625` at v2.5.0-beta `345efec64`): the plate box
//! test, the sinking volume test, the bed exclusions, through the config regions the GUI reads
//! (`:4562-4601`) or else the legacy area boxes (`:4603-4612`), and the IMEX zones
//! (`:4615-4621`). See `docs/architecture/web-plater-arrange-imex.md`, "A2b2" and "A2c2".

use super::{InstanceKey, OutsideInputs, PlateList};
use crate::arrangement::imex_zones::imex_hull_violates_zones;
use crate::bed_exclusion::intersection::{part_intersects, triangulated_prism_footprints, z_slab};
use crate::bed_exclusion::{
    BedExcludeRegion, active_bed_exclude_volume_mode, get_bed_excluded_regions_by_extruder,
};
use crate::geometry::{Point, Polygon, intersection_polygons_paths};
use crate::plater::arrange_job::convex_hull_2d;
use crate::plater::selection::instance_bounding_box;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
use crate::{BedExcludeVolumeMode, Project, ProjectVolumeType, SliceError};

/// `SINKING_Z_THRESHOLD` (`Model.hpp:1967`, a `float`).
const SINKING_Z_THRESHOLD: f64 = -0.001_f32 as f64;

/// `scale_(v)` into `Point(double, double)`, which rounds.
fn scaled_point(x: f64, y: f64) -> Point {
    Point::new((x / 1e-6).round() as i64, (y / 1e-6).round() as i64)
}

fn intersects(a: &Polygon, b: &Polygon) -> bool {
    !intersection_polygons_paths(std::slice::from_ref(a), std::slice::from_ref(b))
        .expect("exclusion intersection")
        .is_empty()
}

/// `ModelInstance::intersects_bed_exclude_region` (`Model.cpp:3753-3836`), yes/no: any model part
/// through `instance matrix * volume matrix`.
fn instance_intersects_region(
    project: &Project,
    (object, instance): InstanceKey,
    region: &BedExcludeRegion,
) -> bool {
    if region.polygon.points().len() < 3 {
        return false;
    }
    let Some(slab) = z_slab(region) else {
        return false;
    };
    let footprints = triangulated_prism_footprints(&region.polygon);
    if footprints.is_empty() {
        return false;
    }
    let object = &project.objects()[object];
    let instance_matrix = object.instances()[instance].transform();
    object
        .volumes()
        .iter()
        .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
        .any(|volume| {
            let mesh = volume.mesh();
            let matrix = instance_matrix.then(volume.transform());
            part_intersects(
                (mesh.vertices(), mesh.triangles()),
                matrix,
                slab,
                &footprints,
            )
        })
}

/// The legacy area boxes (`m_exclude_bounding_box`, `PartPlate.cpp:626-643`): every four
/// `bed_exclude_area` points moved by the plate origin (`set_shape`, `:5159-5161`), as
/// `BoundingBoxf3::polygon(true)` (`BoundingBox.cpp:119-134`) with truncated coordinates.
fn exclude_boxes(config: &DynamicPrintConfig, origin: [f64; 2]) -> Vec<Polygon> {
    let area = config.option("bed_exclude_area").points();
    area.as_chunks::<4>()
        .0
        .iter()
        .map(|corners| {
            let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
            for corner in corners {
                for axis in 0..2 {
                    let value = corner[axis] + origin[axis];
                    min[axis] = min[axis].min(value);
                    max[axis] = max[axis].max(value);
                }
            }
            let point = |x: f64, y: f64| Point::new((x / 1e-6) as i64, (y / 1e-6) as i64);
            Polygon::new(vec![
                point(min[0], min[1]),
                point(max[0], min[1]),
                point(max[0], max[1]),
                point(min[0], max[1]),
            ])
        })
        .collect()
}

impl PlateList {
    /// `check_outside`.
    pub(super) fn outside(
        &self,
        index: usize,
        key: InstanceKey,
        (project, config, zones): OutsideInputs<'_>,
    ) -> Result<bool, SliceError> {
        let mut hull = None::<Polygon>;
        let mut instance_hull = || {
            hull.get_or_insert_with(|| {
                let object = &project.objects()[key.0];
                convex_hull_2d(object, object.instances()[key.1].transform())
            })
            .clone()
        };
        let origin = self.plates[index].origin;
        if self.outside_plate_or_exclusions(origin, key, (project, config), &mut instance_hull)? {
            return Ok(true);
        }
        // `imex_hull_violates` (`:1174-1178`) on the plate's secondary zones and collision strips.
        let zones = &zones[index];
        if zones.secondary_zone_boxes.is_empty() && zones.collision_zones.is_empty() {
            return Ok(false);
        }
        let hull = instance_hull();
        Ok(imex_hull_violates_zones(&zones.secondary_zone_boxes, &hull)
            || imex_hull_violates_zones(&zones.collision_zones, &hull))
    }

    /// `check_outside` up to its IMEX branch.
    fn outside_plate_or_exclusions(
        &self,
        origin: [f64; 2],
        key: InstanceKey,
        (project, config): (&Project, &DynamicPrintConfig),
        instance_hull: &mut impl FnMut() -> Polygon,
    ) -> Result<bool, SliceError> {
        let instance = instance_bounding_box(project, key);
        let (mut min, max) = self.plate_box(origin);
        if instance.1[2] > min[2] {
            min[2] += instance.0[2];
        }
        if instance.0[2] < SINKING_Z_THRESHOLD {
            let intersects =
                (0..3).all(|axis| min[axis] < instance.1[axis] && max[axis] > instance.0[axis]);
            if !intersects {
                return Ok(true);
            }
            return crate::project_slice::instance_partly_outside(
                project,
                (origin[0], origin[1]),
                key,
            );
        }
        if !(0..3).all(|axis| min[axis] <= instance.0[axis] && instance.1[axis] <= max[axis]) {
            return Ok(true);
        }
        let regions_by_extruder = get_bed_excluded_regions_by_extruder(config);
        let mode = active_bed_exclude_volume_mode(config);
        let offset = scaled_point(origin[0], origin[1]);
        let mut checked_config_regions = false;
        for region_src in regions_by_extruder.first().into_iter().flatten() {
            if mode != BedExcludeVolumeMode::Shared && region_src.is_collision_volume() {
                continue;
            }
            checked_config_regions = true;
            let mut region = Polygon::new(
                region_src
                    .polygon
                    .points()
                    .iter()
                    .map(|p| Point::new(p.x() + offset.x(), p.y() + offset.y()))
                    .collect(),
            );
            if !region.is_counter_clockwise() {
                region.reverse();
            }
            if !intersects(&region, &instance_hull()) {
                continue;
            }
            if instance.1[2] < region_src.z_min || instance.0[2] > region_src.z_max {
                continue;
            }
            let translated = BedExcludeRegion {
                polygon: region,
                ..region_src.clone()
            };
            if instance_intersects_region(project, key, &translated) {
                return Ok(true);
            }
        }
        if !checked_config_regions {
            let boxes = exclude_boxes(config, origin);
            if boxes
                .iter()
                .any(|exclude_box| intersects(exclude_box, &instance_hull()))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
